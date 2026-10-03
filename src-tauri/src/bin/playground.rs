use std::path::PathBuf;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use gx_expenses_manager_lib::{database, graphql};

const ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::var("GX_DB").unwrap_or_else(|_| "playground.db".into()));
    let password = std::env::var("GX_PASSWORD").unwrap_or_else(|_| "playground".into());

    let pool = database::open(&path, &password).await?;
    let schema = graphql::schema(pool);

    let app = Router::new().route(
        "/",
        get(Html(GraphiQLSource::build().endpoint("/").finish())).post_service(GraphQL::new(schema)),
    );

    println!("GraphiQL: http://{ADDRESS}");
    axum::serve(tokio::net::TcpListener::bind(ADDRESS).await?, app).await?;
    Ok(())
}
