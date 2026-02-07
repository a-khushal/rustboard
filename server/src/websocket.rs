use crate::session::{ClientInfo, Session};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Join {
        client_id: String,
        name: String,
        color: String,
    },
    Update {
        operation: Operation,
    },
    Presence {
        cursor: Option<Point>,
        selected_ids: Vec<u64>,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Operation {
    AddRectangle {
        id: u64,
        position: Point,
        width: f64,
        height: f64,
    },
    MoveRectangle {
        id: u64,
        position: Point,
    },
    ResizeRectangle {
        id: u64,
        width: f64,
        height: f64,
    },
    DeleteRectangle {
        id: u64,
    },
    AddEllipse {
        id: u64,
        position: Point,
        radius_x: f64,
        radius_y: f64,
    },
    MoveEllipse {
        id: u64,
        position: Point,
    },
    ResizeEllipse {
        id: u64,
        radius_x: f64,
        radius_y: f64,
    },
    DeleteEllipse {
        id: u64,
    },
    AddDiamond {
        id: u64,
        position: Point,
        width: f64,
        height: f64,
    },
    MoveDiamond {
        id: u64,
        position: Point,
    },
    ResizeDiamond {
        id: u64,
        width: f64,
        height: f64,
    },
    DeleteDiamond {
        id: u64,
    },
    AddLine {
        id: u64,
        start: Point,
        end: Point,
    },
    MoveLine {
        id: u64,
        start: Point,
        end: Point,
    },
    DeleteLine {
        id: u64,
    },
    AddArrow {
        id: u64,
        start: Point,
        end: Point,
    },
    MoveArrow {
        id: u64,
        start: Point,
        end: Point,
    },
    DeleteArrow {
        id: u64,
    },
    AddPath {
        id: u64,
        points: Vec<Point>,
    },
    MovePath {
        id: u64,
        offset_x: f64,
        offset_y: f64,
    },
    SetPathPoints {
        id: u64,
        points: Vec<Point>,
    },
    DeletePath {
        id: u64,
    },
    AddImage {
        id: u64,
        position: Point,
        width: f64,
        height: f64,
        image_data: String,
    },
    MoveImage {
        id: u64,
        position: Point,
    },
    ResizeImage {
        id: u64,
        width: f64,
        height: f64,
    },
    DeleteImage {
        id: u64,
    },
    AddText {
        id: u64,
        position: Point,
        width: f64,
        height: f64,
        content: String,
    },
    MoveText {
        id: u64,
        position: Point,
    },
    ResizeText {
        id: u64,
        width: f64,
        height: f64,
    },
    UpdateText {
        id: u64,
        content: String,
    },
    DeleteText {
        id: u64,
    },
    SetRectangleStyle {
        id: u64,
        stroke_color: Option<String>,
        fill_color: Option<Option<String>>,
        line_width: Option<f64>,
        dash_pattern: Option<String>,
        border_radius: Option<f64>,
        rotation_angle: Option<f64>,
    },
    SetEllipseStyle {
        id: u64,
        stroke_color: Option<String>,
        fill_color: Option<Option<String>>,
        line_width: Option<f64>,
        dash_pattern: Option<String>,
        rotation_angle: Option<f64>,
    },
    SetDiamondStyle {
        id: u64,
        stroke_color: Option<String>,
        fill_color: Option<Option<String>>,
        line_width: Option<f64>,
        dash_pattern: Option<String>,
        border_radius: Option<f64>,
        rotation_angle: Option<f64>,
    },
    SetLineStyle {
        id: u64,
        stroke_color: Option<String>,
        line_width: Option<f64>,
        dash_pattern: Option<String>,
    },
    SetArrowStyle {
        id: u64,
        stroke_color: Option<String>,
        line_width: Option<f64>,
        dash_pattern: Option<String>,
    },
    SetPathStyle {
        id: u64,
        stroke_color: Option<String>,
        line_width: Option<f64>,
        dash_pattern: Option<String>,
        rotation_angle: Option<f64>,
    },
    SetImageStyle {
        id: u64,
        rotation_angle: Option<f64>,
    },
    SetTextStyle {
        id: u64,
        color: Option<String>,
        font_size: Option<f64>,
        font_family: Option<String>,
        font_weight: Option<String>,
        text_align: Option<String>,
        rotation_angle: Option<f64>,
    },
    BringToFront {
        id: u64,
    },
    BringForward {
        id: u64,
    },
    SendBackward {
        id: u64,
    },
    SendToBack {
        id: u64,
    },
    SetElementLock {
        id: u64,
        locked: bool,
    },
    FullSync {
        data: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Joined {
        client_id: String,
        clients: Vec<ClientInfo>,
        document: String,
    },
    ClientJoined {
        client: ClientInfo,
    },
    ClientLeft {
        client_id: String,
    },
    Update {
        operation: Operation,
        client_id: String,
        seq: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_local_id: Option<u64>,
    },
    Presence {
        client_id: String,
        cursor: Option<Point>,
        selected_ids: Vec<u64>,
    },
    Error {
        message: String,
    },
    Pong,
}

impl Operation {
    fn id(&self) -> Option<u64> {
        match self {
            Operation::AddRectangle { id, .. }
            | Operation::MoveRectangle { id, .. }
            | Operation::ResizeRectangle { id, .. }
            | Operation::DeleteRectangle { id, .. }
            | Operation::AddEllipse { id, .. }
            | Operation::MoveEllipse { id, .. }
            | Operation::ResizeEllipse { id, .. }
            | Operation::DeleteEllipse { id, .. }
            | Operation::AddDiamond { id, .. }
            | Operation::MoveDiamond { id, .. }
            | Operation::ResizeDiamond { id, .. }
            | Operation::DeleteDiamond { id, .. }
            | Operation::AddLine { id, .. }
            | Operation::MoveLine { id, .. }
            | Operation::DeleteLine { id, .. }
            | Operation::AddArrow { id, .. }
            | Operation::MoveArrow { id, .. }
            | Operation::DeleteArrow { id, .. }
            | Operation::AddPath { id, .. }
            | Operation::MovePath { id, .. }
            | Operation::SetPathPoints { id, .. }
            | Operation::DeletePath { id, .. }
            | Operation::AddImage { id, .. }
            | Operation::MoveImage { id, .. }
            | Operation::ResizeImage { id, .. }
            | Operation::DeleteImage { id, .. }
            | Operation::AddText { id, .. }
            | Operation::MoveText { id, .. }
            | Operation::ResizeText { id, .. }
            | Operation::UpdateText { id, .. }
            | Operation::DeleteText { id, .. }
            | Operation::SetRectangleStyle { id, .. }
            | Operation::SetEllipseStyle { id, .. }
            | Operation::SetDiamondStyle { id, .. }
            | Operation::SetLineStyle { id, .. }
            | Operation::SetArrowStyle { id, .. }
            | Operation::SetPathStyle { id, .. }
            | Operation::SetImageStyle { id, .. }
            | Operation::SetTextStyle { id, .. }
            | Operation::BringToFront { id, .. }
            | Operation::BringForward { id, .. }
            | Operation::SendBackward { id, .. }
            | Operation::SendToBack { id, .. }
            | Operation::SetElementLock { id, .. } => Some(*id),
            Operation::FullSync { .. } => None,
        }
    }

    fn id_mut(&mut self) -> Option<&mut u64> {
        match self {
            Operation::AddRectangle { id, .. }
            | Operation::MoveRectangle { id, .. }
            | Operation::ResizeRectangle { id, .. }
            | Operation::DeleteRectangle { id, .. }
            | Operation::AddEllipse { id, .. }
            | Operation::MoveEllipse { id, .. }
            | Operation::ResizeEllipse { id, .. }
            | Operation::DeleteEllipse { id, .. }
            | Operation::AddDiamond { id, .. }
            | Operation::MoveDiamond { id, .. }
            | Operation::ResizeDiamond { id, .. }
            | Operation::DeleteDiamond { id, .. }
            | Operation::AddLine { id, .. }
            | Operation::MoveLine { id, .. }
            | Operation::DeleteLine { id, .. }
            | Operation::AddArrow { id, .. }
            | Operation::MoveArrow { id, .. }
            | Operation::DeleteArrow { id, .. }
            | Operation::AddPath { id, .. }
            | Operation::MovePath { id, .. }
            | Operation::SetPathPoints { id, .. }
            | Operation::DeletePath { id, .. }
            | Operation::AddImage { id, .. }
            | Operation::MoveImage { id, .. }
            | Operation::ResizeImage { id, .. }
            | Operation::DeleteImage { id, .. }
            | Operation::AddText { id, .. }
            | Operation::MoveText { id, .. }
            | Operation::ResizeText { id, .. }
            | Operation::UpdateText { id, .. }
            | Operation::DeleteText { id, .. }
            | Operation::SetRectangleStyle { id, .. }
            | Operation::SetEllipseStyle { id, .. }
            | Operation::SetDiamondStyle { id, .. }
            | Operation::SetLineStyle { id, .. }
            | Operation::SetArrowStyle { id, .. }
            | Operation::SetPathStyle { id, .. }
            | Operation::SetImageStyle { id, .. }
            | Operation::SetTextStyle { id, .. }
            | Operation::BringToFront { id, .. }
            | Operation::BringForward { id, .. }
            | Operation::SendBackward { id, .. }
            | Operation::SendToBack { id, .. }
            | Operation::SetElementLock { id, .. } => Some(id),
            Operation::FullSync { .. } => None,
        }
    }

    fn is_add_operation(&self) -> bool {
        matches!(
            self,
            Operation::AddRectangle { .. }
                | Operation::AddEllipse { .. }
                | Operation::AddDiamond { .. }
                | Operation::AddLine { .. }
                | Operation::AddArrow { .. }
                | Operation::AddPath { .. }
                | Operation::AddImage { .. }
                | Operation::AddText { .. }
        )
    }
}

pub async fn handle_websocket(
    socket: WebSocket,
    session_id: String,
    state: crate::AppState,
) {
    let (mut sender, mut receiver) = socket.split();

    let session = {
        let sessions = state.sessions.read().unwrap();
        sessions.get_session(&session_id)
    };

    let session = match session {
        Some(s) => s,
        None => {
            error!("Session {} not found", session_id);
            return;
        }
    };

    let mut rx = session.broadcast_tx.subscribe();
    let tx = session.broadcast_tx.clone();
    let receiver_count = tx.receiver_count();
    info!("Client connected to session {}, {} receivers in channel", session_id, receiver_count);

    let session_id_for_log = session_id.clone();
    let session_id_for_send = session_id.clone();
    let client_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let client_id_for_log = client_id.clone();
    
    let session_clone = session.clone();
    let tx_clone = tx.clone();
    
    let (direct_tx, mut direct_rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let mut direct_channel_open = true;

    let mut send_task = tokio::spawn(async move {
        loop {
            if direct_channel_open {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Ok(msg) => {
                                let client_log = {
                                    let client_id_guard = client_id_for_log.lock().unwrap();
                                    client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                                };
                                
                                let json = match serde_json::to_string(&msg) {
                                    Ok(j) => j,
                                    Err(e) => {
                                        error!("Failed to serialize message: {}", e);
                                        continue;
                                    }
                                };
                                
                                if let Err(e) = sender.send(Message::Text(json)).await {
                                    warn!("Failed to send message to client {}: {}", client_log, e);
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                                let client_log = {
                                    let client_id_guard = client_id_for_log.lock().unwrap();
                                    client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                                };
                                warn!("Client {} lagged behind by {} messages, skipping", client_log, skipped);
                                continue;
                            }
                            Err(e) => {
                                let client_log = {
                                    let client_id_guard = client_id_for_log.lock().unwrap();
                                    client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                                };
                                warn!("Error receiving from broadcast channel for client {}: {}", client_log, e);
                                break;
                            }
                        }
                    }
                    msg = direct_rx.recv() => {
                        match msg {
                            Some(msg) => {
                                let client_log = {
                                    let client_id_guard = client_id_for_log.lock().unwrap();
                                    client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                                };
                                
                                let json = match serde_json::to_string(&msg) {
                                    Ok(j) => j,
                                    Err(e) => {
                                        error!("Failed to serialize direct message: {}", e);
                                        continue;
                                    }
                                };
                                
                                if let Err(e) = sender.send(Message::Text(json)).await {
                                    warn!("Failed to send direct message to client {}: {}", client_log, e);
                                    break;
                                }
                            }
                            None => {
                                direct_channel_open = false;
                            }
                        }
                    }
                }
            } else {
                match rx.recv().await {
                    Ok(msg) => {
                        let client_log = {
                            let client_id_guard = client_id_for_log.lock().unwrap();
                            client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                        };
                        
                        let json = match serde_json::to_string(&msg) {
                            Ok(j) => j,
                            Err(e) => {
                                error!("Failed to serialize message: {}", e);
                                continue;
                            }
                        };
                        
                        if let Err(e) = sender.send(Message::Text(json)).await {
                            warn!("Failed to send message to client {}: {}", client_log, e);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        let client_log = {
                            let client_id_guard = client_id_for_log.lock().unwrap();
                            client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                        };
                        warn!("Client {} lagged behind by {} messages, skipping", client_log, skipped);
                        continue;
                    }
                    Err(e) => {
                        let client_log = {
                            let client_id_guard = client_id_for_log.lock().unwrap();
                            client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
                        };
                        warn!("Error receiving from broadcast channel for client {}: {}", client_log, e);
                        break;
                    }
                }
            }
        }
        let client_log = {
            let client_id_guard = client_id_for_log.lock().unwrap();
            client_id_guard.as_ref().map(|id| id.as_str()).unwrap_or("unknown").to_string()
        };
        info!("Send task ended for client {} in session {}", client_log, session_id_for_send);
    });

    let mut recv_task = tokio::spawn(async move {
        let direct_tx = direct_tx;
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Join {
                            client_id: id,
                            name,
                            color,
                        }) => {
                            *client_id.lock().unwrap() = Some(id.clone());
                            session_clone.add_client(id.clone(), name.clone(), color.clone());

                            let document_json = {
                                let doc = session_clone.document.read().unwrap();
                                doc.serialize()
                            };

                            let clients = session_clone.get_clients();
                            let join_msg = ServerMessage::Joined {
                                client_id: id.clone(),
                                clients: clients.clone(),
                                document: document_json,
                            };
                            
                            if let Err(e) = direct_tx.send(join_msg) {
                                error!("Failed to send Joined message directly to client {}: {}", id, e);
                                continue;
                            }

                            let client_joined_msg = ServerMessage::ClientJoined {
                                client: ClientInfo {
                                    id: id.clone(),
                                    name,
                                    color,
                                },
                            };

                            match tx_clone.send(client_joined_msg) {
                                Ok(count) => {
                                    info!("ClientJoined message sent to {} receivers", count);
                                }
                                Err(e) => {
                                    error!("Failed to send ClientJoined message: {}", e);
                                }
                            }
                            info!("Client {} joined session {}", id, session_id_for_log);
                        }
                        Ok(ClientMessage::Update { operation }) => {
                            let id_opt = {
                                let client_id_guard = client_id.lock().unwrap();
                                client_id_guard.clone()
                            };
                            if let Some(id) = id_opt {
                                info!("Received operation from client {}: {:?}", id, operation);
                                let mut canonical_operation = operation.clone();
                                let mut source_local_id = None;

                                if canonical_operation.is_add_operation() {
                                    if let Some(local_id) = canonical_operation.id() {
                                        source_local_id = Some(local_id);
                                    }
                                    if let Some(canonical_id) =
                                        apply_operation(&canonical_operation, &session_clone)
                                    {
                                        if let Some(local_id) = source_local_id {
                                            session_clone.map_client_local_id(&id, local_id, canonical_id);
                                        }
                                        if let Some(op_id) = canonical_operation.id_mut() {
                                            *op_id = canonical_id;
                                        }
                                    }
                                } else {
                                    if let Some(op_id) = canonical_operation.id_mut() {
                                        *op_id = session_clone.resolve_client_id(&id, *op_id);
                                    }
                                    apply_operation(&canonical_operation, &session_clone);
                                }

                                let update_msg = ServerMessage::Update {
                                    operation: canonical_operation.clone(),
                                    client_id: id.clone(),
                                    seq: session_clone.next_operation_seq(),
                                    source_local_id,
                                };
                                
                                let receiver_count = tx_clone.receiver_count();
                                info!("Broadcasting Update from client {} to {} receivers", id, receiver_count);
                                
                                match tx_clone.send(update_msg) {
                                    Ok(sent_count) => {
                                        info!("Update broadcast successfully sent to {} receivers", sent_count);
                                        if sent_count == 0 {
                                            warn!("Update broadcast to 0 receivers!");
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to broadcast Update from client {}: {}", id, e);
                                    }
                                }
                            } else {
                                warn!("Received Update message but client_id is not set - client must join first");
                                if let Err(e) = direct_tx.send(ServerMessage::Error {
                                    message: "Must join session before sending updates".to_string(),
                                }) {
                                    warn!("Failed to send join-first error directly to client: {}", e);
                                }
                            }
                        }
                        Ok(ClientMessage::Presence { cursor, selected_ids }) => {
                            let id_opt = {
                                let client_id_guard = client_id.lock().unwrap();
                                client_id_guard.clone()
                            };
                            if let Some(id) = id_opt {
                                let presence_msg = ServerMessage::Presence {
                                    client_id: id.clone(),
                                    cursor,
                                    selected_ids,
                                };
                                if let Err(e) = tx_clone.send(presence_msg) {
                                    warn!("Failed to broadcast Presence from client {}: {}", id, e);
                                }
                            }
                        }
                        Ok(ClientMessage::Ping) => {
                            if let Err(e) = direct_tx.send(ServerMessage::Pong) {
                                warn!("Failed to send Pong directly to client: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {}", e);
                            if let Err(send_err) = direct_tx.send(ServerMessage::Error {
                                message: format!("Invalid message: {}", e),
                            }) {
                                warn!("Failed to send parse error directly to client: {}", send_err);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }

        let id_opt = {
            let client_id_guard = client_id.lock().unwrap();
            client_id_guard.clone()
        };
        if let Some(id) = id_opt {
            session_clone.remove_client(&id);
            let leave_msg = ServerMessage::ClientLeft {
                client_id: id.clone(),
            };
            let _ = tx_clone.send(leave_msg);
            info!("Client {} left session {}", id, session_id_for_log);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("WebSocket connection closed for session {}", session_id);
}

fn apply_operation(operation: &Operation, session: &Session) -> Option<u64> {
    let mut doc = session.document.write().unwrap();
    use rustboard_editor::geometry::Point as EditorPoint;

    match operation {
        Operation::AddRectangle { position, width, height, .. } => {
            let point = EditorPoint { x: position.x, y: position.y };
            return Some(doc.add_rectangle_without_snapshot(point, *width, *height));
        }
        Operation::MoveRectangle { id, position } => {
            let point = EditorPoint { x: position.x, y: position.y };
            doc.move_rectangle(*id, point, false);
        }
        Operation::ResizeRectangle { id, width, height } => {
            doc.resize_rectangle(*id, *width, *height, false);
        }
        Operation::DeleteRectangle { id } => {
            doc.delete_rectangle_without_snapshot(*id);
        }
        Operation::AddEllipse { position, radius_x, radius_y, .. } => {
            let point = EditorPoint { x: position.x, y: position.y };
            return Some(doc.add_ellipse_without_snapshot(point, *radius_x, *radius_y));
        }
        Operation::MoveEllipse { id, position } => {
            let point = EditorPoint { x: position.x, y: position.y };
            doc.move_ellipse(*id, point, false);
        }
        Operation::ResizeEllipse { id, radius_x, radius_y } => {
            doc.resize_ellipse(*id, *radius_x, *radius_y, false);
        }
        Operation::DeleteEllipse { id } => {
            doc.delete_ellipse_without_snapshot(*id);
        }
        Operation::AddDiamond { position, width, height, .. } => {
            let point = EditorPoint { x: position.x, y: position.y };
            return Some(doc.add_diamond_without_snapshot(point, *width, *height));
        }
        Operation::MoveDiamond { id, position } => {
            let point = EditorPoint { x: position.x, y: position.y };
            doc.move_diamond(*id, point, false);
        }
        Operation::ResizeDiamond { id, width, height } => {
            doc.resize_diamond(*id, *width, *height, false);
        }
        Operation::DeleteDiamond { id } => {
            doc.delete_diamond_without_snapshot(*id);
        }
        Operation::AddLine { start, end, .. } => {
            let start_point = EditorPoint { x: start.x, y: start.y };
            let end_point = EditorPoint { x: end.x, y: end.y };
            return Some(doc.add_line_without_snapshot(start_point, end_point));
        }
        Operation::MoveLine { id, start, end } => {
            let start_point = EditorPoint { x: start.x, y: start.y };
            let end_point = EditorPoint { x: end.x, y: end.y };
            doc.move_line(*id, start_point, end_point, false);
        }
        Operation::DeleteLine { id } => {
            doc.delete_line_without_snapshot(*id);
        }
        Operation::AddArrow { start, end, .. } => {
            let start_point = EditorPoint { x: start.x, y: start.y };
            let end_point = EditorPoint { x: end.x, y: end.y };
            return Some(doc.add_arrow_without_snapshot(start_point, end_point));
        }
        Operation::MoveArrow { id, start, end } => {
            let start_point = EditorPoint { x: start.x, y: start.y };
            let end_point = EditorPoint { x: end.x, y: end.y };
            doc.move_arrow(*id, start_point, end_point, false);
        }
        Operation::DeleteArrow { id } => {
            doc.delete_arrow_without_snapshot(*id);
        }
        Operation::AddPath { points, .. } => {
            let editor_points: Vec<EditorPoint> = points.iter().map(|p| EditorPoint { x: p.x, y: p.y }).collect();
            return Some(doc.add_path_without_snapshot(editor_points));
        }
        Operation::MovePath { id, offset_x, offset_y } => {
            doc.move_path(*id, *offset_x, *offset_y, false);
        }
        Operation::SetPathPoints { id, points } => {
            let editor_points: Vec<EditorPoint> = points.iter().map(|p| EditorPoint { x: p.x, y: p.y }).collect();
            doc.set_path_points(*id, editor_points, false);
        }
        Operation::DeletePath { id } => {
            doc.delete_path_without_snapshot(*id);
        }
        Operation::AddImage { position, width, height, image_data, .. } => {
            let point = EditorPoint { x: position.x, y: position.y };
            return Some(doc.add_image_without_snapshot(point, *width, *height, image_data.clone()));
        }
        Operation::MoveImage { id, position } => {
            let point = EditorPoint { x: position.x, y: position.y };
            doc.move_image(*id, point, false);
        }
        Operation::ResizeImage { id, width, height } => {
            doc.resize_image(*id, *width, *height, false);
        }
        Operation::DeleteImage { id } => {
            doc.delete_image_without_snapshot(*id);
        }
        Operation::AddText { position, width, height, content, .. } => {
            let point = EditorPoint { x: position.x, y: position.y };
            return Some(doc.add_text_without_snapshot(point, *width, *height, content.clone()));
        }
        Operation::MoveText { id, position } => {
            let point = EditorPoint { x: position.x, y: position.y };
            doc.move_text(*id, point, false);
        }
        Operation::ResizeText { id, width, height } => {
            doc.resize_text(*id, *width, *height, false);
        }
        Operation::UpdateText { id, content } => {
            doc.set_text_content(*id, content.clone(), false);
        }
        Operation::DeleteText { id } => {
            doc.delete_text_without_snapshot(*id);
        }
        Operation::SetRectangleStyle { id, stroke_color, fill_color, line_width, dash_pattern, border_radius, rotation_angle } => {
            if let Some(color) = stroke_color {
                doc.set_rectangle_stroke_color(*id, color.clone(), false);
            }
            if let Some(color) = fill_color {
                doc.set_rectangle_fill_color(*id, color.clone(), false);
            }
            if let Some(width) = line_width {
                doc.set_rectangle_line_width(*id, *width, false);
            }
            if let Some(pattern) = dash_pattern {
                doc.set_rectangle_dash_pattern(*id, pattern.clone(), false);
            }
            if let Some(radius) = border_radius {
                doc.set_rectangle_border_radius(*id, *radius, false);
            }
            if let Some(angle) = rotation_angle {
                doc.set_rectangle_rotation(*id, *angle, false);
            }
        }
        Operation::SetEllipseStyle { id, stroke_color, fill_color, line_width, dash_pattern, rotation_angle } => {
            if let Some(color) = stroke_color {
                doc.set_ellipse_stroke_color(*id, color.clone(), false);
            }
            if let Some(color) = fill_color {
                doc.set_ellipse_fill_color(*id, color.clone(), false);
            }
            if let Some(width) = line_width {
                doc.set_ellipse_line_width(*id, *width, false);
            }
            if let Some(pattern) = dash_pattern {
                doc.set_ellipse_dash_pattern(*id, pattern.clone(), false);
            }
            if let Some(angle) = rotation_angle {
                doc.set_ellipse_rotation(*id, *angle, false);
            }
        }
        Operation::SetDiamondStyle { id, stroke_color, fill_color, line_width, dash_pattern, border_radius, rotation_angle } => {
            if let Some(color) = stroke_color {
                doc.set_diamond_stroke_color(*id, color.clone(), false);
            }
            if let Some(color) = fill_color {
                doc.set_diamond_fill_color(*id, color.clone(), false);
            }
            if let Some(width) = line_width {
                doc.set_diamond_line_width(*id, *width, false);
            }
            if let Some(pattern) = dash_pattern {
                doc.set_diamond_dash_pattern(*id, pattern.clone(), false);
            }
            if let Some(radius) = border_radius {
                doc.set_diamond_border_radius(*id, *radius, false);
            }
            if let Some(angle) = rotation_angle {
                doc.set_diamond_rotation(*id, *angle, false);
            }
        }
        Operation::SetLineStyle { id, stroke_color, line_width, dash_pattern } => {
            if let Some(color) = stroke_color {
                doc.set_line_stroke_color(*id, color.clone(), false);
            }
            if let Some(width) = line_width {
                doc.set_line_line_width(*id, *width, false);
            }
            if let Some(pattern) = dash_pattern {
                doc.set_line_dash_pattern(*id, pattern.clone(), false);
            }
        }
        Operation::SetArrowStyle { id, stroke_color, line_width, dash_pattern } => {
            if let Some(color) = stroke_color {
                doc.set_arrow_stroke_color(*id, color.clone(), false);
            }
            if let Some(width) = line_width {
                doc.set_arrow_line_width(*id, *width, false);
            }
            if let Some(pattern) = dash_pattern {
                doc.set_arrow_dash_pattern(*id, pattern.clone(), false);
            }
        }
        Operation::SetPathStyle { id, stroke_color, line_width, dash_pattern, rotation_angle } => {
            if let Some(color) = stroke_color {
                doc.set_path_stroke_color(*id, color.clone(), false);
            }
            if let Some(width) = line_width {
                doc.set_path_line_width(*id, *width, false);
            }
            if let Some(pattern) = dash_pattern {
                doc.set_path_dash_pattern(*id, pattern.clone(), false);
            }
            if let Some(angle) = rotation_angle {
                doc.set_path_rotation(*id, *angle, false);
            }
        }
        Operation::SetImageStyle { id, rotation_angle } => {
            if let Some(angle) = rotation_angle {
                doc.set_image_rotation(*id, *angle, false);
            }
        }
        Operation::SetTextStyle { id, color, font_size, font_family, font_weight, text_align, rotation_angle } => {
            if let Some(c) = color {
                doc.set_text_color(*id, c.clone(), false);
            }
            if let Some(size) = font_size {
                doc.set_text_font_size(*id, *size, false);
            }
            if let Some(family) = font_family {
                doc.set_text_font_family(*id, family.clone(), false);
            }
            if let Some(weight) = font_weight {
                doc.set_text_font_weight(*id, weight.clone(), false);
            }
            if let Some(align) = text_align {
                doc.set_text_text_align(*id, align.clone(), false);
            }
            if let Some(angle) = rotation_angle {
                doc.set_text_rotation(*id, *angle, false);
            }
        }
        Operation::BringToFront { id } => {
            doc.bring_shape_to_front(*id);
        }
        Operation::BringForward { id } => {
            doc.bring_shape_forward(*id);
        }
        Operation::SendBackward { id } => {
            doc.send_shape_backward(*id);
        }
        Operation::SendToBack { id } => {
            doc.send_shape_to_back(*id);
        }
        Operation::SetElementLock { id, locked } => {
            doc.set_element_locked(*id, *locked, false);
        }
        Operation::FullSync { data } => {
            doc.deserialize(data);
        }
    }

    None
}
