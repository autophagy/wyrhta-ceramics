mod config;
mod error;
mod handlers;
mod jwt;
mod models;
mod result;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{config::Region, Client};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::net::SocketAddr;
use std::path::Path;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use config::Config;
use handlers::auth::{auth as is_authed, login};
use handlers::clay::clays;
use handlers::event::events;
use handlers::image::upload_image_to_s3;
use handlers::project::{
    delete_project, post_project, project, projects, put_project, works as project_works,
};
use handlers::work::{
    delete_work, events as work_events, post_work, put_state, put_work, work, works,
};
use jwt::auth;

#[derive(Clone)]
pub struct AppState {
    config: Config,
    pool: SqlitePool,
    s3_client: Client,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = &args.get(1).expect("Expected argument for config path");
    let config_path = Path::new(&config_path);
    let config = Config::from_path(config_path);

    let migrate_opts = SqliteConnectOptions::new()
        .filename(&config.db)
        .journal_mode(SqliteJournalMode::Delete)
        .pragma("foreign_keys", "OFF")
        .create_if_missing(true);

    let migrate_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(migrate_opts)
        .await
        .expect("cannot connect to db");

    sqlx::migrate!("db/migrations")
        .run(&migrate_pool)
        .await
        .unwrap();

    let opts = SqliteConnectOptions::new()
        .filename(&config.db)
        .journal_mode(SqliteJournalMode::Delete);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .expect("cannot connect to db");

    let region_provider = RegionProviderChain::first_try(Region::new("eu-central-1"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = Client::new(&shared_config);

    let state = AppState {
        config,
        pool,
        s3_client,
    };

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .pretty()
        .init();

    let public_routes = Router::new()
        .route("/projects", get(projects))
        .route("/projects/:id", get(project))
        .route("/projects/:id/works", get(project_works))
        .route("/events", get(events))
        .route("/works", get(works))
        .route("/works/:id", get(work))
        .route("/works/:id/events", get(work_events))
        .route("/clays", get(clays))
        .route("/login", post(login));

    let protected_routes = Router::new()
        .route("/auth", get(is_authed))
        .route("/projects", post(post_project))
        .route("/projects/:id", put(put_project).delete(delete_project))
        .route("/works", post(post_work))
        .route("/works/:id", put(put_work).delete(delete_work))
        .route("/works/:id/state", put(put_state))
        .route("/upload", post(upload_image_to_s3))
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
