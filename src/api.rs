use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{RawQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::{
    model::{ErrorDetail, ErrorEnvelope, HealthResponse, Snapshot},
    storage::Storage,
};

#[derive(Clone)]
pub struct AppState {
    pub snapshot: Arc<RwLock<Snapshot>>,
    pub storage: Storage,
    pub started: Instant,
}

#[derive(Debug)]
struct ApiError {
    source: anyhow::Error,
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl From<anyhow::Error> for ApiError {
    fn from(source: anyhow::Error) -> Self {
        Self {
            source,
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR",
            message: "The request could not be completed".into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self.source, "API request failed");
        (
            self.status,
            Json(ErrorEnvelope {
                error: ErrorDetail {
                    code: self.code,
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Serialize)]
struct ThermalResponse {
    sensors: Vec<crate::model::ThermalSensor>,
    timestamp: String,
}

#[derive(Serialize)]
struct SensorsResponse {
    sensors: Vec<crate::model::SensorInfo>,
}

#[derive(Serialize)]
struct AlertsResponse {
    alerts: Vec<crate::model::Alert>,
}

#[derive(Serialize)]
struct BatteryHistoryResponse {
    records: Vec<crate::model::BatteryHistoryRecord>,
}

#[derive(Serialize)]
struct ThermalHistoryResponse {
    records: Vec<crate::model::ThermalHistoryRecord>,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "online",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: state.started.elapsed().as_secs(),
    })
}

async fn system(State(state): State<AppState>) -> Json<crate::model::SystemInfo> {
    let snapshot = state.snapshot.read().await;
    Json(
        snapshot
            .system
            .clone()
            .unwrap_or_else(crate::device::system_info),
    )
}

async fn battery(State(state): State<AppState>) -> Json<crate::model::BatteryInfo> {
    let snapshot = state.snapshot.read().await;
    Json(
        snapshot
            .battery
            .clone()
            .unwrap_or_else(crate::device::battery_info),
    )
}

async fn thermal(State(state): State<AppState>) -> Json<ThermalResponse> {
    let snapshot = state.snapshot.read().await;
    Json(ThermalResponse {
        sensors: snapshot.thermal.clone(),
        timestamp: snapshot
            .collected_at
            .clone()
            .unwrap_or_else(crate::device::timestamp),
    })
}

async fn sensors(State(state): State<AppState>) -> Json<SensorsResponse> {
    Json(SensorsResponse {
        sensors: state.snapshot.read().await.sensors.clone(),
    })
}

async fn alerts(State(state): State<AppState>) -> Json<AlertsResponse> {
    Json(AlertsResponse {
        alerts: state.snapshot.read().await.alerts.clone(),
    })
}

fn history_limit(raw_query: Option<&str>) -> Result<i64, ApiError> {
    let Some(limit) = raw_query
        .into_iter()
        .flat_map(|query| query.split('&'))
        .find_map(|part| part.strip_prefix("limit="))
    else {
        return Ok(60);
    };
    limit
        .parse::<i64>()
        .map(|value| value.clamp(1, 500))
        .map_err(|_| ApiError {
            source: anyhow::anyhow!("invalid history limit"),
            status: StatusCode::BAD_REQUEST,
            code: "INVALID_REQUEST",
            message: "Query parameter 'limit' must be an integer".into(),
        })
}

async fn battery_history(
    State(state): State<AppState>,
    RawQuery(query): RawQuery,
) -> ApiResult<BatteryHistoryResponse> {
    let limit = history_limit(query.as_deref())?;
    let records = state
        .storage
        .battery_history(limit)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(BatteryHistoryResponse { records }))
}

async fn thermal_history(
    State(state): State<AppState>,
    RawQuery(query): RawQuery,
) -> ApiResult<ThermalHistoryResponse> {
    let limit = history_limit(query.as_deref())?;
    let records = state
        .storage
        .thermal_history(limit)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ThermalHistoryResponse { records }))
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/system", get(system))
        .route("/api/v1/battery", get(battery))
        .route("/api/v1/thermal", get(thermal))
        .route("/api/v1/sensors", get(sensors))
        .route("/api/v1/alerts", get(alerts))
        .route("/api/v1/history/battery", get(battery_history))
        .route("/api/v1/history/thermal", get(thermal_history))
        .with_state(state)
}

pub fn app_router(state: AppState, frontend_dir: &str) -> Router {
    api_router(state)
        .fallback_service(ServeDir::new(frontend_dir).append_index_html_on_directories(true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[test]
    fn history_limit_defaults_clamps_and_rejects_invalid_values() {
        assert_eq!(history_limit(None).unwrap(), 60);
        assert_eq!(history_limit(Some("limit=0")).unwrap(), 1);
        assert_eq!(history_limit(Some("limit=900")).unwrap(), 500);
        assert_eq!(
            history_limit(Some("limit=invalid")).unwrap_err().status,
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn health_endpoint_returns_contract_shape() {
        let storage = Storage::open("sqlite::memory:", 30).await.unwrap();
        let app = api_router(AppState {
            snapshot: Arc::new(RwLock::new(Snapshot::default())),
            storage,
            started: Instant::now(),
        });
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "online");
        assert_eq!(json["version"], "0.1.0");
        assert!(json["uptime_seconds"].is_number());
    }
}
