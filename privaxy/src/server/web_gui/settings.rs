use super::get_error_response;
use warp::http::Response;
use warp::Filter as RouteFilter;

use std::convert::Infallible;
use warp::filters::BoxedFilter;

use crate::configuration::Configuration;

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

async fn get_ca_certificates() -> Result<Box<dyn warp::Reply>, Infallible> {
    log::debug!("Getting CA certificates");
    let configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };
    let ca_cert = match configuration.ca_certificate().await {
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

pub(crate) fn create_routes() -> BoxedFilter<(impl warp::Reply,)> {
    warp::path("network")
        .and(warp::path::end().and(warp::get().and_then(self::get_network_settings)))
        .or(warp::path("ca-certificate")
            .and(warp::path::end().and(warp::get().and_then(self::get_ca_certificates))))
        .boxed()
}
