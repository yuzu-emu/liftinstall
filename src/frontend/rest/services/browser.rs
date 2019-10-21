

use frontend::rest::services::{WebService, Request, Response};
use frontend::rest::services::Future as InternalFuture;
use futures::{Stream, Future};
use url::form_urlencoded;
use std::collections::HashMap;
use hyper::header::ContentType;

pub fn handle(_service: &WebService, _req: Request) -> InternalFuture {
    Box::new(
    _req.body().concat2().map(move |body| {
        let req = form_urlencoded::parse(body.as_ref())
            .into_owned()
            .collect::<HashMap<String, String>>();
        if webbrowser::open( req.get("url").unwrap()).is_ok() {
            Response::new()
                .with_status(hyper::Ok)
                .with_header(ContentType::json())
                .with_body("{}")
        } else {
            Response::new()
                .with_status(hyper::BadRequest)
                .with_header(ContentType::json())
                .with_body("{}")
        }
    }))
}

