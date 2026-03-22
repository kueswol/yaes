use crate::{ecs::World, utils::WorldStats, web::views::*};
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
pub async fn start_webserver(world: Arc<Mutex<World>>) {
    let app = Router::new()
        .route("/", get(root))
        .route("/world", get(world_stats))
        .route("/creature/{id}", get(creature_detail))
        .route("/ws", get(ws_handler))
        .with_state(world);

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
async fn world_stats(State(world): State<Arc<Mutex<World>>>) -> Json<WorldStats> {
    let world = world.lock().unwrap();
    // let view = world.to_view();
    // Json(view)
    
    Json(world.get_stats())
}

/******************************************************************************************************************************************/
/// route "/creature/:id"
async fn creature_detail(
    State(world): State<Arc<Mutex<World>>>,
    axum::extract::Path(id): axum::extract::Path<usize>,
) -> Json<CreatureDetailView> {
    let world = world.lock().unwrap();
    Json(world.get_creature_detail_view(id))
}

/******************************************************************************************************************************************/
/// route "/ws" (THE WEBSOCKET)
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(world): State<Arc<Mutex<World>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, world))
}
async fn handle_socket(mut socket: WebSocket, world: Arc<Mutex<World>>) {
    const MSG_CREATURES  : u8   = 1;
    const MSG_FOOD       : u8   = 2;
    let mut msg_counter  : u64  = 0;
    loop {
        let mut buffer: Vec<u8>;

        if msg_counter & 31_u64 == 0 {

            let food = {
                let world = world.lock().unwrap();
                world.foodmap.clone()
            };
            buffer = Vec::with_capacity(1 + food.len());
            buffer.push(MSG_FOOD);
            buffer.extend_from_slice(&food);
            msg_counter = 1;
        
        } else {

            let creatures = {
                let world = world.lock().unwrap();
                world.get_creatures_view().into_iter().take(50000).collect::<Vec<_>>() // limit to 50k creatures for performance
            };
            
            // "* 19" because each creature has the following fields:
            // - u32 `id` --> 4 Bytes
            // - f32 `x` --> 4 Bytes
            // - f32 `y` --> 4 Bytes
            // - f32 `size` --> 4 Bytes
            // - [u8; 3] `color` --> 3 Bytes
            // - f32 `orientation` --> 4 Bytes
            buffer = Vec::with_capacity(1 + creatures.len() * 23);
            buffer.push(MSG_CREATURES);
            for (id , c) in creatures.iter().enumerate() {
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
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
