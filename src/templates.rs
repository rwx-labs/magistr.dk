//! Template rendering

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::models;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
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
pub struct BaseTemplate<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "create_quote.html")]
pub struct NewQuoteTemplate {}

mod utils {
    use chrono::Local;
    use rand::prelude::*;

    pub fn random_statement() -> String {
        const STATEMENTS: &[&str; 11] = &[
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

        let mut rng = rand::thread_rng();

        STATEMENTS.choose(&mut rng).unwrap().to_string()
    }

    pub fn current_date_formatted(s: &str) -> String {
        let dt = Local::now();

        dt.format(s).to_string()
    }

    pub fn random_number(min: usize, max: usize) -> String {
        let mut rng = rand::thread_rng();

        rng.gen_range(min..=max).to_string()
    }

    pub fn random_hex_color() -> String {
        const COLORS: &[&str; 6] = &["CC", "99", "00", "FF", "66", "33"];

        let mut rng = rand::thread_rng();
        let mut c = || COLORS.choose(&mut rng).unwrap().to_string();

        format!("{}{}{}", c(), c(), c())
    }

    pub fn random_font_weight() -> String {
        const WEIGHT: &[&str; 4] = &["oblique", "bold", "italic", "normal"];

        let mut rng = rand::thread_rng();

        WEIGHT.choose(&mut rng).unwrap().to_string()
    }
}
