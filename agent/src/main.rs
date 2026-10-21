#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod connection;
mod command;

use std::{sync::mpsc, time::Duration};
use tokio::runtime::Runtime;
use tray_item::{IconSource, TrayItem};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
use config::{AgentResult, Config};
use connection::TrayEvent;

fn main() {
    if let Err(error) = run() {
        let message: Vec<u16> = format!("에이전트를 시작할 수 없습니다.\n{error}")
            .encode_utf16().chain(Some(0)).collect();
        let title: Vec<u16> = "Lightweight WOL Agent".encode_utf16().chain(Some(0)).collect();
        unsafe { MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR); }
    }
}

/// 트레이의 주소와 연결 상태를 갱신하고 종료 시 연결 작업을 정리합니다.
fn run() -> AgentResult<()> {
    let config = Config::load()?;
    let runtime = Runtime::new()?;
    let (events, receiver) = mpsc::channel();
    let mut tray = TrayItem::new("Lightweight WOL Agent", IconSource::Resource("exe-icon"))?;
    tray.add_label(&format!("대상: {}", config.server.url))?;
    let status_id = tray.inner_mut().add_label_with_id("상태: 연결 대기")?;
    tray.inner_mut().add_separator()?;
    let quit_events = events.clone();
    tray.add_menu_item("종료", move || { let _ = quit_events.send(TrayEvent::Quit); })?;

    let connection = runtime.spawn(connection::run(config.server, events));
    let result = (|| -> AgentResult<()> {
        for event in receiver {
            match event {
                TrayEvent::Status(status) => {
                    tray.inner_mut().set_label(&format!("상태: {status}"), status_id)?;
                }
                TrayEvent::Quit => break,
            }
        }
        Ok(())
    })();
    connection.abort();
    runtime.shutdown_timeout(Duration::from_secs(2));
    drop(tray);
    result
}
