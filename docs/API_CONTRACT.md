# RDM - API CONTRACT

Version: 1.0
Project: Rust Device Manager

## 1. Purpose

This document defines the communication contract between:

- Backend: Rust + Axum
- Frontend: HTML5 + CSS3 + Vanilla JavaScript

The Backend is the single source of truth for all hardware monitoring data.

The Frontend must follow this contract and must not invent API endpoints or response fields.

## 2. General Rules

- API base path: `/api/v1`
- Protocol: HTTP
- Data format: JSON
- Character encoding: UTF-8
- Timestamp format: ISO 8601 UTC
- HTTP methods: GET, POST, PUT as required
- Missing hardware readings must be represented by `null`.
- Unavailable sensors must never return fabricated values.

## 3. API Endpoints

### 3.1 Health Check

GET `/api/v1/health`

Purpose:
Check whether the Backend is running.

Example response:

```json
{
  "status": "online",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### 3.2 System Information

GET `/api/v1/system`

Purpose:
Retrieve basic device information.

Response:

```json
{
  "device_name": "OPPO Reno2 F",
  "android_version": "11",
  "architecture": "aarch64",
  "uptime_seconds": 3600,
  "timestamp": "2026-11-16T14:00:00Z"
}
```

Values must come from actual device information where accessible.

- If Android properties cannot be read, `device_name` is `"Unknown device"` and `android_version` is `"Unknown"`.
- `uptime_seconds` is `null` when `/proc/uptime` is unavailable. `architecture` reports the architecture for which the running backend binary was built.
- On Android/Termux, model and release are read from `getprop`; uptime is read from `/proc/uptime`.

### 3.3 Battery Information

GET `/api/v1/battery`

Purpose:
Retrieve battery metrics.

Response:

```json
{
  "level": 87,
  "charging_status": "charging",
  "temperature_celsius": 34.2,
  "voltage_mv": 4280,
  "current_ma": null,
  "health": "good",
  "technology": null,
  "timestamp": "2026-11-16T14:00:00Z"
}
```

Rules:

- `level`: integer from 0 to 100, otherwise null.
- `charging_status`: charging, discharging, full, not_charging or unknown.
- Temperature: Celsius.
- Voltage: millivolts.
- Current: milliamperes.
- Unavailable values: null.
- The Android backend reads battery values from readable `/sys/class/power_supply/*` entries whose `type` is `Battery`; it does not assume every metric exists.
- Linux power-supply `voltage_now` and `current_now` values are converted from micro-units to millivolts and milliamperes. Current may be signed according to the kernel driver.
- Battery temperature follows the standard power-supply tenths-of-a-degree Celsius unit; a millidegree fallback is accepted only when the tenths conversion is implausible.

The example values are illustrative only, not actual hardware readings.

### 3.4 Thermal Information

GET `/api/v1/thermal`

Purpose:
Retrieve discovered thermal sensors.

Response:

```json
{
  "sensors": [
    {
      "name": "battery",
      "sensor_type": "unknown",
      "value_celsius": 34.2,
      "available": true,
      "error": null
    },
    {
      "name": "thermal_zone_1",
      "sensor_type": "unknown",
      "value_celsius": null,
      "available": false,
      "error": "permission_denied"
    }
  ],
  "timestamp": "2026-11-16T14:00:00Z"
}
```

Rules:

- Discover sensors dynamically.
- Never assume sensor indexes identify CPU or GPU.
- Preserve actual sensor names.
- Use null for unavailable measurements.
- Android backend discovers readable zone entries under `/sys/class/thermal/thermal_zone*`, retaining each zone's kernel `type` as its name. Zones whose `temp` file cannot be read are returned unavailable with an error code.
- A readable battery temperature is also exposed as a `battery` thermal sensor if no discovered zone already has that name.
- `sensor_type` remains `unknown` unless the operating system provides a reliable classification; zone indexes are not treated as sensor classifications.

### 3.5 Sensor Inventory

GET `/api/v1/sensors`

Purpose:
Retrieve the sensor inventory.

Response:

```json
{
  "sensors": [
    {
      "name": "battery",
      "category": "thermal",
      "unit": "celsius",
      "available": true,
      "last_value": 34.2,
      "last_updated": "2026-11-16T14:00:00Z"
    }
  ]
}
```

### 3.6 Alerts

GET `/api/v1/alerts`

Purpose:
Retrieve monitoring alerts.

Response:

```json
{
  "alerts": [
    {
      "id": "alert-001",
      "severity": "warning",
      "metric": "battery_temperature",
      "message": "Battery temperature is above warning threshold",
      "observed_value": 41.0,
      "threshold": 40.0,
      "timestamp": "2026-11-16T14:00:00Z"
    }
  ]
}
```

Severity values:

- info
- warning
- critical

### 3.7 Battery History

GET `/api/v1/history/battery?limit=60`

Purpose:
Retrieve battery monitoring history.

Response:

```json
{
  "records": [
    {
      "level": 87,
      "temperature_celsius": 34.2,
      "timestamp": "2026-11-16T14:00:00Z"
    }
  ]
}
```

### 3.8 Thermal History

GET `/api/v1/history/thermal?limit=60`

Purpose:
Retrieve thermal monitoring history.

Response:

```json
{
  "records": [
    {
      "sensor_name": "battery",
      "value_celsius": 34.2,
      "timestamp": "2026-11-16T14:00:00Z"
    }
  ]
}
```

## 4. Error Response

All API errors should use a consistent JSON structure.

Example:

```json
{
  "error": {
    "code": "SENSOR_UNAVAILABLE",
    "message": "Battery sensor is unavailable"
  }
}
```

Use appropriate HTTP status codes:

- 200: Success
- 400: Invalid request
- 404: Endpoint not found
- 500: Internal server error
- 503: Monitoring service unavailable

A single unavailable sensor should not make the entire dashboard API fail.

## 5. Frontend Requirements

- Fetch data from the endpoints defined here.
- Handle null values.
- Handle missing fields.
- Handle network errors.
- Display timestamps.
- Never fabricate readings.
- Avoid overlapping polling requests.
- Do not implement hardware collection in JavaScript.

## 6. Backend Requirements

- Implement the API endpoints defined here.
- Return actual collected hardware data.
- Use consistent JSON field names.
- Keep response structures stable.
- Document any unavoidable deviations.
- Update this file whenever the actual API contract changes.

## 7. Contract Change Policy

Neither FE nor BE may silently change this contract.

Any change must:

1. Update this document.
2. Update the affected implementation.
3. Verify FE/BE compatibility.
4. Record the change in IMPLEMENTATION_STATUS.md.

Important: Response examples are schemas and illustrative values, not production data.
