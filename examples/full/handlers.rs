//! Endpoint handlers for the application.



//		Packages

use crate::state::AppState;
use axum::{
	extract::State,
	response::Html,
};
use std::sync::Arc;
use tera::Context as Template;



//		Functions

//		get_index																
/// Shows the index page.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// 
pub async fn get_index(State(state): State<Arc<AppState>>) -> Html<String> {
	let mut template = Template::new();
	template.insert("Title",   &state.config.title);
	template.insert("Content", "Index");
	Html(state.tera.render("index", &template).unwrap())
}


