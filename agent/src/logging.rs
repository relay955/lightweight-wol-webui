use std::{env, fs::OpenOptions, io::Write, time::{SystemTime, UNIX_EPOCH}};

/// 실행 파일 옆 로그에 Unix 시각과 상태를 기록합니다.
pub fn write(message: &str) {
    let Ok(executable) = env::current_exe() else { return; };
    let path = executable.with_file_name("Agent.log");
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else { return; };
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let _ = writeln!(file, "[{seconds}] {message}");
}
