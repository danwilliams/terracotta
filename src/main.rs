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
mod stats;
mod utility;



//		Packages

use crate::{
	core::{load_config, setup_logging, setup_tera},
	handlers::get_index,
	assets::handlers::{get_protected_static_asset, get_public_static_asset},
	auth::{
		handlers::{get_logout, post_login},
		middleware::auth_layer,
		routing::RouterExt,
	},
	errors::middleware::{final_error_layer, graceful_error_layer, no_route},
	health::handlers::{get_ping, get_version},
	stats::{
		handlers::{get_stats, get_stats_feed, get_stats_history},
		middleware::stats_layer,
		worker::{AppStateStats, AppStats, start_stats_processor},
	},
	utility::{ApiDoc, AppState},
};
use axum::{
	Router,
	http::HeaderMap,
	middleware::{from_fn, from_fn_with_state},
	routing::{get, post},
};
use bytes::Bytes;
use chrono::Utc;
use ::core::{
	net::SocketAddr,
	time::Duration,
};
use flume::{self};
use include_dir::include_dir;
use std::sync::Arc;
use tikv_jemallocator::Jemalloc;
use tokio::{
	net::TcpListener,
	sync::broadcast,
};
use tower_http::{
	LatencyUnit,
	catch_panic::CatchPanicLayer,
	classify::ServerErrorsFailureClass,
	trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{
	MemoryStore as SessionMemoryStore,
	SessionManagerLayer,
	cookie::Key as SessionKey,
};
use tracing::{Level, Span, info, debug, error};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;



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
	let (send, recv)  = flume::unbounded();
	let (tx, _rx)     = broadcast::channel(10);
	let session_key   = SessionKey::generate();
	let session_store = SessionMemoryStore::default();
	let shared_state  = Arc::new(AppState {
		assets_dir:     Arc::new(include_dir!("static")),
		config,
		content_dir:    Arc::new(include_dir!("content")),
		stats:          AppStateStats {
			data:       AppStats {
				started_at: Utc::now().naive_utc(),
				..Default::default()
			},
			queue:      send,
			broadcast:  tx,
		},
		template:       setup_tera(&Arc::new(include_dir!("html"))),
	});
	if shared_state.config.stats.enabled {
		start_stats_processor(recv, Arc::clone(&shared_state)).await;
	}
	let app           = Router::new()
		.protected_routes(vec![
			("/",      get(get_index)),
			("/*path", get(get_protected_static_asset)),
		], Arc::clone(&shared_state))
		.public_routes(vec![
			("/api/ping",          get(get_ping)),
			("/api/version",       get(get_version)),
			("/api/stats",         get(get_stats)),
			("/api/stats/history", get(get_stats_history)),
			("/api/stats/feed",    get(get_stats_feed)),
			("/login",             post(post_login)),
			("/logout",            get(get_logout)),
			("/css/*path",         get(get_public_static_asset)),
			("/img/*path",         get(get_public_static_asset)),
			("/js/*path",          get(get_public_static_asset)),
			("/webfonts/*path",    get(get_public_static_asset)),
		])
		.merge(SwaggerUi::new("/api-docs/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
		.merge(Redoc::with_url("/api-docs/redoc", ApiDoc::openapi()))
		.merge(RapiDoc::new("/api-docs/openapi.json").path("/api-docs/rapidoc"))
		.fallback(no_route)
		.layer(CatchPanicLayer::new())
		.layer(from_fn_with_state(Arc::clone(&shared_state), graceful_error_layer))
		.layer(from_fn_with_state(Arc::clone(&shared_state), auth_layer))
		.layer(SessionManagerLayer::new(session_store).with_secure(false).with_signed(session_key))
		.layer(from_fn_with_state(Arc::clone(&shared_state), stats_layer))
		.with_state(shared_state)
		.layer(TraceLayer::new_for_http()
			.on_request(
				DefaultOnRequest::new()
					.level(Level::INFO)
			)
			.on_response(
				DefaultOnResponse::new()
					.level(Level::INFO)
					.latency_unit(LatencyUnit::Micros)
			)
			.on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
				debug!("Sending {} bytes", chunk.len());
			})
			.on_eos(|_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
				debug!("Stream closed after {:?}", stream_duration);
			})
			.on_failure(|_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
				error!("Something went wrong");
			})
		)
		.layer(CatchPanicLayer::new())
		.layer(from_fn(final_error_layer))
	;
	let listener          = TcpListener::bind(address).await.unwrap();
	let allocated_address = listener.local_addr().expect("Failed to get local address");
	info!("Listening on {allocated_address}");
	axum::serve(listener, app).await.unwrap();
}


