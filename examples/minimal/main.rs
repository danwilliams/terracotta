//! Terracotta: Minimal example
//! 
//! Boilerplate webserver application based on Axum, with minimal functionality
//! enabled: only error-handling and health endpoints (no authentication, no
//! statistics, no asset serving).
//! 



//		Global configuration

//	Customisations of the standard linting configuration
#![allow(unreachable_pub,                 reason = "Not useful in binaries")]
#![allow(unused_crate_dependencies,       reason = "Not relevant to examples")]
#![allow(clippy::doc_markdown,            reason = "Too many false positives")]
#![allow(clippy::expect_used,             reason = "Acceptable in a binary crate")]
#![allow(clippy::multiple_crate_versions, reason = "Cannot resolve all these")]
#![allow(clippy::unwrap_used,             reason = "Somewhat acceptable in a binary crate")]



//		Modules

mod config;
mod handlers;
mod routes;
mod state;
mod utility;



//		Packages

use crate::{
	config::Config,
	routes::routes,
	state::AppState,
	utility::ApiDoc,
};
use std::sync::Arc;
use terracotta::app::{
	create::{app_minimal as create_app, server as create_server},
	errors::AppError,
	init::{load_config, setup_logging},
	state::StateProvider,
};
use tikv_jemallocator::Jemalloc;
use tracing::info;
use utoipa::OpenApi;



//		Constants

/// The global allocator. This is changed to [`Jemalloc`] in order to obtain
/// memory usage statistics.
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;



//		Functions

//		main																	
#[tokio::main]
async fn main() -> Result<(), AppError> {
	let config = load_config::<Config>()?;
	let _guard = setup_logging(&config.logdir);
	let state  = Arc::new(AppState::new(config));
	let app    = create_app(&state, routes(), ApiDoc::openapi());
	let server = create_server(app, &state).await?;
	info!("Listening on {}", state.address().expect("Server address not set"));
	server.await.unwrap()
}


