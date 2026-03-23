use crate::{ecs::World, utils::*, web::views::*};
use axum::{
    Json, Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use std::sync::{Arc, Mutex};

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// the web server
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

/// init and start webserver
pub async fn start_webserver(
    world: Arc<Mutex<World>>,
    channel_web2sim_tx: std::sync::mpsc::Sender<ChannelWeb2SimMessage>,
) {
    let app = Router::new()
        .route("/", get(root))
        .route("/world", get(world_stats))
        .route("/creature/{id}", get(creature_detail))
        .route("/ws", get(ws_handler))
        .with_state((world, channel_web2sim_tx));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("[WEB  ]: Webserver running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// routes
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

/// route "/"
async fn root() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("web/index.html"))
}

/******************************************************************************************************************************************/
/// route "/world"
async fn world_stats(
    State((world, _channel_web2sim_sender)): State<(
        Arc<Mutex<World>>,
        std::sync::mpsc::Sender<ChannelWeb2SimMessage>,
    )>,
) -> Json<WorldStats> {
    let world = world.lock().unwrap();
    // let view = world.to_view();
    // Json(view)

    Json(world.get_stats())
}

/******************************************************************************************************************************************/
/// route "/creature/:id"
async fn creature_detail(
    State((world, _channel_web2sim_sender)): State<(
        Arc<Mutex<World>>,
        std::sync::mpsc::Sender<ChannelWeb2SimMessage>,
    )>,
    axum::extract::Path(id): axum::extract::Path<usize>,
) -> Json<CreatureDetailView> {
    let world = world.lock().unwrap();
    Json(world.get_creature_detail_view(id))
}

/******************************************************************************************************************************************/
/// route "/ws" (THE WEBSOCKET)
async fn ws_handler(
    ws: WebSocketUpgrade,
    State((world, _channel_web2sim_sender)): State<(
        Arc<Mutex<World>>,
        std::sync::mpsc::Sender<ChannelWeb2SimMessage>,
    )>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, world, _channel_web2sim_sender))
}

/******************************************************************************************************************************************/
/// Websocket handler
async fn handle_socket(
    mut socket: WebSocket,
    world: Arc<Mutex<World>>,
    channel_web2sim_tx: std::sync::mpsc::Sender<ChannelWeb2SimMessage>,
) {
    const MSG_CREATURES: u8 = 1;
    const MSG_FOOD: u8 = 2;
    let mut msg_counter: u64 = 0;

    loop {
        tokio::select! {
            msg = socket.recv() => {
                // incoming messages
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ChannelWeb2SimMessage>(&text) {
                            Ok(command) => {
                                if channel_web2sim_tx.send(command).is_err() {
                                    println!("[WEB ]: Simulation thread ended, closing websocket");
                                    break; // if the simulation thread has ended, we break the loop and end the task
                                }
                            },
                            Err(e) => {
                                println!("[WEB ]: Invalid JSON command '{}': {}", text, e);
                            }
                        }
                    },
                    Some(Ok(Message::Binary(_bytes))) => {
                        // we ignore binary messages for now
                    },
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {
                        // we ignore other message types for now
                    },
                    Some(Err(e)) => { 
                        println!("[WEB ]: Websocket error: {}", e);
                        break;
                    },
                    None => break,
                };
            },
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                // every 50ms we send an update to the client
                let mut buffer: Vec<u8>;
        
                if msg_counter & 31_u64 == 0 {
                    let food = {
                        let world = world.lock().unwrap();
                        world.foodmap.clone()
                    };
                    buffer = Vec::with_capacity(1 + food.len());
                    buffer.push(MSG_FOOD);
                    buffer.extend_from_slice(&food);
                    msg_counter = 1; // reset to 1 to avoid overflow and to ensure that the next message is a creatures update
                } else {
                    let creatures = {
                        let world = world.lock().unwrap();
                        world
                            .get_creatures_view()
                            .into_iter()
                            .take(50000)
                            .collect::<Vec<_>>() // limit to 50k creatures for performance
                    };
        
                    // "* 23" because each creature has the following fields:
                    // - u32 `id` --> 4 Bytes
                    // - f32 `x` --> 4 Bytes
                    // - f32 `y` --> 4 Bytes
                    // - f32 `size` --> 4 Bytes
                    // - [u8; 3] `color` --> 3 Bytes
                    // - f32 `orientation` --> 4 Bytes
                    buffer = Vec::with_capacity(1 + creatures.len() * 23);
                    buffer.push(MSG_CREATURES);
                    for (id, c) in creatures.iter().enumerate() {
                        buffer.extend_from_slice(&(id as u32).to_le_bytes());
                        buffer.extend_from_slice(&(c.x as f32).to_le_bytes());
                        buffer.extend_from_slice(&(c.y as f32).to_le_bytes());
                        buffer.extend_from_slice(&(c.size as f32).to_le_bytes());
                        buffer.push(c.color[0]);
                        buffer.push(c.color[1]);
                        buffer.push(c.color[2]);
                        buffer.extend_from_slice(&(c.orientation as f32).to_le_bytes());
                    }
                    msg_counter += 1;
                }
                if socket.send(Message::Binary(buffer.into())).await.is_err() { break; }
            }
        }
    }
}
