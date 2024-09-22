//! Utility functions and types for the application.



//		Packages

use crate::state::AppState;
use axum::response::Html;
use std::{
	fs,
	sync::Arc,
};
use tera::Context;
use terracotta::{
	app::{
		config::LoadingBehavior,
		state::StateProvider,
	},
	health,
	stats,
};
use utoipa::OpenApi;



//		Structs

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		health::handlers::get_ping,
		health::handlers::get_version,
		stats::handlers::get_stats,
		stats::handlers::get_stats_history,
		stats::handlers::get_stats_feed,
	),
	components(
		schemas(
			health::responses::HealthVersionResponse,
			stats::requests::MeasurementType,
			stats::responses::StatsResponse,
			stats::responses::StatsResponseForPeriod,
			stats::responses::StatsHistoryResponse,
		),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	),
)]
pub struct ApiDoc;



//		Functions

//		render																	
/// Renders a template.
/// 
/// Renders a template with the given context and returns the result.
/// 
/// If the application has been configured to allow template overrides, the
/// local filesystem will be searched, and any matching templates found will be
/// used in preference to the baked-in ones.
/// 
/// # Parameters
/// 
/// * `state`    - The application state.
/// * `template` - The name of the template to render.
/// * `context`  - The context to render the template with.
/// 
pub fn render(
	state:    &Arc<AppState>,
	template: &str,
	context:  &Context,
) -> Html<String> {
	let local_template = state.html_templates_config().local_path.join(format!("{template}.tera.html"));
	let local_layout   = state.html_templates_config().local_path.join("layout.tera.html");
	let mut tera       = state.template.clone();
	if state.html_templates_config().behavior == LoadingBehavior::Override {
		if local_layout.exists() {
			tera.add_raw_template("layout", &fs::read_to_string(local_layout).ok().unwrap()).unwrap();
		};
		if local_template.exists() {
			tera.add_raw_template(template, &fs::read_to_string(local_template).ok().unwrap()).unwrap();
		};
	};
	Html(tera.render(template, context).unwrap())
}


