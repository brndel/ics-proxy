mod actions;
mod config_file;

use std::{borrow::Cow, env, fs::File, io::BufReader, string::FromUtf8Error};

use axum::{Router, extract::Path, response::IntoResponse, routing::get};
use calcard::icalendar::ICalendar;
use reqwest::{IntoUrl, StatusCode};

use crate::{
    actions::CalendarActions,
    config_file::ConfigFile,
};

#[tokio::main]
async fn main() {
    let router = Router::new().route("/{id}", get(handle_request));

    let port = match env::var("PORT") {
        Ok(port) => port.parse::<u16>().unwrap(),
        Err(_) => 9187,
    };


    let host = match env::var("HOST") {
        Ok(host) => host,
        Err(_) => "127.0.0.1".to_string(),
    };

    let listener = tokio::net::TcpListener::bind((host, port))
        .await
        .unwrap();

    println!("listening on port {}", port);

    axum::serve(listener, router).await.unwrap();
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(Cow<'static, str>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

async fn handle_request(Path(id): Path<String>) -> Result<String, Error> {
    println!("requesting thing for {}", id);

    let file = File::open(format!("calendars/{}.json", id))?;

    let config_file: ConfigFile = serde_json::from_reader(BufReader::new(file))?;

    let cal = fetch_ics_and_apply_actions(&config_file.url, &config_file.actions).await?;

    Ok(cal.to_string())
}

async fn fetch_ics_and_apply_actions(
    url: impl IntoUrl,
    ruleset: &CalendarActions,
) -> Result<ICalendar, Error> {
    let result = reqwest::get(url).await?;
    let bytes = result.bytes().await?;
    let ical_file = String::from_utf8(bytes.to_vec())?;

    let mut cal = ICalendar::parse(&ical_file).map_err(|entry| {
        Error::Custom(format!("error while parsing ics file in entry {:?}", entry).into())
    })?;

    ruleset.apply_to_events(&mut cal);

    Ok(cal)
}
