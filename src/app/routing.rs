//! Extended router functionality for the application.



//		Packages

use axum::{
	Router,
	http::HeaderMap,
};
use bytes::Bytes;
use ::core::time::Duration;
use tower_http::{
	LatencyUnit,
	classify::ServerErrorsFailureClass,
	trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, Span, debug, error};
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
	#[must_use]
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
	#[must_use]
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


