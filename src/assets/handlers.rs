#![allow(clippy::exhaustive_structs, reason = "Handlers have auto-generated OpenAPI documentation")]
#![allow(clippy::unused_async,       reason = "Handler functions need to be async")]

//! Endpoint handlers for assets.



//		Packages																										

use crate::app::config::LoadingBehavior;
use super::{
	errors::AssetsError,
	state::StateProvider,
};
use axum::{
	body::Body,
	extract::State,
	http::{HeaderValue, StatusCode, Uri, header},
	response::{IntoResponse, Response},
};
use mime_guess::{self};
use std::sync::Arc;
use tokio::{
	fs::File,
	io::{AsyncReadExt as _, BufReader},
};
use tokio_util::io::ReaderStream;



//		Enums																											

//		AssetContext															
/// The protection contexts for static assets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[expect(clippy::exhaustive_enums, reason = "Exhaustive")]
pub enum AssetContext {
	/// Public files.
	Public,
	
	/// Protected files.
	Protected,
}



//		Functions																										

//		get_protected_static_asset												
/// Serves protected static assets.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `uri`   - The URI of the asset.
/// 
/// # Errors
/// 
/// If the asset is not found, cannot be read, or cannot be served, an error
/// will be returned.
/// 
pub async fn get_protected_static_asset<SP: StateProvider>(
	State(state): State<Arc<SP>>,
	uri:          Uri,
) -> impl IntoResponse {
	get_static_asset(state, uri, AssetContext::Protected).await
}

//		get_public_static_asset													
/// Serves public static assets.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `uri`   - The URI of the asset.
/// 
/// # Errors
/// 
/// If the asset is not found, cannot be read, or cannot be served, an error
/// will be returned.
/// 
pub async fn get_public_static_asset<SP: StateProvider>(
	State(state): State<Arc<SP>>,
	uri:          Uri,
) -> impl IntoResponse {
	get_static_asset(state, uri, AssetContext::Public).await
}

//		get_static_asset														
/// Serves static assets.
/// 
/// # Parameters
/// 
/// * `state`   - The application state.
/// * `uri`     - The URI of the asset.
/// * `context` - The protection context of the asset to serve.
/// 
/// # Errors
/// 
/// If the asset is not found, cannot be read, or cannot be served, an error
/// will be returned.
/// 
async fn get_static_asset<SP: StateProvider>(
	state:   Arc<SP>,
	uri:     Uri,
	context: AssetContext
) -> Result<impl IntoResponse, AssetsError> {
	let path      = uri.path().trim_start_matches('/');
	let mime_type = mime_guess::from_path(path).first_or_text_plain();
	let (basedir, local_path, behavior) = match context {
		AssetContext::Public => (
			state.assets_dir(),
			state.config().public_assets.local_path.join(path),
			&state.config().public_assets.behavior
		),
		AssetContext::Protected => (
			state.content_dir(),
			state.config().protected_assets.local_path.join(path),
			&state.config().protected_assets.behavior
		),
	};
	let is_local = match *behavior {
		LoadingBehavior::Deny       => false,
		LoadingBehavior::Supplement => basedir.get_file(path).is_none(),
		LoadingBehavior::Override   => local_path.exists(),
	};
	let body = if is_local {
		if !local_path.exists() {
			return Err(AssetsError::LocalFileNotFound(local_path));
		}
		let mut file = File::open(&local_path).await
			.map_err(|err| AssetsError::FailedToOpenLocalFile(local_path.clone(), err))?
		;
		let metadata = file.metadata().await
			.map_err(|err| AssetsError::FailedToGetLocalFileMetadata(local_path.clone(), err))?
		;
		let config = &state.config().static_files;
		if metadata.len() > config.stream_threshold.saturating_mul(1_024) as u64 {
			let reader = BufReader::with_capacity(config.read_buffer.saturating_mul(1_024), file);
			let stream = ReaderStream::with_capacity(reader, config.stream_buffer.saturating_mul(1_024));
			Body::from_stream(stream)
		} else {
			let mut contents = vec![];
			let _count       = file.read_to_end(&mut contents).await
				.map_err(|err| AssetsError::FailedToReadLocalFile(local_path, err))?
			;
			Body::from(contents)
		}
	} else {
		Body::from(
			basedir.get_file(path)
				.ok_or_else(|| AssetsError::PackagedFileNotFound(path.to_owned()))?
				.contents()
		)
	};
	Response::builder()
		.status(StatusCode::OK)
		.header(
			header::CONTENT_TYPE,
			HeaderValue::from_str(mime_type.as_ref())
				.map_err(|_err| AssetsError::InvalidMimeTypeHeader(mime_type))?
		)
		.body(body)
		.map_err(AssetsError::FailedToBuildResponseBody)
}


