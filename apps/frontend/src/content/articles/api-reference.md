---
title: "API Reference"
description: "Complete reference for the Waremax HTTP and WebSocket API"
pubDate: 2024-02-01
author: "Waremax Team"
tags: ["api", "reference", "integration"]
---

# API Reference

The Waremax API is served by `waremax-api-server`. All endpoints are prefixed with `/api`.

## Base URL

```
https://waremax-api.skelfresearch.com
```

For local development:

```
http://localhost:8080
```

## HTTP Endpoints

### List Presets

```http
GET /api/presets
```

Returns an array of available simulation presets.

### Create Session

```http
POST /api/session
Content-Type: application/json

{
  "preset": "standard",
  "robot_count": 15,
  "order_rate": 60,
  "duration_minutes": 60
}
```

Response:

```json
{
  "session_id": "uuid-string"
}
```

### Get Map

```http
GET /api/session/{session_id}/map
```

Returns the warehouse map (nodes, edges, bounds) for the session.

### Start Simulation

```http
POST /api/session/{session_id}/start
```

### Pause Simulation

```http
POST /api/session/{session_id}/pause
```

### Resume Simulation

```http
POST /api/session/{session_id}/resume
```

### Step Simulation

```http
POST /api/session/{session_id}/step
```

Advances the simulation by one event.

### Set Speed

```http
POST /api/session/{session_id}/speed
Content-Type: application/json

{
  "speed": 2.0
}
```

### Add Robot

```http
POST /api/session/{session_id}/add-robot
Content-Type: application/json

{
  "node_id": 42
}
```

### Delete Session

```http
DELETE /api/session/{session_id}
```

## WebSocket

Connect to receive real-time updates:

```
wss://waremax-api.skelfresearch.com/ws/{session_id}
```

### Message Types

| Type | Description |
|------|-------------|
| `Connected` | WebSocket connection established |
| `StateSync` | Full simulation state snapshot |
| `Tick` | Simulation time advanced |
| `RobotMoved` | Robot changed node |
| `OrderCompleted` | Order finished with cycle time |
| `MetricsUpdate` | Aggregated metrics refreshed |
| `Finished` | Simulation completed |
| `Error` | Simulation or session error |

## CORS

The API server supports configurable CORS origins via the `WAREMAX_CORS_ORIGINS` environment variable.
