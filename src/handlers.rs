//! Endpoint handlers for the application.



//		Packages

use crate::state::AppState;
use axum::{
	extract::State,
	response::Html,
};
use std::sync::Arc;
use tera::Context;



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
	context.insert("Title",   &state.config.title);
	context.insert("Content", "Index");
	Html(state.template.render("index", &context).unwrap())
}


