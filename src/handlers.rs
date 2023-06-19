//		Packages

use crate::{
	ASSETS_DIR,
	CONTENT_DIR,
	utility::*,
};
use axum::{
	body::Body,
	extract::State,
	http::{HeaderValue, StatusCode, Uri, header},
	response::{Html, IntoResponse, Response},
};
use mime_guess::{self};
use std::sync::Arc;
use tera::Context;
use tokio::{
	fs::File,
	io::{AsyncReadExt, BufReader},
};
use tokio_util::io::ReaderStream;



//		Enums

//		AssetContext															
/// The protection contexts for static assets.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AssetContext {
	/// Public files.
	Public,
	/// Protected files.
	Protected,
}



//		Functions

//		get_index																
/// Shows the index page.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// 
pub async fn get_index(State(state): State<Arc<AppState>>) -> Html<String> {
	let mut context = Context::new();
	context.insert("Title",   &state.Config.title);
	context.insert("Content", "Index");
	Html(state.Template.render("index", &context).unwrap())
}

//		get_protected_static_asset												
/// Serves protected static assets.
///
/// # Parameters
///
/// * `state` - The application state.
/// * `uri`   - The URI of the asset.
///
pub async fn get_protected_static_asset(
	State(state): State<Arc<AppState>>,
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
pub async fn get_public_static_asset(
	State(state): State<Arc<AppState>>,
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
async fn get_static_asset(
	state:   Arc<AppState>,
	uri:     Uri,
	context: AssetContext
) -> impl IntoResponse {
	let path       =  uri.path().trim_start_matches('/');
	let mime_type  =  mime_guess::from_path(path).first_or_text_plain();
	let (basedir, local_path, behavior) = match context {
		AssetContext::Public    => (
			&ASSETS_DIR,
			state.Config.local_paths.public_assets.join(path),
			&state.Config.local_loading.public_assets
		),
		AssetContext::Protected => (
			&CONTENT_DIR,
			state.Config.local_paths.protected_assets.join(path),
			&state.Config.local_loading.protected_assets
		),
	};
	let is_local   =  match behavior {
		LoadingBehavior::Deny       => false,
		LoadingBehavior::Supplement => basedir.get_file(path).is_none(),
		LoadingBehavior::Override   => local_path.exists(),
	};
	let file       =  if is_local {
		if local_path.exists() {
			File::open(local_path).await.ok()
		} else {
			None
		}
	} else {
		match basedir.get_file(path) {
			Some(file) => File::open(file.path()).await.ok(),
			None       => None,
		}
	};
	match file {
		None           => Err((StatusCode::NOT_FOUND, "")),
		Some(mut file) => {
			let body   =  if file.metadata().await.unwrap().len() > 1024 * 1000 {
				let reader = BufReader::with_capacity(1024 * 128, file);
				let stream = ReaderStream::with_capacity(reader, 1024 * 256);
				Body::wrap_stream(stream)
			} else {
				let mut contents = vec![];
				file.read_to_end(&mut contents).await.unwrap();
				Body::from(contents)
			};
			Ok(Response::builder()
				.status(StatusCode::OK)
				.header(
					header::CONTENT_TYPE,
					HeaderValue::from_str(mime_type.as_ref()).unwrap(),
				)
				.body(body)
				.unwrap()
			)
		},
	}
}


