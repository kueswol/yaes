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
        .route("/creatures", get(creatures))
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
/// route "/creatures"
async fn creatures(State(world): State<Arc<Mutex<World>>>) -> Json<Vec<CreatureView>> {
    let world = world.lock().unwrap();
    Json(world.creatures_view())
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

        if msg_counter % 1000 == 0 {

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
                world.creatures_view().into_iter().take(50000).collect::<Vec<_>>() // limit to 50k creatures for performance
            };
            buffer = Vec::with_capacity(1 + creatures.len()*8); // "* 8" because each creature has 2 f32 values (x and y) which are 4 bytes each
            buffer.push(MSG_CREATURES);
            for c in creatures {
                buffer.extend_from_slice(&(c.x as f32).to_le_bytes());
                buffer.extend_from_slice(&(c.y as f32).to_le_bytes());
            }
            msg_counter += 1;
        
        }

        if socket.send(Message::Binary(buffer.into())).await.is_err() { break; }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
