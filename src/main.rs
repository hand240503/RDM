mod api;
mod device;
mod model;
mod monitor;
mod storage;

use std::{env, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::Router;
use model::Snapshot;
use tokio::{
    net::TcpListener,
    sync::{RwLock, watch},
};
use tracing_subscriber::EnvFilter;

fn data_dir() -> PathBuf {
    env::var_os("RDM_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn poll_seconds() -> u64 {
    env::var("RDM_POLL_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
        .clamp(1, 3600)
}

async fn storage() -> Result<storage::Storage> {
    let dir = data_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("create data directory {}", dir.display()))?;
    let db = dir.join("rdm.db");
    let path = db.to_string_lossy();
    let retention = env::var("RDM_RETENTION_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    storage::Storage::open(&path, retention).await
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

async fn run_server() -> Result<()> {
    let storage = storage().await?;
    let snapshot = Arc::new(RwLock::new(Snapshot::default()));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let monitor_storage = storage.clone();
    let monitor_snapshot = Arc::clone(&snapshot);
    let interval = Duration::from_secs(poll_seconds());
    let monitor_handle = tokio::spawn(monitor::run_monitor(
        monitor_storage,
        monitor_snapshot,
        interval,
        shutdown_rx,
        false,
    ));

    let bind = env::var("RDM_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let address = tokio::net::lookup_host(&bind)
        .await?
        .next()
        .context("RDM_BIND did not resolve to an address")?;
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("bind to {address}"))?;
    tracing::info!(address = %listener.local_addr()?, poll_seconds = interval.as_secs(), "RDM backend started");

    let state = api::AppState {
        snapshot,
        storage,
        started: std::time::Instant::now(),
    };
    let frontend = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("FE");
    let app: Router = api::app_router(state, &frontend.to_string_lossy());
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::warn!(%error, "Could not listen for Ctrl+C");
            }
        })
        .await?;
    let _ = shutdown_tx.send(true);
    if let Err(error) = monitor_handle.await {
        tracing::warn!(%error, "Monitoring task stopped unexpectedly");
    }
    Ok(())
}

async fn cli(command: &str, interval_override: Option<u64>) -> Result<()> {
    match command {
        "info" => print_json(&device::system_info()),
        "battery" => print_json(&device::battery_info()),
        "thermal" => {
            let battery = device::battery_info();
            print_json(
                &serde_json::json!({ "sensors": device::thermal_sensors(&battery), "timestamp": device::timestamp() }),
            )
        }
        "sensors" => {
            let battery = device::battery_info();
            let thermal = device::thermal_sensors(&battery);
            print_json(
                &serde_json::json!({ "sensors": device::sensor_inventory(&thermal, &battery.timestamp) }),
            )
        }
        "alerts" => print_json(
            &serde_json::json!({ "alerts": device::evaluate_alerts(&device::battery_info()) }),
        ),
        "monitor" => {
            let store = storage().await?;
            let snapshot = Arc::new(RwLock::new(Snapshot::default()));
            let (tx, rx) = watch::channel(false);
            tracing::info!(
                poll_seconds = interval_override.unwrap_or_else(poll_seconds),
                "Foreground monitoring started; press Ctrl+C to stop"
            );
            tokio::select! {
                _ = monitor::run_monitor(store, snapshot, Duration::from_secs(interval_override.unwrap_or_else(poll_seconds)), rx, true) => {},
                _ = tokio::signal::ctrl_c() => { let _ = tx.send(true); }
            }
            Ok(())
        }
        "serve" => run_server().await,
        "help" | "--help" | "-h" => {
            println!(
                "Usage: rdm [serve|info|battery|thermal|sensors|alerts|monitor --interval SECONDS]"
            );
            println!("With no command, starts the local web/API server.");
            Ok(())
        }
        other => anyhow::bail!(
            "unknown command '{other}'. Use: serve | info | battery | thermal | sensors | alerts | monitor [--interval SECONDS]"
        ),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("serve");
    let interval_override = args
        .windows(2)
        .find(|pair| pair[0] == "--interval")
        .and_then(|pair| pair[1].parse().ok());
    cli(command, interval_override).await
}
