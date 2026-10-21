# Lightweight WOL Agent

Windows 트레이에서 실행되는 Rust 기반 에이전트입니다. 시작 시 서버로 WSS 연결을 시도하고,
접속 실패나 연결 종료 시 설정된 간격으로 다시 연결합니다. 우클릭 메뉴에서 대상 URL과
현재 연결 상태를 확인하고 종료할 수 있습니다. Release 실행 시 콘솔 창을 띄우지 않습니다.

## 설정과 실행

독립 Cargo 프로젝트이므로 저장소 루트에서 다음과 같이 검사하거나 실행합니다.

```powershell
cargo check --manifest-path agent/Cargo.toml
cargo test --manifest-path agent/Cargo.toml
cargo run --manifest-path agent/Cargo.toml
```

Windows MSVC Rust 도구 체인과 Visual Studio C++ Build Tools, Windows SDK가 필요합니다.
아이콘은 기존 서버의 `../resources/`를 공유합니다.

설정은 작업 디렉터리가 아닌 **실행 파일 옆의 `Agent.toml`**에서 읽습니다.
파일이 없으면 `Agent.example.toml` 내용으로 자동 생성합니다. 개발 실행 시 경로는
`agent/target/debug/Agent.toml`입니다. 배포할 때는 exe 옆에 설정을 두면 됩니다.

```toml
[server]
url = "wss://wol.example.com/api/agent/connect"
token = ""
reconnect_seconds = 5
```

- `url`: 실제 WebSocket 엔드포인트를 지정합니다. WSS만 허용하며 인증서와 호스트 이름을
  Windows 신뢰 저장소를 통해 검증합니다. URL 내 인증 정보, 쿼리, fragment는 허용하지 않습니다.
- `token`: 기기별 토큰입니다. 값이 있으면 접속 요청에 `Authorization: Bearer <token>`을
  보냅니다. 빈 값이면 헤더를 생략하므로 인증 없는 개발 서버에만 사용하세요.
  현재는 TOML에 평문 보관하므로 파일 접근 권한을 제한하고 저장소에 커밋하지 마세요.
- `reconnect_seconds`: 재접속 간격으로, 기본값은 5초이며 1~3600초를 허용합니다.

설정을 바꾼 뒤에는 에이전트를 다시 실행합니다. 설정 오류는 메시지 창으로 표시합니다.
연결 시도 제한은 10초이며, 30초 간격 Ping에 대한 Pong이 90초간 없으면 재접속합니다.
트레이의 `연결됨`은 WebSocket 핸드셰이크 성공을 의미합니다.

## 서버 연동 범위

WOL 서버는 `/api/agent/connect`에서 장치의 `authkey`를 검증한 뒤 연결을 허가합니다.
장치 목록에서 키를 클릭해 복사하고 `Agent.toml`의 `server.token`에 설정합니다.
키가 없거나 일치하지 않으면 서버는 HTTP 401로 연결을 거부합니다.
서버 앞의 리버스 프록시에서 TLS를 처리하고 WebSocket 업그레이드와 Authorization 헤더를
전달해야 합니다. 프록시의 유휴 제한 시간은 Ping 간격인 30초보다 길게 설정합니다.

이번 구현은 연결과 상태 표시까지 제공합니다. 종료·재부팅 명령,
명령 ACK와 실행 결과 프로토콜은 포함하지 않습니다. 수신한 텍스트·바이너리 메시지는
실행하지 않고 무시합니다.
