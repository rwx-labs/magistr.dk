//! Template rendering

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tracing::{error, instrument, trace};

use crate::models;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    #[instrument(skip(self), fields(template = std::any::type_name::<T>()))]
    fn into_response(self) -> Response {
        trace!("starting template rendering");

        match self.0.render() {
            Ok(html) => {
                trace!(len = %html.len(), "template rendered successfully");
                Html(html).into_response()
            }
            Err(err) => {
                error!(?err, "template rendering failed: {err}");

                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

#[derive(Template)]
#[template(path = "quote.html", ext = "html")]
pub struct QuoteTemplate {
    pub quote: models::Quote,
}

#[derive(Template)]
#[template(path = "quotes.html", ext = "html")]
pub struct QuotesTemplate {
    pub quotes: Vec<models::Quote>,
}

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate {}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "create_quote.html")]
pub struct NewQuoteTemplate {}

mod utils {
    use chrono::Local;
    use rand::prelude::*;

    /// Returns a random statement from a predefined list.
    pub fn random_statement() -> String {
        const STATEMENTS: &[&str] = &[
            "Treds",
            "Hold kæft!",
            "Goddag.",
            "Grønne svin på et skod beat",
            "dddddd",
            "<robutler> Goddag og farvel.",
            "ja",
            "60",
            ":D",
            "Godt.",
            "100",
        ];

        let mut rng = rand::rng();
        STATEMENTS.choose(&mut rng).unwrap().to_string()
    }

    /// Formats the current date using the provided format string.
    pub fn current_date_formatted(format: &str) -> String {
        Local::now().format(format).to_string()
    }

    /// Generates a random number within the specified range (inclusive)
    pub fn random_number(min: usize, max: usize) -> String {
        let mut rng = rand::rng();
        rng.random_range(min..=max).to_string()
    }

    /// Generates a random hex color using predefined color components.
    pub fn random_hex_color() -> String {
        const COLOR_COMPONENTS: &[&str] = &["CC", "99", "00", "FF", "66", "33"];

        let mut rng = rand::rng();
        let mut color_component = || COLOR_COMPONENTS.choose(&mut rng).unwrap().to_string();

        format!(
            "#{}{}{}",
            color_component(),
            color_component(),
            color_component()
        )
    }

    /// Returns a random CSS font-style property value.
    pub fn random_font_style() -> String {
        const FONT_STYLE: &[&str] = &["oblique", "bold", "italic", "normal"];

        let mut rng = rand::rng();
        FONT_STYLE.choose(&mut rng).unwrap().to_string()
    }
}
