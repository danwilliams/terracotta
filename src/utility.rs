//! Utility functions and types for the application.



//		Packages

use crate::{
	assets::{
		config::AssetsConfig,
		state::AssetsStateProvider,
	},
	auth::state::AuthStateProvider,
	config::Config,
	health::handlers as health,
	stats::{
		config::StatsConfig,
		handlers as stats,
		state::StatsStateProvider,
		worker::AppStateStats,
	},
};
use axum::http::Uri;
use core::fmt::Display;
use std::collections::HashMap;
use tera::{Context, Error as TemplateError, Tera};
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

//󰭅		AppStateProvider														
impl AppStateProvider for AppState {
	//		render																
	fn render<T: AsRef<str>>(&self, template: T, context: &Context) -> Result<String, TemplateError> {
		self.template.render(template.as_ref(), context)
	}
	
	//		title																
	fn title(&self) -> &String {
		&self.config.title
	}
}

//󰭅		AssetsStateProvider														
impl AssetsStateProvider for AppState {
	//		assets_config														
	fn assets_config(&self) -> &AssetsConfig {
		&self.config.assets
	}
}

//󰭅		AuthStateProvider														
impl AuthStateProvider for AppState {
	//		users																
	fn users(&self) -> &HashMap<String, String> {
		&self.config.users
	}
}

//󰭅		StatsStateProvider														
impl StatsStateProvider for AppState {
	//		stats_config														
	fn stats_config(&self) -> &StatsConfig {
		&self.config.stats
	}
	
	//		stats_state															
	fn stats_state(&self) -> &AppStateStats {
		&self.stats
	}
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



//		Traits

//§		AppStateProvider														
/// A trait for providing the application state for general functionality.
pub trait AppStateProvider: Send + Sync + 'static {
	//		render																
	/// Renders a template.
	/// 
	/// # Parameters
	/// 
	/// * `template` - The template to render.
	/// * `context`  - The context to render the template with.
	/// 
	/// # Errors
	/// 
	/// If the template cannot be rendered, an error is returned.
	/// 
	fn render<T: AsRef<str>>(&self, template: T, context: &Context) -> Result<String, TemplateError>;
	
	//		title																
	/// Gets the application title.
	fn title(&self) -> &String;
}



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


