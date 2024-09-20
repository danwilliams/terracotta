//! Utility functions and types for the application.



//		Packages

use terracotta::health;
use utoipa::OpenApi;



//		Structs

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		health::handlers::get_ping,
		health::handlers::get_version,
	),
	components(
		schemas(
			health::responses::HealthVersionResponse,
		),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	),
)]
pub struct ApiDoc;


