#![allow(clippy::unused_async, reason = "Handler functions need to be async")]

//! Health check endpoints.



//		Modules

#[cfg(test)]
#[path = "../tests/health/handlers.rs"]
mod tests;



//		Packages

use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;



//		Structs

//		HealthVersionResponse													
/// The current version information returned by the `/api/version` endpoint.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, ToSchema)]
pub struct HealthVersionResponse {
	//		Public properties													
	/// The current version of the application.
	pub version: String,
}



//		Functions

//		get_ping																
/// Availability check.
/// 
/// This endpoint is designed for use with uptime monitors. It simply returns
/// a 200 code and no content.
/// 
#[utoipa::path(
	get,
	path = "/api/ping",
	tag  = "health",
	responses(
		(status = 200, description = "Availability check"),
	),
)]
pub async fn get_ping() {}

//		get_version																
/// Current version.
/// 
/// This endpoint returns the current version of the API.
/// 
#[utoipa::path(
	get,
	path = "/api/version",
	tag  = "health",
	responses(
		(status = 200, description = "Current version retrieved successfully"),
	),
)]
pub async fn get_version() -> Json<HealthVersionResponse> {
	Json(HealthVersionResponse {
		version: env!("CARGO_PKG_VERSION").to_owned(),
	})
}


