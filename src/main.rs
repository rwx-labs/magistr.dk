use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{Form, Query, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{
    trace::{BatchConfig, RandomIdGenerator},
    Resource,
};
use opentelemetry_semantic_conventions::{
    resource::{SERVICE_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use tokio::signal;
use tower_http::{
    compression::{CompressionLayer, CompressionLevel},
    trace::TraceLayer,
};
use tracing::{debug, instrument, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod database;
mod error;
mod models;
mod templates;

use templates::HtmlTemplate;

pub use error::Error;

#[derive(Clone)]
struct AppState {
    pub database: database::Database,
}

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

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = Body::from(content.data);
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404"))
                .unwrap(),
        }
    }
}

#[instrument(skip_all)]
async fn quote(Query(params): Query<QuoteId>, State(state): State<AppState>) -> impl IntoResponse {
    let db = state.database;

    if let Some(id) = params.id {
        trace!("fetching single quote");
        let quote = database::get_quote(&db, id as i32).await;
        trace!("fetched single quote");

        if let Some(quote) = quote {
            trace!("quote found");
            (
                StatusCode::OK,
                HtmlTemplate(templates::QuoteTemplate { quote }).into_response(),
            )
        } else {
            trace!("quote not found");
            (
                StatusCode::NOT_FOUND,
                HtmlTemplate(templates::NotFoundTemplate).into_response(),
            )
        }
    } else {
        trace!("fetching all quotes");
        let quotes = database::get_quotes(&db).await;
        trace!("fetched all quotes");

        match quotes {
            Some(quotes) => (
                StatusCode::OK,
                HtmlTemplate(templates::QuotesTemplate { quotes }).into_response(),
            ),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HtmlTemplate(templates::BaseTemplate { title: "error" }).into_response(),
            ),
        }
    }
}

#[instrument]
async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        HtmlTemplate(templates::NotFoundTemplate).into_response(),
    )
}

#[instrument]
async fn new_quote() -> impl IntoResponse {
    HtmlTemplate(templates::NewQuoteTemplate {}).into_response()
}

#[instrument(skip(state))]
async fn post_quote(
    State(state): State<AppState>,
    Form(quote): Form<CreateQuoteRequest>,
) -> impl IntoResponse {
    let number_1 = quote.number_1.parse::<usize>().unwrap_or(0);
    let number_2 = quote.number_2.parse::<usize>().unwrap_or(0);
    let addition = quote.addition.parse::<usize>().unwrap_or(6080);

    if number_1 + number_2 == addition {
        debug!("adding quote to database");
        let db = state.database;

        let result = db
            .create_quote(models::NewQuote {
                date: quote.date,
                text: quote.text,
            })
            .await;

        match result {
            Ok(()) => "oki",
            Err(_) => "pis",
        }
    } else {
        "4kert"
    }
}

#[instrument]
async fn robots<'a>() -> impl IntoResponse {
    StaticFile("robots.txt")
}

#[instrument]
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("static/") {
        path = path.replace("static/", "");
    }

    StaticFile(path)
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
        () = ctrl_c => {},
        () = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource() -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ],
        SCHEMA_URL,
    )
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let opts = cli::Opts::parse();

    // Create a tracing layer with the configured tracer
    let telemetry_layer = if opts.tracing {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(resource()),
            )
            .with_batch_config(BatchConfig::default())
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("could not create otlp pipeline");
        Some(tracing_opentelemetry::layer().with_tracer(tracer))
    } else {
        None
    };

    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "magistr=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(false),
        )
        .with(telemetry_layer)
        .init();

    debug!("connecting to database");
    let db = database::connect(opts.database_url.as_str()).await?;
    debug!("connected to database");

    debug!("running database migrations");
    database::migrate(db.clone()).await?;
    debug!("database migrations complete");

    let app_state = AppState { database: db };
    let app = Router::new()
        .route("/", get(quote))
        .route("/ny.php", post(post_quote).get(new_quote))
        .route("/static/*file", get(static_handler))
        .route("/robots.txt", get(robots))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().quality(CompressionLevel::Fastest))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    debug!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}
