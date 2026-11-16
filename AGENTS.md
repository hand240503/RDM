
# RDM - PROJECT RULES

## Project
Rust Device Manager (RDM)

Target:
- OPPO Reno2 F
- Android 11
- Termux
- No root access

## Backend (BE)
- Rust
- Tokio
- Axum
- SQLite
- Responsible for hardware monitoring.
- Responsible for data collection and storage.
- Responsible for alerts.
- Provides REST API to FE.

## Frontend (FE)
- HTML5
- CSS3
- Vanilla JavaScript
- Lightweight SVG charts.
- Mobile-first responsive design.
- No React, Vue, Flutter or Electron.

## Architecture Rules
1. BE owns all monitoring logic.
2. FE only displays and controls supported features.
3. FE and BE must follow docs/API_CONTRACT.md.
4. Monitoring must continue when FE is closed.
5. Never fabricate hardware readings.
6. Do not assume Android APIs are unrestricted.
7. Do not introduce unnecessary dependencies.

## Development Rules
1. Inspect existing files before modifying.
2. Preserve existing working code.
3. Do not create unnecessary files.
4. Do not modify the other module without necessity.
5. Test every meaningful change.
6. Update IMPLEMENTATION_STATUS.md.
7. Never claim tests passed without execution.

## Execution Order
1. Implement BE.
2. Define and verify API contract.
3. Implement FE.
4. Integrate FE and BE.
5. Test on actual Android device.
