mod account;
mod auth;
mod device;
mod schema;

use poem::{listener::TcpListener, middleware, EndpointExt, Route, Server};
use poem_openapi::{OpenApiService, Tags};

use crate::{
    cache::{Cache, RedisCache},
    config::SETTINGS,
    repository::{PostgresRepository, Repository},
};

use self::{
    account::AccountService, auth::AuthService, device::DeviceService, schema::SchemaService,
};

#[derive(Tags)]
enum ApiTags {
    /// Auth相关API
    Auth,
    /// 账号相关API(需要管理员权限)
    Account,
    /// 设备相关API
    Device,
    /// 数据模型相关API
    Schema,
}
const fn default_page() -> usize {
    1
}
const fn default_page_size() -> usize {
    10
}

#[derive(Clone)]
pub struct AppState<R: Repository = PostgresRepository, C: Cache = RedisCache> {
    pub repo: R,
    pub cache: C,
}

pub async fn run() {
    let repo = PostgresRepository::new(SETTINGS.postgres_url.clone()).await;
    let cache = RedisCache::new(SETTINGS.redis_url.clone()).await;
    repo.initial_admin().await;
    let state = AppState { repo, cache };
    let api_service = OpenApiService::new(
        (AuthService, AccountService, DeviceService, SchemaService),
        "NEOIOT Core",
        "v1.0",
    )
    .server(format!("http://{}/api", SETTINGS.endpoint.clone()));
    let redoc = api_service.redoc();
    let swagger = api_service.swagger_ui();
    let rapidoc = api_service.rapidoc();

    let api_service = api_service
        .with(middleware::Tracing::default())
        .with(middleware::Cors::new())
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
