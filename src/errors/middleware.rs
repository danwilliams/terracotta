#![allow(clippy::unused_async, reason = "Middleware functions need to be async")]

//! Error-handling middleware.



//		Packages

use axum::{
	body::Body,
	http::{Request, StatusCode},
	middleware::Next,
	response::{Html, IntoResponse, Response},
};
use rubedo::http::UnpackedResponseBody;
use tracing::error;

#[cfg(feature = "tera")]
use super::errors::ErrorsError;
#[cfg(feature = "tera")]
use crate::app::state::StateProvider as AppStateProvider;
#[cfg(feature = "tera")]
use ::{
	axum::{
		extract::State,
		http::HeaderValue,
	},
	std::sync::Arc,
	tera::Context as Template,
};



//		Functions

//		no_route																
/// Handles non-existent routes.
/// 
/// This function is called as a fallback when a route is not found. It returns
/// a 404 status code.
/// 
pub async fn no_route() -> impl IntoResponse {
	(
		StatusCode::NOT_FOUND,
		[
			("protected", "protected"),
		],
	).into_response()
}

//		graceful_error_layer													
/// Handles errors gracefully.
/// 
/// This function is called when an error occurs. It returns a 500 status code
/// and a page with the error message.
/// 
/// If the error is a 404, it returns a 404 status code and a 404 page.
/// 
/// # Parameters
/// 
/// * `state`   - The application state.
/// * `request` - The request.
/// * `next`    - The next middleware.
/// 
/// # Errors
/// 
/// If there is an error rendering the error page, an error will be returned.
/// 
#[cfg(feature = "tera")]
pub async fn graceful_error_layer<SP>(
	State(state): State<Arc<SP>>,
	request:      Request<Body>,
	next:         Next,
) -> Result<Response, ErrorsError>
where
	SP: AppStateProvider,
{
	let response          = next.run(request).await;
	let (mut parts, body) = response.into_parts();
	Ok(match parts.status {
		//		404: Not Found													
		StatusCode::NOT_FOUND => {
			drop(parts.headers.remove("content-length"));
			drop(parts.headers.remove("content-type"));
			let mut template = Template::new();
			template.insert("Title", &state.title());
			(
				parts,
				Html(state.render("404-notfound", &template)?),
			).into_response()
		},
		//		500: Internal Server Error										
		StatusCode::INTERNAL_SERVER_ERROR => {
			error!("Internal server error: {}", UnpackedResponseBody::from(body));
			let mut template = Template::new();
			template.insert("Title", &state.title());
			drop(parts.headers.remove("content-length"));
			drop(parts.headers.remove("content-type"));
			drop(parts.headers.insert("error-handled", HeaderValue::from_static("gracefully")));
			(
				parts,
				Html(state.render("500-error", &template)?),
			).into_response()
		},
		//		Everything else													
		_ => {
			(
				parts,
				body,
			).into_response()
		},
	})
}

//		final_error_layer														
/// Catches unhandled errors.
/// 
/// This function is called when an error occurs in the
/// [`graceful_error_layer()`] handler. It returns a 500 status code and an
/// error message.
/// 
/// # Parameters
/// 
/// * `request` - The request.
/// * `next`    - The next middleware.
/// 
pub async fn final_error_layer(
	request: Request<Body>,
	next:    Next,
) -> Response {
	let response = next.run(request).await;
	match response.status() {
		//		500: Internal Server Error										
		StatusCode::INTERNAL_SERVER_ERROR => {
			let (mut parts, body) = response.into_parts();
			if parts.headers.contains_key("error-handled") {
				drop(parts.headers.remove("error-handled"));
				return (parts, body).into_response();
			}
			error!("Internal server error: {}", UnpackedResponseBody::from(body));
			drop(parts.headers.remove("content-length"));
			drop(parts.headers.remove("content-type"));
			(
				parts,
				Html(r"<h1>Internal server error</h1>"),
			).into_response()
		},
		//		Everything else													
		_ => response,
	}
}


