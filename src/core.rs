//! Core functionality for the application.



//		Packages

use crate::config::Config;
use axum::{
	Router,
	http::HeaderMap,
};
use bytes::Bytes;
use ::core::time::Duration;
use figment::{
	Figment,
	providers::{Env, Format, Serialized, Toml},
};
use include_dir::Dir;
use std::{
	io::stdout,
	sync::Arc,
};
use tera::Tera;
use tower_http::{
	LatencyUnit,
	classify::ServerErrorsFailureClass,
	trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, Span, debug, error};
use tracing_appender::{self, non_blocking, non_blocking::WorkerGuard, rolling::daily};
use tracing_subscriber::{
	EnvFilter,
	fmt::{layer, writer::MakeWriterExt},
	layer::SubscriberExt,
	registry,
	util::SubscriberInitExt,
};
use utoipa::openapi::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;



//		Traits

//§		RouterExt																
/// Extension methods for the Axum [`Router`].
pub trait RouterExt<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_http_logging													
	/// Adds logging of HTTP requests and responses to the router.
	fn add_http_logging(self) -> Self;
	
	//		add_openapi															
	/// Adds OpenAPI functionality using Utoipa.
	/// 
	/// # Parameters
	/// 
	/// * `prefix`  - The prefix to use for the OpenAPI documentation endpoints,
	///               e.g. `/api-docs`. If this is an empty string, no prefix
	///               will be used.
	/// * `openapi` - The OpenAPI specification to use.
	/// 
	fn add_openapi<P: AsRef<str>>(self, prefix: P, openapi: OpenApi) -> Self;
}

//󰭅		RouterExt																
impl<S> RouterExt<S> for Router<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_http_logging													
	fn add_http_logging(self) -> Self {
		self.layer(TraceLayer::new_for_http()
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
	}
	
	//		add_openapi															
	fn add_openapi<P: AsRef<str>>(self, prefix: P, openapi: OpenApi) -> Self {
		self
			.merge(RapiDoc::new(format!("{}/openapi.json", prefix.as_ref()))
				.path(format!("{}/rapidoc", prefix.as_ref()))
			)
			.merge(Redoc::with_url(format!("{}/redoc", prefix.as_ref()), openapi.clone()))
			.merge(SwaggerUi::new(format!("{}/swagger", prefix.as_ref()))
				.url(format!("{}/openapi.json", prefix.as_ref()), openapi)
			)
	}
}



//		Functions

//		load_config																
/// Loads the application configuration.
#[expect(clippy::expect_used, reason = "Misconfiguration or inability to start, so hard quit")]
pub fn load_config() -> Config {
	Figment::from(Serialized::defaults(Config::default()))
		.merge(Toml::file("Config.toml"))
		.merge(Env::raw())
		.extract()
		.expect("Error loading config")
}

//		setup_logging															
/// Sets up logging for the application.
/// 
/// This function sets up logging to the terminal and to a file in the specified
/// directory.
/// 
/// # Parameters
/// 
/// * `logdir` - The directory to write the log files to.
/// 
pub fn setup_logging<S: AsRef<str>>(logdir: S) -> WorkerGuard {
	let (non_blocking_appender, guard) = non_blocking(
		daily(logdir.as_ref(), "general.log")
	);
	registry()
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| format!("{}=debug,tower_http=debug", env!("CARGO_PKG_NAME")).into()),
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
	guard
}

//		setup_tera																
/// Sets up the Tera template engine.
/// 
/// This function reads all the `.tera.html` files in the specified directory
/// and adds them to the Tera template engine.
/// 
/// # Parameters
/// 
/// * `template_dir` - The directory containing the HTML templates. This is
///                    wrapped inside an [`Arc`] to support reusability across
///                    the application if required.
/// 
#[expect(clippy::expect_used, reason = "Misconfiguration or inability to start, so hard quit")]
pub fn setup_tera(template_dir: &Arc<Dir<'static>>) -> Tera {
	let mut templates = vec![];
	for file in template_dir.find("**/*.tera.html").expect("Failed to read glob pattern") {
		templates.push((
			file.path().file_name().unwrap()
				.to_str().unwrap()
				.strip_suffix(".tera.html").unwrap()
				.to_owned(),
			template_dir.get_file(file.path()).unwrap().contents_utf8().unwrap(),
		));
	}
	let mut tera = Tera::default();
	tera.add_raw_templates(templates).expect("Error parsing templates");
	tera.autoescape_on(vec![".tera.html", ".html"]);
	tera
}


