mod auth;
mod http;
use std::time::Duration;

use entity::sea_orm::{ConnectOptions, Database, DatabaseConnection};
use poem::{listener::TcpListener, middleware, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;

use crate::{
    config::SETTINGS,
    repository::{PostgresRepository, Repository},
};

use self::http::APIv1;

#[derive(Clone)]
pub struct AppState<T: Repository + Clone = PostgresRepository> {
    pub repo: T,
}

async fn get_db_conn() -> Result<DatabaseConnection, std::io::Error> {
    let url = &SETTINGS.postgres_url;
    let mut opt = ConnectOptions::new(url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    println!("{}", url);
    let db = Database::connect(opt).await.unwrap();
    Ok(db)
}

pub async fn run() {
    let conn = get_db_conn().await.unwrap();
    let repo = PostgresRepository::new(conn);
    repo.initial_admin().await;
    let state = AppState { repo };
    let api_service = OpenApiService::new(APIv1::default(), "NEOIOT Core", "v1.0")
        .server(format!("http://{}/api", SETTINGS.endpoint.clone()));
    let redoc = api_service.redoc();
    let swagger = api_service.swagger_ui();
    let rapidoc = api_service.rapidoc();

    let api_service = api_service
        .with(middleware::Tracing::default())
        .with(middleware::Cors::default())
        .with(middleware::NormalizePath::new(
            middleware::TrailingSlash::Trim,
        ))
        .with(middleware::Compression::default());
    Server::new(TcpListener::bind(SETTINGS.endpoint.as_str()))
        .run(
            Route::new()
                .nest("/api", api_service)
                .nest("/swagger", swagger)
                .nest("/redoc", redoc)
                .nest("/rapidoc", rapidoc)
                .data(state),
        )
        .await
        .unwrap();
}
