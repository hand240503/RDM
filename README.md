# Rust Device Manager

Lightweight Rust backend and mobile dashboard for Android/Termux. The backend reads only interfaces visible to the current process, records samples in SQLite, and serves the API contract at `/api/v1`.

## Build and run

Requires a stable Rust toolchain and Cargo. From the project root:

```sh
cargo run --release
```

The server binds to `127.0.0.1:8080` by default and serves the dashboard from `FE/`. Open `http://127.0.0.1:8080/` in the phone browser. Monitoring runs in the backend process at a 30-second interval whether the dashboard is open or closed.

Environment settings:

| Variable | Default | Purpose |
|---|---|---|
| `RDM_BIND` | `127.0.0.1:8080` | HTTP bind address. Keep localhost unless LAN access is needed on a trusted network. |
| `RDM_DATA_DIR` | `data` | Directory for the SQLite database. |
| `RDM_POLL_SECONDS` | `30` | Collection interval, clamped to 1–3600 seconds. |
| `RDM_RETENTION_DAYS` | `30` | History retention period. |

## CLI

The executable supports `info`, `battery`, `thermal`, `sensors`, `alerts`, and `monitor`. With a release build, for example:

```sh
./target/release/rdm battery
./target/release/rdm monitor --interval 30
```

Running with no command or with `serve` starts the web/API service.

## Android readings and limitations

Battery values are read from readable entries under `/sys/class/power_supply/*` where the entry reports `type=Battery`. Voltage and current use Linux power-supply units and are converted from micro-units to milli-units. Battery temperature uses the standard tenths-of-a-degree unit, with a millidegree fallback when the first conversion is implausible. Thermal sensors are discovered under `/sys/class/thermal/thermal_zone*`; their kernel-reported `type` is retained as the name, and no zone index is treated as a CPU/GPU label.

Android/ColorOS permissions and vendor kernel interfaces vary. An unavailable value stays `null`; unreadable thermal zones remain visible with an error code when the zone itself can be enumerated. `getprop` supplies device model and Android release when available. `system.architecture` is the architecture this binary was compiled for. Host-side checks do not verify actual Reno2 F behavior.

Battery alert thresholds are the initial project values: warning at 40 °C, critical at 45 °C, and low level at 20%. They are monitoring thresholds only and do not alter charging or power behavior.

## API

The implementation follows `docs/API_CONTRACT.md`: health, system, battery, thermal sensors, sensor inventory, alerts, and battery/thermal history. History endpoints accept `?limit=60` and cap requests at 500 records.

## Termux install notes

Install the compiler and native build tools in Termux and copy this project into `~/rdm` in Termux's private home. Then build and start it:

```sh
pkg update
pkg install rust clang make pkg-config
cd ~/rdm
cargo run --release
```

Keep the database in Termux's private app storage. The sampler continues while the dashboard is closed as long as the backend process is running; Android/ColorOS may still suspend or terminate background processes, which must be checked on the phone. No root access or Android bridge is assumed. For access from another trusted LAN device, set `RDM_BIND=0.0.0.0:8080` and allow the port only on a trusted network; the current dashboard/API has no authentication, so do not expose it to untrusted networks.

## Verification

Run `cargo fmt --check`, `cargo check`, and `cargo test`. Device-specific readings and long-running behavior still need testing on the target OPPO Reno2 F.
