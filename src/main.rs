mod actions;
mod config_file;

use std::{
    borrow::Cow,
    env,
    fs::{self},
    string::FromUtf8Error,
};

use axum::{Router, extract::Path, response::IntoResponse, routing::get};
use calcard::icalendar::ICalendar;
use reqwest::StatusCode;

use crate::config_file::ConfigFile;

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

    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();

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
    Serde(#[from] toml::de::Error),

    #[error("{0}")]
    Custom(Cow<'static, str>),

    #[error("Invalid char '{0}' in id")]
    InvalidCharInId(char),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

async fn handle_request(Path(id): Path<String>) -> Result<String, Error> {
    // This also blocks suspicious accesses like '../../passwords.txt'
    if let Some(invalid_char) = id
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '-'))
    {
        return Err(Error::InvalidCharInId(invalid_char));
    }

    let file_content = fs::read_to_string(format!("calendars/{}.toml", id))?;

    let config_file: ConfigFile = toml::from_str(&file_content)?;

    let cal = fetch_ics_and_apply_actions(&config_file).await?;

    Ok(cal.to_string())
}

async fn fetch_ics_and_apply_actions(config: &ConfigFile) -> Result<ICalendar, Error> {
    let result = reqwest::get(&config.url).await?;
    let bytes = result.bytes().await?;
    let ical_file = String::from_utf8(bytes.to_vec())?;

    let mut cal = ICalendar::parse(&ical_file).map_err(|entry| {
        Error::Custom(format!("error while parsing ics file in entry {:?}", entry).into())
    })?;

    config.apply(&mut cal);

    Ok(cal)
}
