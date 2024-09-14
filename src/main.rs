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

mod config;
mod handlers;
mod middleware;
mod utility;



//		Packages

use crate::{
	config::Config,
	handlers::{
		assets::{get_protected_static_asset, get_public_static_asset},
		auth::{get_logout, post_login},
		health::{get_ping, get_version},
		stats::{get_stats, get_stats_feed, get_stats_history},
		get_index,
	},
	middleware::{
		auth::{auth_layer, protect},
		errors::{final_error_layer, graceful_error_layer, no_route},
		stats::{AppStateStats, AppStats, start_stats_processor, stats_layer},
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
use core::{
	net::SocketAddr,
	time::Duration,
};
use figment::{
	Figment,
	providers::{Env, Format, Serialized, Toml},
};
use flume::{self};
use include_dir::{Dir, include_dir};
use std::{
	io::stdout,
	sync::Arc,
};
use tera::Tera;
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
use tracing_appender::{self, non_blocking, rolling::daily};
use tracing_subscriber::{
	EnvFilter,
	fmt::{layer, writer::MakeWriterExt},
	layer::SubscriberExt,
	registry,
	util::SubscriberInitExt,
};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;



//		Constants

/// The global allocator. This is changed to Jemalloc in order to obtain memory
/// usage statistics.
#[global_allocator]
static GLOBAL:       Jemalloc = Jemalloc;

/// The directory containing the HTML templates.
static TEMPLATE_DIR: Dir<'_>  = include_dir!("html");

/// The directory containing the static assets.
static ASSETS_DIR:   Dir<'_>  = include_dir!("static");

/// The directory containing the Markdown content.
static CONTENT_DIR:  Dir<'_>  = include_dir!("content");



//		Functions

//		main																	
#[expect(clippy::expect_used, reason = "Misconfiguration or inability to start, so hard quit")]
#[tokio::main]
async fn main() {
	let config: Config = Figment::from(Serialized::defaults(Config::default()))
		.merge(Toml::file("Config.toml"))
		.merge(Env::raw())
		.extract()
		.expect("Error loading config")
	;
	let address = SocketAddr::from((config.host, config.port));
	let (non_blocking_appender, _guard) = non_blocking(
		daily(&config.logdir, "general.log")
	);
	registry()
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "terracotta=debug,tower_http=debug".into()),
		)
		.with(
			layer()
				.with_writer(stdout.with_max_level(Level::DEBUG))
		)
		.with(
			layer()
				.with_writer(non_blocking_appender.with_max_level(Level::INFO))
		)
		.init()
	;
	let mut templates = vec![];
	for file in TEMPLATE_DIR.find("**/*.tera.html").expect("Failed to read glob pattern") {
		templates.push((
			file.path().file_name().unwrap()
				.to_str().unwrap()
				.strip_suffix(".tera.html").unwrap()
				.to_owned(),
			TEMPLATE_DIR.get_file(file.path()).unwrap().contents_utf8().unwrap(),
		));
	}
	let mut tera      = Tera::default();
	tera.add_raw_templates(templates).expect("Error parsing templates");
	tera.autoescape_on(vec![".tera.html", ".html"]);
	let (send, recv)  = flume::unbounded();
	let (tx, _rx)     = broadcast::channel(10);
	let session_key   = SessionKey::generate();
	let session_store = SessionMemoryStore::default();
	let shared_state  = Arc::new(AppState {
		config,
		stats:          AppStateStats {
			data:       AppStats {
				started_at: Utc::now().naive_utc(),
				..Default::default()
			},
			queue:      send,
			broadcast:  tx,
		},
		template:       tera,
	});
	if shared_state.config.stats.enabled {
		start_stats_processor(recv, Arc::clone(&shared_state)).await;
	}
	//	Protected routes
	let app           = Router::new()
		.route("/",      get(get_index))
		.route("/*path", get(get_protected_static_asset))
		.route_layer(from_fn_with_state(Arc::clone(&shared_state), protect))
		.merge(
			//	Public routes
			Router::new()
				.route("/api/ping",          get(get_ping))
				.route("/api/version",       get(get_version))
				.route("/api/stats",         get(get_stats))
				.route("/api/stats/history", get(get_stats_history))
				.route("/api/stats/feed",    get(get_stats_feed))
				.route("/login",             post(post_login))
				.route("/logout",            get(get_logout))
				.route("/css/*path",         get(get_public_static_asset))
				.route("/img/*path",         get(get_public_static_asset))
				.route("/js/*path",          get(get_public_static_asset))
				.route("/webfonts/*path",    get(get_public_static_asset))
		)
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


