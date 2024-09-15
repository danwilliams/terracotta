//! Utility functions and types for the application.



//		Packages

use crate::{
	config::Config,
	health::handlers as health,
	stats::handlers  as stats,
	stats::worker::AppStateStats,
};
use axum::http::Uri;
use core::fmt::Display;
use std::collections::HashMap;
use tera::Tera;
use url::form_urlencoded;
use utoipa::OpenApi;



//		Structs

//		AppState																
/// The application state.
/// 
/// This is used to store global state information that is shared between
/// requests.
/// 
pub struct AppState {
	//		Public properties													
	/// The application configuration.
	pub config:   Config,
	
	/// The application statistics.
	pub stats:    AppStateStats,
	
	/// The Tera template engine.
	pub template: Tera,
}

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



//		Functions

//		extract_uri_query_parts													
/// Extracts the query parts from a URI.
/// 
/// Extracts the query parts of a [`Uri`] and returns them as a [`HashMap`].
/// 
/// # Parameters
/// 
/// * `uri` - The URI to extract the query parts from.
/// 
pub fn extract_uri_query_parts(uri: &Uri) -> HashMap<String, String> {
	uri
		.query()
		.map(|v| {
			form_urlencoded::parse(v.as_bytes())
				.into_owned()
				.collect()
		})
		.unwrap_or_default()
}

//		build_uri																
/// Builds a URI from a path and a set of query parameters.
/// 
/// # Parameters
/// 
/// * `path`   - The path to build the URI from.
/// * `params` - The query parameters to add to the URI.
/// 
pub fn build_uri<S, K, V>(path: S, params: &HashMap<K, V>) -> Uri
where
	S: AsRef<str>,
	K: AsRef<str> + Display,
	V: AsRef<str> + Display,
{
	Uri::builder()
		.path_and_query(format!("{}?{}",
			path.as_ref(),
			params
				.iter()
				.map(|(k, v)| format!("{k}={v}"))
				.collect::<Vec<String>>()
				.join("&")
			,
		))
		.build()
		.unwrap()
}


