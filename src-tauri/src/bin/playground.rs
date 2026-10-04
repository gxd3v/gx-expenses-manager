use std::path::PathBuf;

use age::secrecy::SecretString;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::response::Html;
use axum::routing::{get, post_service};
use gx_expenses_manager_lib::module::AppContext;
use gx_expenses_manager_lib::{database, graphql};

const ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::var("GX_DB").unwrap_or_else(|_| "playground.db".into()));
    let password = std::env::var("GX_PASSWORD").unwrap_or_else(|_| "playground".into());
    let data_dir = path.parent().map(PathBuf::from).unwrap_or_default();

    let pool = database::open(&path, &password, &data_dir.join("backups")).await?;
    let context = AppContext {
        data_dir,
        password: SecretString::from(password),
    };
    let schema = graphql::schema(pool, context);

    let app = Router::new()
        .route(
            "/",
            get(Html(GraphiQLSource::build().endpoint("/graphql").finish())),
        )
        .route("/graphql", post_service(GraphQL::new(schema)));

    println!("GraphiQL: http://{ADDRESS}");
    axum::serve(tokio::net::TcpListener::bind(ADDRESS).await?, app).await?;
    Ok(())
}
