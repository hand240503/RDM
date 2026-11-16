use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_seconds: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SystemInfo {
    pub device_name: String,
    pub android_version: String,
    pub architecture: String,
    pub uptime_seconds: Option<u64>,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BatteryInfo {
    pub level: Option<i64>,
    pub charging_status: String,
    pub temperature_celsius: Option<f64>,
    pub voltage_mv: Option<i64>,
    pub current_ma: Option<i64>,
    pub health: Option<String>,
    pub technology: Option<String>,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ThermalSensor {
    pub name: String,
    pub sensor_type: String,
    pub value_celsius: Option<f64>,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SensorInfo {
    pub name: String,
    pub category: String,
    pub unit: String,
    pub available: bool,
    pub last_value: Option<f64>,
    pub last_updated: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Alert {
    pub id: String,
    pub severity: String,
    pub metric: String,
    pub message: String,
    pub observed_value: f64,
    pub threshold: f64,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BatteryHistoryRecord {
    pub level: Option<i64>,
    pub temperature_celsius: Option<f64>,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ThermalHistoryRecord {
    pub sensor_name: String,
    pub value_celsius: f64,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Snapshot {
    pub system: Option<SystemInfo>,
    pub battery: Option<BatteryInfo>,
    pub thermal: Vec<ThermalSensor>,
    pub sensors: Vec<SensorInfo>,
    pub alerts: Vec<Alert>,
    pub collected_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorDetail,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
}
