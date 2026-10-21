#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod connection;
mod command;
mod logging;

use std::{ffi::OsString, time::Duration};
use tokio::{runtime::Runtime, sync::mpsc};
use windows_service::{
    define_windows_service,
    service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType},
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
};
use config::{AgentResult, Config};

const SERVICE_NAME: &str = "LightweightWolAgent";

fn main() {
    if let Err(error) = service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
        logging::write(&format!("서비스 관리자 연결 실패: {error}. 등록 스크립트로 실행하세요."));
        std::process::exit(1);
    }
}

define_windows_service!(ffi_service_main, service_main);

fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = run_service() { logging::write(&format!("서비스 오류: {error}")); }
}

/// 서비스 제어 요청을 전달하고 작업 결과에 따라 최종 종료 상태를 보고합니다.
fn run_service() -> AgentResult<()> {
    let (stop, receiver) = mpsc::unbounded_channel();
    let status = service_control_handler::register(SERVICE_NAME, move |control| {
        match control {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = stop.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    })?;
    let result = run_agent(&status, receiver);
    if let Err(error) = &result { logging::write(&format!("에이전트 실행 실패: {error}")); }
    let exit_code = if result.is_ok() { ServiceExitCode::Win32(0) }
        else { ServiceExitCode::ServiceSpecific(1) };
    report_status(&status, ServiceState::Stopped, exit_code)?;
    result
}

/// 설정을 읽고 연결 작업을 실행하며 중지 요청 시 런타임을 정리합니다.
fn run_agent(status: &ServiceStatusHandle, mut stop: mpsc::UnboundedReceiver<()>) -> AgentResult<()> {
    report_status(status, ServiceState::StartPending, ServiceExitCode::Win32(0))?;
    let config = Config::load()?;
    let runtime = Runtime::new()?;
    report_status(status, ServiceState::Running, ServiceExitCode::Win32(0))?;
    logging::write("서비스 시작");
    let mut connection = runtime.spawn(connection::run(config.server));
    let result = runtime.block_on(async {
        tokio::select! {
            _ = stop.recv() => Ok(()),
            result = &mut connection => {
                result?;
                Err("연결 작업이 예기치 않게 종료되었습니다.".into())
            }
        }
    });
    let stopping = report_status(status, ServiceState::StopPending, ServiceExitCode::Win32(0));
    connection.abort();
    runtime.shutdown_timeout(Duration::from_secs(2));
    logging::write("서비스 중지");
    stopping?;
    result
}

fn report_status(handle: &ServiceStatusHandle, state: ServiceState,
    exit_code: ServiceExitCode) -> windows_service::Result<()> {
    let pending = matches!(state, ServiceState::StartPending | ServiceState::StopPending);
    handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS, current_state: state, exit_code,
        controls_accepted: if state == ServiceState::Running {
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        } else { ServiceControlAccept::empty() },
        checkpoint: if pending { 1 } else { 0 },
        wait_hint: if pending { Duration::from_secs(10) } else { Duration::ZERO },
        process_id: None,
    })
}
