use super::get_error_response;
use warp::http::Response;
use warp::Filter as RouteFilter;

use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use warp::filters::BoxedFilter;

use crate::configuration::{self, Ca, Configuration};

async fn get_network_settings() -> Result<Box<dyn warp::Reply>, Infallible> {
    log::debug!("Getting network settings");
    let configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to get network settings: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };
    Ok(Box::new(warp::reply::json(&configuration.network)))
}

async fn put_network_settings(
    network_settings: String,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let _guard = configuration_save_lock.lock().await;
    let mut configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to get network settings: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };
    let network_settings =
        match serde_json::from_str::<configuration::NetworkConfig>(&network_settings) {
            Ok(network_settings) => network_settings,
            Err(err) => {
                log::error!("Failed to parse network settings: {err}");
                return Ok(Box::new(get_error_response(err)));
            }
        };
    if let Err(err) = configuration.set_network_settings(&network_settings).await {
        log::error!("Invalid network settings: {err}");
        return Ok(Box::new(get_error_response(err)));
    };

    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();
    Ok(Box::new(
        Response::builder()
            .status(http::StatusCode::NO_CONTENT)
            .body("".to_string()),
    ))
}

async fn get_ca_certificates() -> Result<Box<dyn warp::Reply>, Infallible> {
    log::debug!("Getting CA certificates");
    let configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };
    let ca_cert = match configuration.ca.get_ca_certificate().await {
        Ok(ca_cert) => ca_cert,
        Err(err) => {
            log::error!("Failed to get CA certificates: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };

    Ok(Box::new(
        Response::builder()
            .header(
                http::header::CONTENT_DISPOSITION,
                "attachment; filename=privaxy-ca-certificate.pem;",
            )
            .header(http::header::CONTENT_TYPE, "application/x-pem-file")
            .body(ca_cert.to_pem().unwrap()),
    ))
}

async fn put_ca_certificates(
    ca_cert_body: String,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let _guard = configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };
    let ca_cert_struct = match serde_json::from_str::<Ca>(&ca_cert_body) {
        Ok(ca_cert_req) => ca_cert_req,
        Err(err) => {
            log::error!("Failed to parse CA certificates: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };

    if let Err(err) = configuration.set_ca_settings(&ca_cert_struct).await {
        log::error!("Failed to set CA certificate: {err}");
        return Ok(Box::new(get_error_response(err)));
    }

    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(Box::new(
        Response::builder()
            .status(http::StatusCode::NO_CONTENT)
            .body("".to_string()),
    ))
}

fn network_settings_route(
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path("network")
        .and(
            warp::path::end()
                .and(warp::get().and_then(self::get_network_settings))
                .or(warp::put()
                    .and(warp::body::json())
                    .and(super::with_configuration_updater_sender(
                        configuration_updater_sender,
                    ))
                    .and(super::with_configuration_save_lock(configuration_save_lock))
                    .and_then(self::put_network_settings)),
        )
        .boxed()
}

pub(crate) fn create_routes(
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> BoxedFilter<(impl warp::Reply,)> {
    let network_settings_route = network_settings_route(
        configuration_updater_sender.clone(),
        configuration_save_lock.clone(),
    );
    network_settings_route
        .or(warp::path("ca-certificate").and(
            warp::path::end().and(
                warp::get()
                    .and_then(self::get_ca_certificates)
                    .or(warp::put()
                        .and(warp::body::json())
                        .and(super::with_configuration_updater_sender(
                            configuration_updater_sender,
                        ))
                        .and(super::with_configuration_save_lock(configuration_save_lock))
                        .and_then(self::put_ca_certificates)),
            ),
        ))
        .boxed()
}
