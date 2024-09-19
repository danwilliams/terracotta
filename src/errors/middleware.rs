//! Error-handling middleware.



//		Packages

use super::errors::ErrorsError;
use crate::{
	auth::{
		handlers::get_login,
		middleware::{Context as AuthContext, User as AuthUser},
	},
	state::AppStateProvider,
};
use axum::{
	Extension,
	body::Body,
	extract::State,
	http::{HeaderValue, Request, StatusCode, Uri},
	middleware::Next,
	response::{Html, IntoResponse, Response},
};
use rubedo::http::UnpackedResponseBody;
use std::sync::Arc;
use tera::Context as Template;
use tracing::error;



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
/// If the error is a 404, it returns a 404 status code and a page with a link
/// to the login page.
/// 
/// # Parameters
/// 
/// * `state`   - The application state.
/// * `auth_cx` - The authentication context.
/// * `uri`     - The URI of the request.
/// * `request` - The request.
/// * `next`    - The next middleware.
/// 
pub async fn graceful_error_layer<SP, U>(
	State(state):       State<Arc<SP>>,
	Extension(auth_cx): Extension<AuthContext<U>>,
	uri:                Uri,
	request:            Request<Body>,
	next:               Next,
) -> Result<Response, ErrorsError>
where
	SP: AppStateProvider,
	U:  AuthUser,
{
	let response          = next.run(request).await;
	let (mut parts, body) = response.into_parts();
	Ok(match parts.status {
		//		404: Not Found													
		StatusCode::NOT_FOUND             => {
			drop(parts.headers.remove("content-length"));
			drop(parts.headers.remove("content-type"));
			if parts.headers.contains_key("protected") {
				drop(parts.headers.remove("protected"));
				if auth_cx.current_user.is_none() {
					parts.status = StatusCode::UNAUTHORIZED;
					return Ok((
						parts,
						get_login(State(state), uri).await,
					).into_response());
				}
			}
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
		_                                 => {
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
	request:  Request<Body>,
	next:     Next,
) -> Response {
	let response = next.run(request).await;
	match response.status() {
		StatusCode::INTERNAL_SERVER_ERROR => {
			let (mut parts, body) = response.into_parts();
			if parts.headers.contains_key("error-handled") {
				drop(parts.headers.remove("error-handled"));
				return (parts, body).into_response();
			}
			drop(parts.headers.remove("content-length"));
			drop(parts.headers.remove("content-type"));
			(
				parts,
				Html(r"<h1>Internal server error</h1>"),
			).into_response()
		},
		_                                 => response,
	}
}


