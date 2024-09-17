//! Utility functions and types for the application.



//		Packages

use crate::{
	health::handlers as health,
	stats::handlers as stats,
};
use utoipa::OpenApi;



//		Structs

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		health::get_ping,
		health::get_version,
		stats::get_stats,
		stats::get_stats_history,
		stats::get_stats_feed,
	),
	components(
		schemas(
			health::HealthVersionResponse,
			stats::StatsResponse,
			stats::StatsResponseForPeriod,
			stats::StatsHistoryResponse,
		),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	),
)]
pub struct ApiDoc;


