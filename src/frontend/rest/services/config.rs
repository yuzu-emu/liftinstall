//! The /api/config call returns the current installer framework configuration.
//!
//! This endpoint should be usable directly from a <script> tag during loading.

use frontend::rest::services::default_future;
use frontend::rest::services::Future;
use frontend::rest::services::Request;
use frontend::rest::services::Response;
use frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};
use hyper::StatusCode;

use logging::LoggingErrors;

use http;

use config::Config;

pub fn handle(service: &WebService, _req: Request) -> Future {
    let framework_url = {
        service
            .get_framework_read()
            .base_attributes
            .target_url
            .clone()
    };

    info!("Downloading configuration from {:?}...", framework_url);

    default_future(
        match http::download_text(&framework_url).map(|x| Config::from_toml_str(&x)) {
            Ok(Ok(config)) => {
                service.get_framework_write().config = Some(config.clone());

                info!("Configuration file downloaded successfully.");

                let file = service
                    .get_framework_read()
                    .get_config()
                    .log_expect("Config should be loaded by now")
                    .to_json_str()
                    .log_expect("Failed to render JSON representation of config");

                Response::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            Ok(Err(v)) => {
                error!("Bad configuration file: {:?}", v);

                Response::new()
                    .with_status(StatusCode::ServiceUnavailable)
                    .with_header(ContentType::plaintext())
                    .with_body("Bad HTTP response")
            }
            Err(v) => {
                error!(
                    "General connectivity error while downloading config: {:?}",
                    v
                );

                Response::new()
                    .with_status(StatusCode::ServiceUnavailable)
                    .with_header(ContentLength(v.len() as u64))
                    .with_header(ContentType::plaintext())
                    .with_body(v)
            }
        },
    )
}
