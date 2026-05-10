//! WebSocket handler for real-time simulation updates

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

use crate::simulation::SimUpdate;
use crate::state::AppState;
use crate::types::WebSocketMessage;

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, session_id))
}

/// Handle a WebSocket connection with heartbeat support.
///
/// Sends `Ping` frames every 15s and closes the connection if no `Pong`
/// is received within 30s.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, session_id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Get the session
    let session = match state.session_manager.get_session(&session_id).await {
        Some(s) => s,
        None => {
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&WebSocketMessage::Error {
                        message: "Session not found".to_string(),
                    })
                    .unwrap(),
                ))
                .await;
            return;
        }
    };

    // Send connected message
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&WebSocketMessage::Connected {
                session_id: session_id.clone(),
            })
            .unwrap(),
        ))
        .await;

    // Subscribe to simulation updates
    let mut update_rx = {
        let session = session.lock().await;
        session.subscribe()
    };

    // Channel for forwarding command errors from the receiver task to the sender task
    let (error_tx, mut error_rx) = mpsc::unbounded_channel::<String>();

    // Shared last-pong timestamp
    let last_pong = Arc::new(Mutex::new(Instant::now()));
    let last_pong_rx = last_pong.clone();

    // Spawn a task to forward simulation updates to the WebSocket
    let send_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                    if last_pong.lock().await.elapsed() > Duration::from_secs(30) {
                        tracing::warn!(session_id = %session_id, "WebSocket pong timeout");
                        let _ = sender.close().await;
                        break;
                    }
                }
                Ok(update) = update_rx.recv() => {
                    let msg = match update {
                        SimUpdate::StateChanged(state) => WebSocketMessage::StateSync { state },
                        SimUpdate::Tick {
                            time_s,
                            events_processed,
                        } => WebSocketMessage::Tick {
                            time_s,
                            events_processed,
                        },
                        SimUpdate::RobotMoved {
                            robot_id,
                            from_node,
                            to_node,
                            time_s,
                        } => WebSocketMessage::RobotMoved {
                            robot_id,
                            from_node,
                            to_node,
                            time_s,
                        },
                        SimUpdate::RobotStateChanged {
                            robot_id,
                            old_state,
                            new_state,
                            time_s,
                        } => WebSocketMessage::RobotStateChanged {
                            robot_id,
                            old_state,
                            new_state,
                            time_s,
                        },
                        SimUpdate::RobotPositionUpdate {
                            robot_id,
                            edge_id,
                            from_node,
                            to_node,
                            progress,
                            time_s,
                        } => WebSocketMessage::RobotPositionUpdate {
                            robot_id,
                            edge_id,
                            from_node,
                            to_node,
                            progress,
                            time_s,
                        },
                        SimUpdate::OrderCompleted {
                            order_id,
                            cycle_time_s,
                            on_time,
                        } => WebSocketMessage::OrderCompleted {
                            order_id,
                            cycle_time_s,
                            on_time,
                        },
                        SimUpdate::MetricsUpdate(metrics) => WebSocketMessage::MetricsUpdate { metrics },
                        SimUpdate::Finished(final_metrics) => WebSocketMessage::Finished { final_metrics },
                        SimUpdate::Error(message) => WebSocketMessage::Error { message },
                    };

                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(_) => continue,
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Some(err_json) = error_rx.recv() => {
                    if sender.send(Message::Text(err_json)).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }
    });

    // Handle incoming messages (for control commands via WebSocket + pong tracking)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Pong(_)) => {
                *last_pong_rx.lock().await = Instant::now();
            }
            Ok(Message::Text(text)) => {
                // Parse control commands from client
                if let Ok(cmd) = serde_json::from_str::<crate::types::ControlCommand>(&text) {
                    let session = session.lock().await;
                    let sim_cmd = match cmd {
                        crate::types::ControlCommand::Start => {
                            crate::simulation::SimCommand::Resume
                        }
                        crate::types::ControlCommand::Pause => crate::simulation::SimCommand::Pause,
                        crate::types::ControlCommand::Resume => {
                            crate::simulation::SimCommand::Resume
                        }
                        crate::types::ControlCommand::SetSpeed { speed } => {
                            crate::simulation::SimCommand::SetSpeed(speed)
                        }
                        crate::types::ControlCommand::Step => crate::simulation::SimCommand::Step,
                        crate::types::ControlCommand::AddRobot { node_id } => {
                            crate::simulation::SimCommand::AddRobot { node_id }
                        }
                        crate::types::ControlCommand::Reset => {
                            // Reset is handled by creating a new session
                            continue;
                        }
                        crate::types::ControlCommand::Stop => crate::simulation::SimCommand::Stop,
                    };
                    if let Err(e) = session.send_command(sim_cmd).await {
                        let err_msg = serde_json::to_string(&WebSocketMessage::Error {
                            message: format!("Command failed: {}", e),
                        })
                        .unwrap_or_else(|_| r#"{"type":"Error","message":"Command failed"}"#.to_string());
                        let _ = error_tx.send(err_msg);
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    // Cancel the send task when the connection closes
    send_task.abort();
}
