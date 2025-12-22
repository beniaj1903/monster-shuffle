mod handlers;
mod state;

use axum::{routing::{get, post}, Router};
use handlers::{game, battle, management, moves};
use state::{load_pokedex, load_moves, AppState};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Inicializar el subscriber de tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Cargar el pokedex al iniciar
    tracing::info!("📚 Cargando pokedex...");
    let pokedex = load_pokedex()
        .expect("Failed to load pokedex - this is a critical error");
    
    tracing::info!("✅ Pokedex cargado: {} Pokémon en memoria", pokedex.len());

    // Cargar los movimientos al iniciar
    tracing::info!("⚔️ Cargando movimientos...");
    let moves = load_moves()
        .expect("Failed to load moves - this is a critical error");
    
    tracing::info!("✅ Movimientos cargados: {} movimientos en memoria", moves.len());

    // Crear el estado de la aplicación
    let state = AppState::new(pokedex, moves);

    // Configurar CORS
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permite cualquier origen (o usar "http://localhost:5173".parse().unwrap() para específico)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    // Configurar logging de requests
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| {
            tracing::info!("Incoming request");
        })
        .on_response(|_response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
            tracing::info!("Response sent in {:?}", latency);
        });

    // Configurar las rutas
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/pokedex/count", get(pokedex_count))
        .route("/api/game/new", post(game::start_new_game))
        .route("/api/game/choose-starter", post(game::choose_starter))
        .route("/api/game/explore", post(game::explore))
        .route("/api/game/select-encounter", post(game::select_encounter))
        .route("/api/game/:session_id", get(game::get_game_state))
        .route("/api/game/battle/move", post(battle::submit_move))
        .route("/api/game/battle/switch", post(battle::switch_pokemon))
        .route("/api/game/team/reorder", post(management::reorder_team))
        .route("/api/game/pokemon/move-reorder", post(management::reorder_moves))
        .route("/api/game/pokemon/evolve", post(management::evolve_pokemon))
        .route("/api/game/select-loot", post(game::select_loot))
        .route("/api/moves", get(moves::get_all_moves))
        .layer(trace_layer)
        .layer(cors)
        .with_state(state);

    // Configurar dirección y puerto desde variables de entorno
    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    let bind_addr = format!("{}:{}", bind_address, port);

    // Iniciar el servidor
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind address");

    tracing::info!("🚀 Server listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server failed");
}

async fn root() -> &'static str {
    "PokeRandomWeb server is running"
}

/// Endpoint de health check
async fn health() -> &'static str {
    "OK"
}

/// Endpoint que devuelve el número de Pokémon cargados en memoria
async fn pokedex_count(axum::extract::State(state): axum::extract::State<AppState>) -> String {
    format!("{}", state.count())
}

