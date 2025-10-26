use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{info, error};
use std::sync::Arc;
use crate::monitor::Monitor;
use crate::config::Target;

pub async fn get_targets(
    State(monitor): State<Arc<Monitor>>
) -> impl IntoResponse {
    let targets = monitor.get_targets().await;
    Json(targets)
}

pub async fn add_target(
    State(monitor): State<Arc<Monitor>>,
    Json(input): Json<Target>,
) -> impl IntoResponse {
  let target_name = input.name.clone();
  info!("Received request to create target: {}", input.name);
     match monitor.add_target(input).await {
        Ok(_) => {
            info!("Target '{}' created successfully", target_name);
            (StatusCode::CREATED, Json("Target added successfully".to_string()))
        }
        Err(e) => {
            error!(error = %e, target_name = %target_name, "Failed to create target");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("Error: {}", e)))
        }
    }
}

pub async fn remove_target(
    State(monitor): State<Arc<Monitor>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
  match monitor.remove_target(&name).await {
    Ok(_) => {
      info!("Target '{}' removed successfully", &name);
      (StatusCode::OK, Json("Target removed successfully".to_string()))
    }
    Err(e) => {
      error!(error = %e, target_name = %name, "Failed to delete target");
      (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("Error: {}", e)))
    }
  }
}