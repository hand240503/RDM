# FRONTEND IMPLEMENTATION PROMPT
# Rust Device Manager (RDM) – Lightweight Web Dashboard

## 1. ROLE

Act as a Senior Frontend Engineer and UI/UX Designer specializing in lightweight system monitoring dashboards.

Your task is to independently design, implement, integrate, and test the frontend for Rust Device Manager (RDM).

Do not merely provide design suggestions or code snippets. Create the actual frontend files, connect them to the Rust backend API, run the application, and verify functionality.

## 2. PROJECT OBJECTIVE

Build a beautiful, modern, lightweight device monitoring dashboard for:

- Device: OPPO Reno2 F
- Android: Android 11
- Runtime: Termux
- Backend: Rust
- Frontend: HTML5 + CSS3 + Vanilla JavaScript
- Display: Mobile-first, responsive
- Access: Local browser through the Rust HTTP server

The frontend must monitor the phone itself.

The initial dashboard focuses on:

1. Battery monitoring.
2. Temperature monitoring.
3. Device health overview.
4. Monitoring history.
5. Alert management.
6. Backend and sensor availability.

Future compatibility:

- CPU monitoring.
- RAM monitoring.
- SSD monitoring.
- USB OTG monitoring.
- Network monitoring.
- Camera/NVR monitoring.

Do not implement future monitoring modules until the backend supports them.

## 3. STRICT TECHNOLOGY REQUIREMENTS

Use:

| Component | Technology |
|---|---|
| Backend integration | Rust REST API |
| Frontend | HTML5 |
| Styling | CSS3 |
| Interaction | Vanilla JavaScript |
| Charts | Lightweight SVG or uPlot |
| Data format | JSON |
| Icons | Lucide Static SVG or inline SVG |
| Fonts | System font stack |
| Build system | No build system required |
| Framework | None |

DO NOT USE:

- React
- Vue
- Angular
- Next.js
- Nuxt
- Flutter
- Electron
- Tauri
- Bootstrap
- Tailwind CSS
- Heavy component libraries
- Unnecessary animation libraries

The frontend should work without Node.js at runtime.

Avoid unnecessary dependencies and external CDN requirements.

## 4. DESIGN DIRECTION

Create a professional system monitoring interface inspired by modern infrastructure monitoring software.

Visual direction:

- Minimalist.
- Modern.
- Clean.
- Premium appearance.
- Information-dense but readable.
- Dark-first interface.
- Mobile-first responsive layout.

Suggested design references:

- Grafana
- Uptime Kuma
- Netdata
- Modern NAS management interfaces

Do not copy their interfaces directly.

### Color palette

Use CSS variables:

```css
:root {
    --bg-primary: #0B1120;
    --bg-secondary: #111827;
    --bg-card: #172033;
    --bg-card-hover: #1E293B;

    --accent-primary: #38BDF8;
    --accent-secondary: #818CF8;

    --status-success: #22C55E;
    --status-warning: #F59E0B;
    --status-critical: #EF4444;

    --text-primary: #F8FAFC;
    --text-secondary: #94A3B8;
    --border-color: #263449;
}
```

Keep colors consistent across the application.

Support light and dark themes if this can be implemented without unnecessary complexity. Dark mode should be the default.

## 5. MAIN DASHBOARD

Design the main dashboard with the following sections.

### 5.1 Header

Display:

- RDM logo/icon.
- Application name: Rust Device Manager.
- Device name: OPPO Reno2 F.
- Backend connection status.
- Last updated timestamp.
- Theme toggle.

Example:

```text
RDM                         ● SYSTEM ONLINE
Rust Device Manager         OPPO Reno2 F
```

Connection states:

- Online.
- Connecting.
- Offline.
- API Error.

### 5.2 System Overview

Create compact monitoring cards.

Card 1: Battery

Display:

- Battery percentage.
- Charging status.
- Battery temperature.
- Voltage, if available.
- Battery health, if available.

Use a simple battery icon and progress indicator.

Card 2: Thermal

Display:

- Battery temperature.
- Available thermal sensors.
- Highest detected temperature.
- Thermal warning state.

Do not invent CPU or GPU temperatures.

Card 3: Monitoring Agent

Display:

- Agent status.
- Monitoring uptime.
- Polling interval.
- Last successful collection.
- Number of available sensors.

Card 4: Alerts

Display:

- Critical alerts.
- Warning alerts.
- Active alerts.
- Most recent alert.

Unknown metrics must display as `N/A` or `Unknown`.

Never display fake readings.

### 5.3 Battery Detail

Create a dedicated battery section.

Display:

- Large battery percentage.
- Charging/discharging status.
- Battery temperature.
- Voltage.
- Current, if available.
- Battery health.
- Historical battery level.

Use an SVG chart or lightweight chart library.

Chart requirements:

- Display the last 30–60 data points.
- Smooth but minimal rendering.
- No unnecessary animation.
- Clear timestamps.
- Responsive sizing.

### 5.4 Thermal Detail

Create a thermal monitoring section.

Display discovered sensors dynamically.

Example:

```text
THERMAL MONITOR

Battery             34.2 °C
Thermal Zone 0      38.1 °C
Thermal Zone 1      Unknown
Thermal Zone 2      Permission Denied
```

Requirements:

- Never hardcode sensor indexes as CPU/GPU.
- Display actual sensor names when available.
- Show unavailable sensors explicitly.
- Highlight warning and critical conditions.
- Show temperature history where data exists.

### 5.5 Alert Center

Create an alert list.

Each alert must show:

- Severity.
- Alert description.
- Metric.
- Observed value.
- Threshold.
- Timestamp.

Severity levels:

- INFO
- WARNING
- CRITICAL

Use distinct but restrained visual indicators.

Allow filtering by severity.

If the backend does not support acknowledging or resolving alerts, do not create fake interactive buttons for those actions.

### 5.6 Sensor Inventory

Display all discovered sensors.

Columns:

- Sensor name.
- Sensor type.
- Current value.
- Unit.
- Availability.
- Last updated.

Support mobile-friendly cards instead of forcing a wide table.

## 6. NAVIGATION

Create a simple navigation structure:

1. Overview
2. Battery
3. Thermal
4. Alerts
5. Sensors
6. Settings

On mobile:

- Use a compact bottom navigation bar or collapsible sidebar.
- Avoid taking too much screen space.
- Keep important monitoring information accessible.

Settings should initially support:

- Theme selection.
- Refresh interval.
- Temperature warning display preferences.

Only expose settings that the backend actually supports.

## 7. BACKEND API INTEGRATION

The frontend must communicate with the Rust Axum backend.

Do not assume API endpoints already exist.

First inspect the Rust backend.

If endpoints are missing, coordinate with the backend implementation and propose a minimal API contract.

Suggested API:

```text
GET /api/v1/system
GET /api/v1/battery
GET /api/v1/thermal
GET /api/v1/sensors
GET /api/v1/alerts
GET /api/v1/history/battery
GET /api/v1/history/thermal
GET /api/v1/health
```

Example battery response:

```json
{
  "level": 87,
  "charging_status": "Charging",
  "temperature_celsius": 34.2,
  "voltage_mv": 4280,
  "current_ma": null,
  "health": "Good",
  "timestamp": "2026-11-16T15:00:00Z"
}
```

Example thermal response:

```json
{
  "sensors": [
    {
      "name": "battery",
      "value_celsius": 34.2,
      "available": true
    },
    {
      "name": "thermal_zone_1",
      "value_celsius": null,
      "available": false
    }
  ]
}
```

These are example contracts, not actual device readings.

Requirements:

- Use `fetch()` for REST requests.
- Handle connection failures.
- Handle invalid JSON.
- Handle null values.
- Handle missing fields.
- Display loading states.
- Display API errors clearly.
- Avoid crashing the UI when one endpoint fails.
- Do not use mock data in production mode.

If mock data is needed for frontend development, place it in a separate development-only module and make the mode clearly visible.

## 8. PERFORMANCE REQUIREMENTS

The application must remain lightweight.

Target:

- Minimal JavaScript.
- Minimal DOM updates.
- No unnecessary polling.
- No continuous animations.
- No heavy chart libraries.
- No unnecessary background tasks.
- No memory leaks from timers or event listeners.

Polling strategy:

- Default dashboard refresh: 5 seconds.
- Configurable refresh interval.
- Avoid overlapping API requests.
- Pause or reduce polling when the page is hidden.
- Resume polling when the page becomes visible.

Important:

The Rust monitoring backend must continue collecting data independently of whether the dashboard is open.

The frontend must not be responsible for collecting hardware metrics.

## 9. RESPONSIVE DESIGN

Optimize for:

- OPPO Reno2 F mobile display.
- Android browser.
- Desktop browser.
- Tablet.

Mobile requirements:

- No horizontal overflow.
- Readable text.
- Touch-friendly controls.
- Compact cards.
- Clear navigation.
- Good contrast.
- Charts resize correctly.

Use CSS Grid and Flexbox.

## 10. ACCESSIBILITY AND RELIABILITY

Implement:

- Semantic HTML.
- Accessible labels.
- Keyboard navigation where relevant.
- Sufficient color contrast.
- Visible focus states.
- Error and empty states.
- Graceful handling of missing data.

Do not communicate status through color alone.

## 11. PROJECT STRUCTURE

Create the frontend inside the existing RDM project.

Suggested structure:

```text
rust-device-manager/
├── src/
│   └── ...
├── frontend/
│   ├── index.html
│   ├── assets/
│   │   ├── css/
│   │   │   ├── main.css
│   │   │   ├── dashboard.css
│   │   │   └── responsive.css
│   │   ├── js/
│   │   │   ├── app.js
│   │   │   ├── api.js
│   │   │   ├── dashboard.js
│   │   │   ├── battery.js
│   │   │   ├── thermal.js
│   │   │   ├── alerts.js
│   │   │   └── charts.js
│   │   └── icons/
│   └── README.md
├── Cargo.toml
└── README.md
```

You may simplify the structure if appropriate.

Do not create unnecessary files.

## 12. RUST BACKEND INTEGRATION

Use Axum to serve static frontend files and REST endpoints.

Requirements:

- Serve `frontend/` locally.
- Bind to localhost by default.
- Allow configurable bind address.
- Do not expose the dashboard to the public internet by default.
- Add CORS only if genuinely required.
- Validate API responses.
- Keep hardware monitoring independent of frontend requests.

Suggested access:

```text
http://127.0.0.1:8080
```

If the user needs to access the dashboard from another device on the same trusted network, document how to configure a LAN bind address and the associated security implications.

Do not expose an unauthenticated management interface to an untrusted network.

## 13. TESTING

Execute and report:

1. HTML validation.
2. CSS and JavaScript syntax checks.
3. API connectivity tests.
4. Missing/null data tests.
5. Backend offline tests.
6. Mobile viewport tests.
7. Desktop viewport tests.
8. Chart rendering tests.
9. Polling and visibility behavior.
10. Memory/timer cleanup checks.

Verify the dashboard against the actual Rust backend.

Do not claim tests passed unless they were executed.

## 14. DEFINITION OF DONE

The frontend is complete only when:

- Dashboard renders correctly.
- UI is responsive on mobile.
- UI works on Android 11 browser.
- Rust API integration works.
- Battery information is displayed from actual backend data.
- Thermal sensor information is displayed from actual backend data.
- Alerts are displayed correctly.
- Missing data is handled honestly.
- Polling does not overlap or leak.
- Backend continues monitoring when the dashboard is closed.
- No unnecessary frontend framework is introduced.
- Documentation explains how to run and access the dashboard.

## 15. FINAL DELIVERABLES

Provide:

1. Complete frontend source code.
2. CSS design system.
3. JavaScript modules.
4. API contract.
5. Rust Axum integration changes.
6. README.
7. Android browser access instructions.
8. Test report.
9. Screenshots of the actual running dashboard.
10. Known limitations and future improvements.

## 16. EXECUTION RULE

Start by inspecting the existing RDM project.

Do not create a separate incompatible application.

Preserve the existing Rust backend and project decisions.

Implement the frontend incrementally, integrate it with the backend, run it, test it, and fix errors.

Prioritize:

1. Lightweight performance.
2. Beautiful UI.
3. Accurate monitoring data.
4. Mobile usability.
5. Maintainable code.

Do not sacrifice performance for visual effects.

Do not fabricate hardware readings, API responses, screenshots, or test results.