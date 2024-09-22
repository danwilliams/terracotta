//! Routes for the application.



//		Packages

use crate::{
	handlers::get_index,
	state::AppState,
};
use axum::routing::{MethodRouter, get};
use std::sync::Arc;
use terracotta::health::handlers::{get_ping, get_version};



//		Functions

//		routes																	
/// Returns a list of routes.
pub fn routes() -> Vec<(&'static str, MethodRouter<Arc<AppState>>)> {
	vec![
		("/",            get(get_index)),
		("/api/ping",    get(get_ping)),
		("/api/version", get(get_version)),
	]
}


