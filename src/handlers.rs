//		Packages

use crate::{
	ASSETS_DIR,
	CONTENT_DIR,
	utility::*,
};
use axum::{
	body,
	extract::State,
	http::{HeaderValue, StatusCode, Uri, header},
	response::{Html, IntoResponse, Response},
};
use mime_guess::{self};
use std::sync::Arc;
use tera::Context;



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
/// * `uri` - The URI of the asset.
///
pub async fn get_protected_static_asset(uri: Uri) -> impl IntoResponse {
	get_static_asset(uri, AssetContext::Protected).await
}

//		get_public_static_asset													
/// Serves public static assets.
///
/// # Parameters
///
/// * `uri` - The URI of the asset.
///
pub async fn get_public_static_asset(uri: Uri) -> impl IntoResponse {
	get_static_asset(uri, AssetContext::Public).await
}

//		get_static_asset														
/// Serves static assets.
/// 
/// # Parameters
///
/// * `uri`     - The URI of the asset.
/// * `context` - The protection context of the asset to serve.
/// 
async fn get_static_asset(uri: Uri, context: AssetContext) -> impl IntoResponse {
	let path       =  uri.path().trim_start_matches('/');
	let mime_type  =  mime_guess::from_path(path).first_or_text_plain();
	let basedir    =  match context {
		AssetContext::Public    => &ASSETS_DIR,
		AssetContext::Protected => &CONTENT_DIR,
	};
	match basedir.get_file(path) {
		None       => Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body(body::boxed(body::Empty::new()))
			.unwrap()
		,
		Some(file) => Response::builder()
			.status(StatusCode::OK)
			.header(
				header::CONTENT_TYPE,
				HeaderValue::from_str(mime_type.as_ref()).unwrap(),
			)
			.body(body::boxed(body::Full::from(file.contents())))
			.unwrap()
		,
	}
}


