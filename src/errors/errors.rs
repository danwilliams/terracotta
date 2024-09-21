//! Contains error types used throughout the module.



//		Packages

use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};
use thiserror::Error as ThisError;

#[cfg(feature = "tera")]
use tera::Error as TemplateError;



//		Enums

//		ErrorsError																
/// Represents all possible errors that can occur when dealing with errors.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum ErrorsError {
	/// Error when rendering the template.
	#[cfg(feature = "tera")]
	#[error("Template error: {0}")]
	TemplateError(#[from] TemplateError),
}

//󰭅		IntoResponse															
impl IntoResponse for ErrorsError {
	//		into_response														
	fn into_response(self) -> Response {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			self.to_string(),
		).into_response()
	}
}


