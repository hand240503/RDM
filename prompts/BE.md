# PROJECT IMPLEMENTATION PROMPT

# Rust Device Manager (RDM) – Phase 1: Battery & Thermal Monitoring

## 1. ROLE

Act as a Senior Rust Engineer, Android Systems Engineer, and Embedded Device Monitoring Architect.

Your task is to independently implement, build, test, and document Phase 1 of Rust Device Manager (RDM) on a real Android device.

Do not merely provide code snippets or theoretical instructions. Execute the development workflow using available terminal tools, inspect errors, fix them, and verify the final result.

Do not claim a build or test succeeded unless you actually executed it and observed the result.

## 2. PROJECT OBJECTIVE

Build a lightweight Rust-based Android Device Management Agent for an OPPO Reno2 F.

The application will eventually manage:

- Battery health and charging.
- Device temperature and thermal conditions.
- CPU and RAM.
- External SSD storage.
- USB OTG connection.
- Camera/NVR workloads.
- Device alerts and system protection.

For this phase, implement ONLY:

1. Battery Manager.
2. Thermal Manager.
3. Device Monitoring Core.
4. Local data collection and logging.
5. CLI monitoring interface.

Do not implement NVR or SSD management yet.

## 3. TARGET HARDWARE AND ENVIRONMENT

| Property                | Value                              |
| ----------------------- | ---------------------------------- |
| Device                  | OPPO Reno2 F                       |
| Android                 | Android 11                         |
| Development environment | Termux                             |
| Programming language    | Rust                               |
| Target architecture     | Detect actual device architecture  |
| Runtime                 | Android / Termux                   |
| SSD                     | 256GB (future phase)               |
| Root access             | Not available / do not assume root |

The program must work without root privileges.

First inspect the actual environment:

- Android version.
- CPU architecture.
- Rust toolchain.
- Cargo version.
- Termux environment.
- Available filesystem permissions.
- Available Android system information.
- Available thermal and battery interfaces.

Do not assume Linux desktop APIs work identically on Android.

## 4. TECHNICAL REQUIREMENTS

### 4.1 Rust architecture

Use modular Rust architecture.

Suggested dependencies:

- tokio: asynchronous runtime.
- serde: serialization.
- serde_json: JSON serialization.
- chrono: timestamps.
- anyhow: error handling.
- thiserror: typed errors.
- tracing: logging.
- tracing-subscriber: log output.
- sysinfo: system information, only where supported.
- clap: CLI argument parsing.

Use stable Rust and select compatible crate versions.

Avoid unnecessary dependencies.

### 4.2 Battery Manager

Collect the following metrics where available:

| Metric              | Description                             |
| ------------------- | --------------------------------------- |
| Battery Level       | Battery percentage                      |
| Charging Status     | Charging / Discharging / Full / Unknown |
| Battery Temperature | Battery temperature in Celsius          |
| Battery Voltage     | Voltage in millivolts                   |
| Battery Current     | Current if accessible                   |
| Battery Health      | Android-reported health                 |
| Battery Technology  | Battery technology if available         |

Android 11 may expose battery data through Android BatteryManager APIs or system interfaces.

IMPORTANT:

- Do not assume every metric is accessible.
- Do not invent missing values.
- Return null or Unknown when unavailable.
- Document the source of every metric.
- Distinguish unavailable metrics from hardware faults.

### 4.3 Thermal Manager

Inspect available thermal sources.

Possible sources:

- Android thermal interfaces.
- /sys/class/thermal/
- /sys/devices/virtual/thermal/
- Android battery temperature.
- Other accessible thermal zones.

Requirements:

- Discover available sensors dynamically.
- Identify sensor names where possible.
- Convert units correctly.
- Handle permission errors.
- Never assume sensor indexes represent specific components.
- Do not hardcode CPU or GPU identification without evidence.

### 4.4 Monitoring Core

Implement:

- Configurable polling interval.
- Default polling interval: 30 seconds.
- Continuous monitoring.
- Graceful shutdown.
- Error recovery.
- Structured logging.
- Timestamped measurements.

The monitoring process must not crash when one sensor becomes unavailable.

### 4.5 Alert Engine

Implement configurable warning thresholds.

Initial suggested values:

- Battery temperature warning: 40°C.
- Battery temperature critical: 45°C.
- General thermal warning: use discovered sensor information and conservative configurable thresholds.
- Charging state abnormality: configurable.
- Battery level warning: 20%.

These values are initial monitoring thresholds, NOT guaranteed manufacturer safety limits.

Alerts should include:

- Alert ID.
- Timestamp.
- Metric.
- Observed value.
- Threshold.
- Severity.
- Description.

Do not automatically shut down the phone or alter charging behavior.

### 4.6 Data Storage

Store monitoring history locally.

Use SQLite or JSONL, choosing the simplest reliable implementation for Phase 1.

Requirements:

- Timestamped records.
- No duplicate records from repeated polling.
- Persistent history after restart.
- Configurable retention.
- Safe file handling.
- No storage outside permitted Termux directories.

Suggested output:

data/
rdm.db

logs/
rdm.log

config/
device.toml

### 4.7 CLI Interface

Implement the following commands:

rdm info
rdm battery
rdm thermal
rdm monitor
rdm alerts
rdm sensors

Example:

$ rdm battery

## Battery Information

Level: 87%
Charging: Charging
Temperature: 34.2 C
Voltage: 4280 mV
Health: Good

Unavailable fields must be shown as Unknown.

Example:

$ rdm thermal

## Thermal Sensors

Sensor 1: Unknown
Sensor 2: 36.5 C
Sensor 3: Permission denied

Example:

$ rdm monitor --interval 30

Starting Rust Device Manager...
Monitoring interval: 30 seconds
Press Ctrl+C to stop.

## 5. ANDROID INTEGRATION STRATEGY

Use a layered architecture.

Rust Core:

- Business logic.
- Data models.
- Threshold calculations.
- Alert processing.
- Storage.
- CLI.

Android Integration:

- Access Android BatteryManager when necessary.
- Use Termux-supported interfaces where appropriate.
- Keep Android-specific code isolated.
- Provide a clear path for future JNI/Kotlin integration.

Do not pretend that Termux shell access is equivalent to unrestricted Android API access.

If direct battery metrics cannot be obtained reliably from Rust, implement a small Android bridge only if the development environment supports building and installing it.

Do not silently replace real measurements with fake test data.

## 6. PROJECT STRUCTURE

Create:

rust-device-manager/
├── Cargo.toml
├── README.md
├── config/
│ └── device.toml
├── crates/
│ ├── rdm-core/
│ ├── battery-manager/
│ ├── thermal-manager/
│ ├── monitoring-engine/
│ ├── alert-manager/
│ └── storage-engine/
├── src/
│ └── main.rs
├── tests/
├── data/
└── logs/

You may improve this architecture if a simpler workspace structure provides better maintainability.

Do not create unnecessary placeholder files.

## 7. TESTING REQUIREMENTS

Implement unit and integration tests for:

- Battery data parsing.
- Temperature conversion.
- Voltage conversion.
- Missing sensor handling.
- Permission denied handling.
- Threshold detection.
- Alert generation.
- SQLite persistence.
- Monitoring loop.
- Graceful shutdown.

Use mocked data ONLY in tests.

For actual device testing:

- Read battery information.
- Discover thermal sensors.
- Run monitoring for at least 5 minutes.
- Verify logs.
- Verify database records.
- Verify behavior when a metric is unavailable.

Report exactly which tests ran and their results.

## 8. BUILD AND DEPLOYMENT

Independently:

1. Inspect Termux.
2. Check Rust installation.
3. Install missing development dependencies when permitted.
4. Create project files.
5. Implement source code.
6. Run cargo fmt.
7. Run cargo check.
8. Run cargo test.
9. Run cargo build --release.
10. Execute the binary on the Android device.
11. Verify actual hardware readings.

Do not overwrite existing user files without permission.

Do not request root access.

Do not expose sensitive environment variables in logs.

If a command fails:

- Capture the error.
- Determine the cause.
- Fix the implementation.
- Retry.
- Document unresolved issues honestly.

## 9. DELIVERABLES

Provide:

1. Complete Rust source code.
2. Cargo.toml and dependency configuration.
3. Configuration examples.
4. README.md.
5. Termux installation instructions.
6. Build instructions.
7. Execution instructions.
8. Testing report.
9. Actual hardware monitoring output.
10. Known limitations.
11. Recommended next steps for Phase 2.

The project must be reproducible.

## 10. DEFINITION OF DONE

Phase 1 is complete only when:

- The Rust project builds successfully on the target environment.
- Battery metrics are collected where accessible.
- Thermal sensors are discovered dynamically.
- Missing permissions do not crash the program.
- Monitoring runs continuously in the foreground.
- Alerts are generated correctly.
- History is persisted.
- CLI commands work.
- Unit tests pass.
- Actual OPPO Reno2 F test results are documented.

If a requirement cannot be completed due to Android restrictions, clearly identify the limitation and implement the best supported alternative.

## 11. FUTURE COMPATIBILITY

Design interfaces so later phases can add:

Phase 2: SSD / USB OTG Storage Manager.
Phase 3: CPU / RAM / Network Monitor.
Phase 4: Web Dashboard.
Phase 5: Device Protection and Android Foreground Service.
Phase 6: IMOU Ranger 2C Event-based NVR Integration.

Do not implement future phases prematurely.

## 12. FINAL EXECUTION RULE

You are expected to perform the work, not just explain how it could be performed.

Start by inspecting the actual OPPO Reno2 F Termux environment.

Then implement Phase 1 incrementally, build it, test it, and provide the final project files and execution report.

Never fabricate successful execution, hardware metrics, test results, or Android capabilities.
