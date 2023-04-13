//! Template rendering

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

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
#[template(path = "quote.html")]
pub struct QuoteTemplate {
    pub name: String,
}

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<'a> {
    pub title: &'a str,
}

mod utils {
    pub fn random_statement() -> String {
        use rand::prelude::*;

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
}
