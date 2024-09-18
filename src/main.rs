//! Terracotta
//! 
//! Boilerplate webserver application based on Axum.
//! 



//		Global configuration

//	Customisations of the standard linting configuration
#![allow(unreachable_pub,                 reason = "Not useful in a binary crate")]
#![allow(clippy::doc_markdown,            reason = "Too many false positives")]
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

mod assets;
mod auth;
mod config;
mod core;
mod errors;
mod handlers;
mod health;
mod routes;
mod state;
mod stats;
mod utility;



//		Packages

use crate::{
	core::{RouterExt, load_config, setup_logging, setup_tera},
	auth::routing::RouterExt as AuthRouterExt,
	errors::{
		middleware::no_route,
		routing::RouterExt as ErrorsRouterExt,
	},
	routes::{protected, public},
	stats::{
		routing::RouterExt as StatsRouterExt,
		state::AppStateStats,
		worker::start_stats_processor,
	},
	state::AppState,
	utility::{ApiDoc, User},
};
use axum::Router;
use ::core::net::SocketAddr;
use include_dir::include_dir;
use std::sync::Arc;
use tikv_jemallocator::Jemalloc;
use tokio::{
	net::TcpListener,
	sync::RwLock,
};
use tracing::info;
use utoipa::OpenApi;



//		Constants

/// The global allocator. This is changed to Jemalloc in order to obtain memory
/// usage statistics.
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;



//		Functions

//		main																	
#[expect(clippy::expect_used, reason = "Misconfiguration or inability to start, so hard quit")]
#[tokio::main]
async fn main() {
	let config        = load_config();
	let address       = SocketAddr::from((config.host, config.port));
	let _guard        = setup_logging(&config.logdir);
	let shared_state  = Arc::new(AppState {
		assets_dir:     Arc::new(include_dir!("static")),
		config,
		content_dir:    Arc::new(include_dir!("content")),
		stats:          RwLock::new(AppStateStats::default()),
		template:       setup_tera(&Arc::new(include_dir!("html"))),
	});
	start_stats_processor(&shared_state).await;
	let app           = Router::new()
		.protected_routes::<_, User>(protected(), &shared_state)
		.public_routes(public())
		.add_openapi("/api-docs", ApiDoc::openapi())
		.fallback(no_route)
		.add_error_template::<_, User>(&shared_state)
		.add_authentication::<_, User, User>(&shared_state)
		.add_stats_gathering(&shared_state)
		.with_state(shared_state)
		.add_http_logging()
		.add_error_catcher()
	;
	let listener          = TcpListener::bind(address).await.unwrap();
	let allocated_address = listener.local_addr().expect("Failed to get local address");
	info!("Listening on {allocated_address}");
	axum::serve(listener, app).await.unwrap();
}


