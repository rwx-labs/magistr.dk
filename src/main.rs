use std::net::SocketAddr;

use axum::{
    extract::{Form, Query},
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use serde::Deserialize;
use tokio::signal;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod database;
mod error;
mod fs;
mod templates;

use templates::HtmlTemplate;

pub use error::Error;

#[derive(Deserialize)]
struct QuoteId {
    id: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CreateQuoteRequest {
    #[serde(rename = "tal0")]
    number_1: String,
    #[serde(rename = "tal1")]
    number_2: String,
    #[serde(rename = "inp_dato")]
    date: String,
    #[serde(rename = "inp_tekst")]
    text: String,
    addition: String,
}

async fn quote(Query(params): Query<QuoteId>) -> impl IntoResponse {
    if let Some(id) = params.id {
        let name = format!("{}", id);

        HtmlTemplate(templates::QuoteTemplate { name }).into_response()
    } else {
        HtmlTemplate(templates::BaseTemplate { title: "goddag" }).into_response()
    }
}

async fn new_quote() -> impl IntoResponse {
    HtmlTemplate(templates::NewQuoteTemplate {}).into_response()
}

#[tracing::instrument]
async fn post_quote(Form(quote): Form<CreateQuoteRequest>) -> &'static str {
    let number_1 = quote.number_1.parse::<usize>().unwrap_or(0);
    let number_2 = quote.number_2.parse::<usize>().unwrap_or(0);
    let addition = quote.addition.parse::<usize>().unwrap_or(6080);

    if number_1 + number_2 == addition {
        debug!("adding quote to database");

        "oki"
    } else {
        "ka du ik regne mand"
    }
}

async fn fartscroll<'a>() -> &'a [u8] {
    fs::ASSETS.get("static/fartscroll.js").unwrap()
}

async fn robots<'a>() -> &'a [u8] {
    fs::ASSETS.get("static/robots.txt").unwrap()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "magistr=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let opts = cli::Opts::parse();

    debug!("connecting to database");
    let pool = database::connect(opts.database_url.as_str()).await?;
    debug!("connected to database");

    debug!("running database migrations");
    database::migrate(pool.clone()).await?;
    debug!("database migrations complete");

    let app = Router::new()
        .route("/", get(quote))
        .route("/ny.php", get(new_quote).post(post_quote))
        .route("/static/fartscroll.js", get(fartscroll))
        .route("/robots.txt", get(robots))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}
