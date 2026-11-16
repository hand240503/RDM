use std::{collections::HashSet, fs, io, path::Path, process::Command};

use chrono::{SecondsFormat, Utc};

use crate::model::{Alert, BatteryInfo, SensorInfo, SystemInfo, ThermalSensor};

pub fn timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn read_text(path: &Path) -> io::Result<String> {
    fs::read_to_string(path).map(|value| value.trim().to_owned())
}

fn read_number(path: &Path) -> Option<i64> {
    read_text(path).ok()?.parse().ok()
}

fn property(name: &str) -> Option<String> {
    let output = Command::new("getprop").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn system_uptime() -> Option<u64> {
    let value = read_text(Path::new("/proc/uptime")).ok()?;
    value
        .split_whitespace()
        .next()?
        .split('.')
        .next()?
        .parse()
        .ok()
}

pub fn system_info() -> SystemInfo {
    SystemInfo {
        device_name: property("ro.product.model").unwrap_or_else(|| "Unknown device".into()),
        android_version: property("ro.build.version.release").unwrap_or_else(|| "Unknown".into()),
        architecture: std::env::consts::ARCH.to_owned(),
        uptime_seconds: system_uptime(),
        timestamp: timestamp(),
    }
}

fn power_supply_dirs() -> Vec<std::path::PathBuf> {
    fs::read_dir("/sys/class/power_supply")
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            read_text(&path.join("type")).is_ok_and(|kind| kind.eq_ignore_ascii_case("battery"))
        })
        .collect()
}

fn milli_celsius(value: i64) -> Option<f64> {
    let celsius = value as f64 / 1000.0;
    (-100.0..=250.0).contains(&celsius).then_some(celsius)
}

fn battery_temperature(value: i64) -> Option<f64> {
    // Linux power_supply temperature is normally reported in tenths of °C.
    // Some Android vendor drivers expose millidegrees; accept that representation
    // only when the tenths conversion is outside a plausible sensor range.
    let tenths = value as f64 / 10.0;
    if (-100.0..=250.0).contains(&tenths) {
        Some(tenths)
    } else {
        milli_celsius(value)
    }
}

pub fn battery_info() -> BatteryInfo {
    let dir = power_supply_dirs().into_iter().next();
    let read = |name: &str| dir.as_ref().and_then(|path| read_number(&path.join(name)));
    let status = dir
        .as_ref()
        .and_then(|path| read_text(&path.join("status")).ok())
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "charging" => "charging",
            "discharging" => "discharging",
            "full" => "full",
            "not charging" | "not_charging" => "not_charging",
            _ => "unknown",
        })
        .unwrap_or("unknown")
        .to_owned();

    BatteryInfo {
        level: read("capacity").filter(|value| (0..=100).contains(value)),
        charging_status: status,
        temperature_celsius: read("temp").and_then(battery_temperature),
        voltage_mv: read("voltage_now").map(|value| value / 1000),
        current_ma: read("current_now").map(|value| value / 1000),
        health: dir
            .as_ref()
            .and_then(|path| read_text(&path.join("health")).ok())
            .filter(|value| !value.is_empty()),
        technology: dir
            .as_ref()
            .and_then(|path| read_text(&path.join("technology")).ok())
            .filter(|value| !value.is_empty()),
        timestamp: timestamp(),
    }
}

fn sensor_error(error: &io::Error) -> String {
    if error.kind() == io::ErrorKind::PermissionDenied {
        "permission_denied".into()
    } else {
        "read_error".into()
    }
}

fn zone_dirs() -> Vec<std::path::PathBuf> {
    fs::read_dir("/sys/class/thermal")
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("thermal_zone"))
        })
        .collect()
}

pub fn thermal_sensors(battery: &BatteryInfo) -> Vec<ThermalSensor> {
    let mut sensors = Vec::new();
    let mut names = HashSet::new();
    for zone in zone_dirs() {
        let fallback = zone
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("thermal_zone");
        let name = read_text(&zone.join("type"))
            .ok()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| fallback.to_owned());
        names.insert(name.clone());
        let sensor_type = String::from("unknown");
        match read_number(&zone.join("temp")) {
            Some(raw) => {
                let value = milli_celsius(raw);
                sensors.push(ThermalSensor {
                    name,
                    sensor_type,
                    available: value.is_some(),
                    value_celsius: value,
                    error: value.is_none().then(|| "invalid_reading".into()),
                });
            }
            None => {
                let error = fs::read_to_string(zone.join("temp"))
                    .as_ref()
                    .err()
                    .map(sensor_error)
                    .unwrap_or_else(|| "invalid_reading".into());
                sensors.push(ThermalSensor {
                    name,
                    sensor_type,
                    value_celsius: None,
                    available: false,
                    error: Some(error),
                });
            }
        }
    }

    if let Some(value) = battery.temperature_celsius {
        if names.insert("battery".into()) {
            sensors.push(ThermalSensor {
                name: "battery".into(),
                sensor_type: "unknown".into(),
                value_celsius: Some(value),
                available: true,
                error: None,
            });
        }
    }
    sensors.sort_by(|a, b| a.name.cmp(&b.name));
    sensors
}

pub fn sensor_inventory(thermal: &[ThermalSensor], updated: &str) -> Vec<SensorInfo> {
    thermal
        .iter()
        .map(|sensor| SensorInfo {
            name: sensor.name.clone(),
            category: "thermal".into(),
            unit: "celsius".into(),
            available: sensor.available,
            last_value: sensor.value_celsius,
            last_updated: updated.into(),
        })
        .collect()
}

pub fn evaluate_alerts(battery: &BatteryInfo) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let ts = battery.timestamp.clone();
    if let Some(value) = battery.temperature_celsius {
        let (severity, threshold, message) = if value >= 45.0 {
            (
                "critical",
                45.0,
                "Battery temperature is above the critical threshold",
            )
        } else if value >= 40.0 {
            (
                "warning",
                40.0,
                "Battery temperature is above the warning threshold",
            )
        } else {
            ("", 0.0, "")
        };
        if !severity.is_empty() {
            alerts.push(Alert {
                id: format!("battery-temperature-{}", ts),
                severity: severity.into(),
                metric: "battery_temperature".into(),
                message: message.into(),
                observed_value: value,
                threshold,
                timestamp: ts.clone(),
            });
        }
    }
    if let Some(level) = battery.level.filter(|level| *level <= 20) {
        alerts.push(Alert {
            id: format!("battery-level-{}", ts),
            severity: "warning".into(),
            metric: "battery_level".into(),
            message: "Battery level is at or below the warning threshold".into(),
            observed_value: level as f64,
            threshold: 20.0,
            timestamp: ts,
        });
    }
    alerts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_battery_temperature_from_tenths() {
        assert_eq!(battery_temperature(342), Some(34.2));
    }

    #[test]
    fn converts_battery_temperature_from_millidegrees_when_needed() {
        assert_eq!(battery_temperature(34_200), Some(34.2));
    }

    #[test]
    fn ignores_implausible_battery_temperature() {
        assert_eq!(battery_temperature(400_000), None);
    }

    #[test]
    fn uses_conservative_battery_alert_thresholds() {
        let mut battery = battery_info();
        battery.timestamp = "2026-01-01T00:00:00Z".into();
        battery.temperature_celsius = Some(45.0);
        battery.level = Some(20);
        let alerts = evaluate_alerts(&battery);
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].severity, "critical");
        assert_eq!(alerts[1].severity, "warning");
    }
}
