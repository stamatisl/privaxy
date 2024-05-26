use super::get_error_response;
use crate::configuration::{Configuration, Filter, FilterGroup, calculate_sha256_hex};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc::Sender;
use warp::http::Response;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct FilterStatusChangeRequest {
    enabled: bool,
    file_name: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AddFilterRequest {
    pub enabled: bool,
    pub title: String,
    pub group: FilterGroup,
    #[serde_as(as = "DisplayFromStr")]
    pub url: Url,
}

pub async fn change_filter_status(
    filter_status_change_request: Vec<FilterStatusChangeRequest>,
    http_client: reqwest::Client,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<impl warp::Reply, Infallible> {
    let _guard = configuration_save_lock.lock().await;

    let mut configuration = match Configuration::read_from_home(http_client).await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to change filter status: {err}");
            return Ok(get_error_response(err));
        }
    };

    for filter in filter_status_change_request {
        if let Err(err) = configuration
            .set_filter_enabled_status(&filter.file_name, filter.enabled)
            .await
        {
            log::error!("Failed to change filter status: {err}");
            return Ok(get_error_response(err));
        }
    }

    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(Response::builder()
        .status(http::StatusCode::ACCEPTED)
        .body("".to_string())
        .unwrap())
}

pub async fn get_filters_configuration(
    http_client: reqwest::Client,
) -> Result<impl warp::Reply, Infallible> {
    let configuration = match Configuration::read_from_home(http_client).await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to get filters configuration: {err}");
            return Ok(get_error_response(err));
        }
    };

    let filters = configuration.filters;
    log::debug!("Filters: {:?}", filters);
    Ok(Response::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&filters).unwrap())
        .unwrap())
}

pub async fn add_filter(
    add_filter_request: AddFilterRequest,
    http_client: reqwest::Client,
    configuration_updater_sender: Sender<Configuration>,
    configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<impl warp::Reply, Infallible> {
    let _guard = configuration_save_lock.lock().await;

    // Attempt to retrieve the file from the provided URL
    let filter_content = match http_client.get(&add_filter_request.title).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(content) => content,
                    Err(err) => {
                        log::error!("Failed to read filter content: {err}");
                        return Ok(get_error_response(err));
                    }
                }
            } else {
                log::error!("Failed to fetch filter content: {}", response.status());
                return Ok(Response::builder()
                    .status(http::StatusCode::BAD_REQUEST)
                    .body(format!("Failed to fetch filter content: {}", response.status()))
                    .unwrap());
            }
        },
        Err(err) => {
            log::error!("Failed to fetch filter URL: {err}");
            return Ok(get_error_response(err));
        }
    };

    // Read the current configuration
    let mut configuration = match Configuration::read_from_home(http_client.clone()).await {
        Ok(configuration) => configuration,
        Err(err) => {
            log::error!("Failed to read configuration: {err}");
            return Ok(get_error_response(err));
        }
    };


    // Add the new filter to the configuration
    let new_filter = Filter {
        enabled: add_filter_request.enabled,
        url: add_filter_request.url,
        title: add_filter_request.title.clone(),
        group: add_filter_request.group,
        file_name: calculate_sha256_hex(&add_filter_request.title.clone()), // Generate a unique file name
    };

    // Append the filter content to the custom_filters vector (or write to a file if needed)
    configuration.custom_filters.push(filter_content);

    // Add the new filter to the filters vector
    configuration.filters.push(new_filter);

    // Send the updated configuration to the updater
    configuration_updater_sender
        .send(configuration.clone())
        .await
        .unwrap();

    Ok(Response::builder()
        .status(http::StatusCode::CREATED)
        .body("Filter added successfully".to_string())
        .unwrap())
}