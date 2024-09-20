//! Contains error types used in the core application functionality.



//		Packages

use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};
use figment::Error as FigmentError;
use glob::PatternError;
use std::path::PathBuf;
use tera::Error as TemplateError;
use thiserror::Error as ThisError;



//		Enums

//		AppError																
/// Represents all possible errors that can occur in the application.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum AppError {
	/// Error when loading config.
	#[error("Configuration error: {0}")]
	ConfigError(#[from] FigmentError),
	
	/// Error when reading files.
	#[error("Glob pattern error: {0}")]
	GlobError(#[from] PatternError),
	
	/// The template file specified could not be loaded because it is not valid
	/// UTF-8.
	#[error("Invalid template encoding: {0}")]
	InvalidTemplateEncoding(PathBuf),
	
	/// The template path specified could not be loaded because it is invalid.
	#[error("Invalid template path: {0}")]
	InvalidTemplatePath(PathBuf),
	
	/// Error when rendering the template.
	#[error("Template error: {0}")]
	TemplateError(#[from] TemplateError),
	
	/// The template file specified could not be found.
	#[error("Template file not found: {0}")]
	TemplateFileNotFound(PathBuf),
}

//󰭅		IntoResponse															
impl IntoResponse for AppError {
	//		into_response														
	fn into_response(self) -> Response {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			self.to_string(),
		).into_response()
	}
}


