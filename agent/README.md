# Windows Agent

로그인 여부와 관계없이 실행되는 Windows 서비스 전용 에이전트입니다.
트레이 아이콘은 없으며, EXE를 직접 실행하는 대신 서비스로 등록해서 사용합니다.
서비스 이름은 `LightweightWolAgent`입니다.

## 빌드 및 준비

```powershell
cargo build --release --manifest-path agent/Cargo.toml
```

`agent/target/release/lightweight-wol-agent.exe`, `service.ps1`, `Agent.example.toml`을
설치 폴더에 복사합니다. 서비스는 LocalSystem 권한으로 실행되므로 관리자만 수정할 수 있는
폴더(예: `C:\Program Files\LightweightWolAgent`)에 배치합니다.
실행 중인 EXE를 교체할 때는 먼저 서비스를 중지합니다.

`Agent.example.toml`을 `Agent.toml`로 복사하고 서버 WSS 주소와 기기 토큰을 입력합니다.
설정과 로그는 작업 디렉터리가 아닌 실행 파일 옆에서 읽고 기록합니다.

## 서비스 등록

관리자 PowerShell에서 설치 폴더로 이동한 뒤 실행합니다.

```powershell
.\service.ps1
```

EXE가 다른 폴더에 있다면 경로를 지정할 수 있습니다.

```powershell
.\service.ps1 -ExecutablePath 'C:\Program Files\LightweightWolAgent\lightweight-wol-agent.exe'
```

설정이 없으면 예제 설정을 생성하고 등록을 중단합니다. 생성된 설정을 편집하고 다시 실행합니다.
이미 서비스가 등록되어 있다면 먼저 제거한 뒤 재등록합니다.

등록 시 자동 시작과 즉시 시작을 설정합니다. 서비스가 실패하면 10초, 30초, 이후 60초 간격으로
재시작하며, 실패 횟수는 실패 없이 하루가 지나면 초기화됩니다.
서버 접속 실패는 서비스 실패로 처리하지 않고 `reconnect_seconds` 간격으로 재접속합니다.

## 상태 확인 및 중지

```powershell
Get-Service LightweightWolAgent
Stop-Service LightweightWolAgent
Start-Service LightweightWolAgent
Get-Content .\Agent.log -Tail 30
```

서비스 시작·중지, 연결·재접속 상태, 초기화 오류는 실행 파일 옆 `Agent.log`에 기록합니다.
각 행의 대괄호 안 숫자는 Unix 시각(초)입니다. 로그 자동 순환은 없으므로 장기 운영 시
필요에 따라 서비스를 중지한 상태에서 로그를 정리합니다.
서비스의 `Running` 상태는 로컬 작업이 실행 중이라는 뜻이며 서버 연결 성공을 보장하지 않습니다.
설정 변경은 서비스를 재시작해야 적용됩니다.

## 서비스 제거

```powershell
.\service.ps1 -Action Uninstall
```

서비스를 중지하고 등록을 제거합니다. 실행 파일, 설정과 로그는 보존합니다.
서비스 관리 창이 열려 있으면 삭제 완료가 지연될 수 있으므로 재등록 전 창을 닫습니다.
