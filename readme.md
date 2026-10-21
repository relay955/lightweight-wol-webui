## Lightweight WOL WebUI
a simple windows web ui for Wake-on-LAN

![img.png](img.png)
### Features
- Written in Rust for minimal memory usage
- JWT-based authentication: Login session maintains with periodic connections
- Runs web service in tray without console
- Simple user interface

### Execution
Extract the uploaded zip file from releases and run the exe.
default port is 25533(http://localhost:25533).

### Configuration
Port number can be modified in the configuration file (Rocket.toml) if needed.

### Windows Agent
The standalone Rust Windows service agent lives in [agent/](agent/README.md).
It connects to a configured WSS endpoint without a tray icon or an interactive login.
See the agent guide for service registration, removal and logging.

### Caution
This program is designed to be used through a reverse-proxy when accessed from outside.  
Using this program exposed externally without TLS could result in password exposure.

