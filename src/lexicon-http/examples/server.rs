use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    requests: Arc<RwLock<u64>>,
    lexicon_module: Arc<lexicon_http::HttpModule>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn root(State(state): State<AppState>) -> String {
    lexicon_http::root()
}

async fn hello() -> String {
    lexicon_http::hello()
}

async fn users() -> String {
    lexicon_http::users()
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> String {
    let mut count = state.requests.write().await;
    *count += 1;
    
    format!(
        "{{\"success\":true,\"data\":{{\"id\":{},\"name\":\"{}\",\"email\":\"{}\"}},\"message\":\"User created successfully!\"}}",
        count, payload.name, payload.email
    )
}

async fn stats(State(state): State<AppState>) -> String {
    let count = *state.requests.read().await;
    lexicon_http::stats(count as i32)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        requests: Arc::new(RwLock::new(0)),
        lexicon_module: Arc::new(lexicon_http::HttpModule::new()),
    };
    
    let app = Router::new()
        .route("/", get(root))
        .route("/hello", get(hello))
        .route("/users", get(users))
        .route("/users", post(create_user))
        .route("/stats", get(stats))
        .with_state(state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    
    println!("\n🟣 LexiconLang Web Server");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚀 Server running on http://localhost:3000");
    println!("");
    println!("📋 Available Endpoints:");
    println!("   GET  /        → Welcome message");
    println!("   GET  /hello   → Hello world");
    println!("   GET  /users   → List all users");
    println!("   POST /users   → Create new user");
    println!("   GET  /stats   → Request statistics");
    println!("");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
