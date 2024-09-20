//! Contains error types used throughout the module.



//		Packages

use super::utility::InvalidUriParts;
use axum::{
	http::{StatusCode, uri::InvalidUri},
	response::{IntoResponse, Response},
};
use tera::Error as TemplateError;
use thiserror::Error as ThisError;
use tower_sessions::session::Error as SessionError;



//		Enums

//		AuthError																
/// Represents all possible errors that can occur when dealing with
/// authentication.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum AuthError {
	/// Error when handling sessions.
	#[error("Session error: {0}")]
	SessionError(#[from] SessionError),
	
	/// Error when rendering the template.
	#[error("Template error: {0}")]
	TemplateError(#[from] TemplateError),
	
	/// There was a problem parsing the URL.
	#[error("URL error: {0}")]
	UrlError(#[from] InvalidUri),
	
	/// There was a problem constructing a URL from parts.
	#[error("URL error: {0}")]
	UrlPartsError(#[from] InvalidUriParts),
}

//󰭅		IntoResponse															
impl IntoResponse for AuthError {
	//		into_response														
	fn into_response(self) -> Response {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			self.to_string(),
		).into_response()
	}
}


