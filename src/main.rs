use axum::{
    extract::FromRef,
    routing::post,
    Router,
};
use sqlx::PgPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod webhook;

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    webhook_secret: String,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "valocore_relayer=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/valocore".to_string());
    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .unwrap_or_else(|_| "development_secret".to_string());

    let db_pool = PgPool::connect(&database_url).await?;
    tracing::info!("Connected to database");

    let app_state = AppState {
        db_pool: db_pool.clone(),
        webhook_secret,
    };

    let app = Router::new()
        .route("/api/v1/webhooks/github", post(webhook::handle_github_webhook))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
