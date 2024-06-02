use super::get_error_response;
use crate::{configuration::Configuration, proxy::exclusions::LocalExclusionStore};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc::Sender;
use warp::http::StatusCode;

pub async fn get_exclusions(
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to get exclusions: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };

    let exclusions = Vec::from_iter(configuration.exclusions.into_iter()).join("\n");

    Ok(Box::new(warp::reply::json(&exclusions)))
}

pub async fn put_exclusions(
    exclusions: String,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
    local_exclusions_store: LocalExclusionStore,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let _guard = configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home().await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to put exclusions: {err}");
            return Ok(Box::new(get_error_response(err)));
        }
    };

    if let Err(err) = configuration
        .set_exclusions(&exclusions, local_exclusions_store)
        .await
    {
        return Ok(Box::new(get_error_response(err)));
    }

    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(Box::new(StatusCode::ACCEPTED))
}
