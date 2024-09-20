//! Terracotta
//! 
//! Boilerplate webserver application based on Axum.
//! 



//		Global configuration

//	Customisations of the standard linting configuration
#![allow(unreachable_pub,                 reason = "Not useful in a binary crate")]
#![allow(clippy::doc_markdown,            reason = "Too many false positives")]
#![allow(clippy::expect_used,             reason = "Acceptable in a binary crate")]
#![allow(clippy::multiple_crate_versions, reason = "Cannot resolve all these")]
#![allow(clippy::unwrap_used,             reason = "Somewhat acceptable in a binary crate")]

//	Lints specifically disabled for unit tests
#![cfg_attr(test, allow(
	non_snake_case,
	clippy::arithmetic_side_effects,
	clippy::cast_lossless,
	clippy::cast_precision_loss,
	clippy::cognitive_complexity,
	clippy::default_numeric_fallback,
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::expect_used,
	clippy::indexing_slicing,
	clippy::let_underscore_must_use,
	clippy::let_underscore_untyped,
	clippy::missing_assert_message,
	clippy::missing_panics_doc,
	clippy::must_use_candidate,
	clippy::panic,
	clippy::print_stdout,
	clippy::too_many_lines,
	clippy::unwrap_in_result,
	clippy::unwrap_used,
	reason = "Not useful in unit tests"
))]



//		Modules

mod config;
mod handlers;
mod routes;
mod state;
mod utility;

/// List of crates used in the library and not in the binary.
mod lib {
	use bytes as _;
	use chrono as _;
	use figment as _;
	use flume as _;
	use glob as _;
	use indexmap as _;
	use itertools as _;
	use mime_guess as _;
	use parking_lot as _;
	use rubedo as _;
	use serde_json as _;
	use thiserror as _;
	use tikv_jemalloc_ctl as _;
	use tokio_util as _;
	use tower_http as _;
	use tower_sessions as _;
	use tracing_appender as _;
	use tracing_subscriber as _;
	use url as _;
	use utoipa_rapidoc as _;
	use utoipa_redoc as _;
	use utoipa_swagger_ui as _;
	use velcro as _;
}

/// List of crates used only in library tests.
#[cfg(test)]
mod lib_tests {
	use assert_json_diff as _;
}



//		Packages

use crate::{
	config::Config,
	routes::{protected, public},
	state::AppState,
	utility::{ApiDoc, User},
};
use std::sync::Arc;
use terracotta::{
	app::{
		create::{app as create_app, server as create_server},
		errors::AppError,
		init::{load_config, setup_logging},
		state::StateProvider,
	},
	stats::worker::start as start_stats_processor,
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
	start_stats_processor(&state).await;
	let app    = create_app::<_, User, User>(&state, protected(), public(), ApiDoc::openapi());
	let server = create_server(app, &state).await?;
	info!("Listening on {}", state.address().expect("Server address not set"));
	server.await.unwrap()
}


