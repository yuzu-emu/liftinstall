
use http::build_async_client;

use hyper::header::{ContentLength, ContentType};
use reqwest::header::{USER_AGENT};
use futures::{Stream, Future};
use jwt::{decode, Validation, Algorithm};

use frontend::rest::services::{WebService, Request, Response, default_future};
use frontend::rest::services::Future as InternalFuture;
use logging::LoggingErrors;
use url::form_urlencoded;
use std::collections::HashMap;
use std::sync::Arc;

/// claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(rename = "releaseChannels", default)]
    channels: Vec<String>,
    #[serde(rename = "IsPatreonAccountLinked")]
    is_linked: bool,
    #[serde(rename = "IsPatreonSubscriptionActive")]
    is_subscribed: bool,
}

fn get_text(future: impl Future<Item = reqwest::async::Response, Error = reqwest::Error>) -> impl Future<Item = String, Error = Response> {
    future.map(|mut response| {
            // Get the body of the response
            match response.status() {
                reqwest::StatusCode::OK =>
                    Ok(response.text()
                        .map_err(|e| {
                            error!("Error while converting the response to text {:?}", e);
                            Response::new()
                                .with_status(hyper::StatusCode::InternalServerError)
                        })),
                _ => {
                    error!("Error wrong response code from server {:?}", response.status());
                    Err(Response::new()
                        .with_status(hyper::StatusCode::InternalServerError))
                }
            }
        })
        .map_err(|err| {
            error!("Error cannot get text on errored stream {:?}", err);
            Response::new()
                .with_status(hyper::StatusCode::InternalServerError)
        })
        .and_then(|x| x)
        .flatten()
}

pub fn handle(service: &WebService, _req: Request) -> InternalFuture {
    let framework = service.framework.read().log_expect("InstallerFramework has been dirtied");
    let credentials = framework.database.credentials.clone();
    let config = framework.config.clone().unwrap();

    // If authentication isn't configured, just return immediately
    if config.authentication.is_none() {
        return default_future(Response::new().with_status(hyper::Ok).with_body("{}"));
    }

    // Create moveable framework references so that the lambdas can write to them later
    let write_cred_fw = Arc::clone(&service.framework);

    Box::new(
        _req.body().concat2().map(move |body| {
            let req = form_urlencoded::parse(body.as_ref())
                .into_owned()
                .collect::<HashMap<String, String>>();

            // Determine which credentials we should use
            let (username, token) = {
                let req_username = req.get("username").unwrap();
                let req_token = req.get("token").unwrap();
                // if the user didn't provide credentials, and theres nothing stored in the database, return an early error
                let req_cred_valid = !req_username.is_empty() && !req_token.is_empty();
                let stored_cred_valid = !credentials.username.is_empty() && !credentials.token.is_empty();
                if !req_cred_valid && !stored_cred_valid {
                    info!("No passed in credential and no stored credentials to validate");
                    return default_future(Response::new().with_status(hyper::BadRequest));
                }
                if req_cred_valid {
                    (req.get("username").unwrap().clone(), req.get("token").unwrap().clone())
                } else {
                    (credentials.username.clone(), credentials.token.clone())
                }
            };

            let authentication = config.authentication.unwrap();

            // Get the public key for this authentication url
            let pub_key = if authentication.pub_key_base64.is_empty() {
                vec![]
            } else {
                match base64::decode(&authentication.pub_key_base64) {
                    Ok(v) => v,
                    Err(err) => {
                        error!("Configured public key was not empty and did not decode as base64 {:?}", err);
                        return default_future(Response::new().with_status(hyper::StatusCode::InternalServerError));
                    },
                }
            };

            // Build the HTTP client up
            let client = match build_async_client() {
                Ok(v) => v,
                Err(_) => {
                    return default_future(Response::new().with_status(hyper::StatusCode::InternalServerError));
                },
            };

            // call the authentication URL to see if we are authenticated
            Box::new(get_text(
                    client.post(&authentication.auth_url)
                    .header(USER_AGENT, "liftinstall (j-selby)")
                    .header("X-USERNAME", username.clone())
                    .header("X-TOKEN", token.clone())
                    .send()
                ).map(move |body| {
                    // Configure validation for audience and issuer if the configuration provides it
                    let validation = match authentication.validation {
                        Some(v) => {
                            let mut valid = Validation::new(Algorithm::RS256);
                            valid.iss = v.iss;
                            if v.aud.is_some() {
                                valid.set_audience(&v.aud.unwrap());
                            }
                            valid
                        }
                        None => Validation::default()
                    };

                    // Verify the JWT token
                    let tok = match decode::<JWTClaims>(&body, pub_key.as_slice(), &validation) {
                        Ok(v) => v,
                        Err(v) => {
                            error!("Error while decoding the JWT. error: {:?} str: {:?}", v, &body);
                            return Err(Response::new().with_status(hyper::StatusCode::InternalServerError));
                        },
                    };

                    {
                        // Store the validated username and password into the installer database
                        let mut framework = write_cred_fw.write().log_expect("InstallerFramework has been dirtied");
                        framework.database.credentials.username = username.clone();
                        framework.database.credentials.token = token.clone();
                        // And store the JWT token temporarily in the
                        framework.authorization_token = Some(body.clone());
                    }

                    // Convert the json to a string and return the json token
                    match serde_json::to_string(&tok.claims) {
                        Ok(v) => Ok(v),
                        Err(e) => {
                            error!("Error while converting the claims to JSON string: {:?}", e);
                            Err(Response::new().with_status(hyper::StatusCode::InternalServerError))
                        }
                    }
                })
                .and_then(|res| res)
                .map(|out| {
                    // Finally return the JSON with the response
                    info!("successfully verified username and token");
                    Response::new()
                        .with_header(ContentLength(out.len() as u64))
                        .with_header(ContentType::json())
                        .with_status(hyper::StatusCode::Ok)
                        .with_body(out)
                })
                .or_else(|err| {
                    // Convert the Err value into an Ok value since the error code from this HTTP request is an Ok(response)
                    Ok(err)
                })
            )
        })
         // Flatten the internal future into the output response future
         .flatten()
    )
}