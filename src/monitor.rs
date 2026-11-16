use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tokio::sync::{RwLock, watch};
use tokio::time::{MissedTickBehavior, interval};

use crate::{device, model::Snapshot, storage::Storage};

pub async fn collect_once(storage: &Storage, snapshot: &Arc<RwLock<Snapshot>>) -> Result<Snapshot> {
    let collected = tokio::task::spawn_blocking(|| {
        let battery = device::battery_info();
        let thermal = device::thermal_sensors(&battery);
        let timestamp = battery.timestamp.clone();
        let sensors = device::sensor_inventory(&thermal, &timestamp);
        let alerts = device::evaluate_alerts(&battery);
        Snapshot {
            system: Some(device::system_info()),
            battery: Some(battery),
            thermal,
            sensors,
            alerts,
            collected_at: Some(timestamp),
        }
    })
    .await?;

    if let Some(battery) = &collected.battery {
        if let Err(error) = storage.record(battery, &collected.thermal).await {
            tracing::error!(%error, "Could not persist this monitoring sample");
        }
    }
    *snapshot.write().await = collected.clone();
    Ok(collected)
}

pub async fn run_monitor(
    storage: Storage,
    snapshot: Arc<RwLock<Snapshot>>,
    poll_interval: Duration,
    mut shutdown: watch::Receiver<bool>,
    print_samples: bool,
) {
    let mut ticker = interval(poll_interval.max(Duration::from_secs(1)));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match collect_once(&storage, &snapshot).await {
                    Ok(sample) if print_samples => println!("{}", serde_json::to_string(&sample).unwrap_or_else(|_| "{}".into())),
                    Ok(_) => tracing::debug!("Monitoring sample collected"),
                    Err(error) => tracing::error!(%error, "Monitoring collection failed"),
                }
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { break; }
            }
        }
    }
}
