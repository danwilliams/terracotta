//! Contains error types used throughout the module.



//		Packages																										

use axum::{
	http::{Error as HttpError, StatusCode},
	response::{IntoResponse, Response},
};
use mime_guess::Mime;
use rubedo::sugar::s;
use std::{
	io::Error as IoError,
	path::PathBuf,
};
use thiserror::Error as ThisError;



//		Enums																											

//		AssetsError																
/// Represents all possible errors that can occur when dealing with assets.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum AssetsError {
	/// Could not build a response body to return the file.
	#[error("Failed to build a response body: {0}")]
	FailedToBuildResponseBody(HttpError),
	
	/// The local file metadata could not be retrieved.
	#[error("Failed to get metadata for local file {0}: {1}")]
	FailedToGetLocalFileMetadata(PathBuf, IoError),
	
	/// The local file could not be opened.
	#[error("Failed to open local file {0}: {1}")]
	FailedToOpenLocalFile(PathBuf, IoError),
	
	/// The local file could not be read.
	#[error("Failed to read local file {0}: {1}")]
	FailedToReadLocalFile(PathBuf, IoError),
	
	/// A valid header could not be constructed from the MIME type. This is
	/// never expected to happen.
	#[error("Invalid MIME type header: {0}")]
	InvalidMimeTypeHeader(Mime),
	
	/// The file could not be found in the local filesystem.
	#[error("Local file not found: {0}")]
	LocalFileNotFound(PathBuf),
	
	/// The file could not be found in the packaged filesystem.
	#[error("Packaged file not found: {0}")]
	PackagedFileNotFound(String),
}

//󰭅		IntoResponse															
impl IntoResponse for AssetsError {
	//		into_response														
	fn into_response(self) -> Response {
		match self {
			Self::LocalFileNotFound(_)             |
			Self::PackagedFileNotFound(_)          => (StatusCode::NOT_FOUND,             s!("")),
			Self::FailedToBuildResponseBody(_)     |
			Self::FailedToGetLocalFileMetadata(..) |
			Self::FailedToOpenLocalFile(..)        |
			Self::FailedToReadLocalFile(..)        |
			Self::InvalidMimeTypeHeader(_)         => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
		}.into_response()
	}
}


