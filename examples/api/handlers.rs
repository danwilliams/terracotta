//! Endpoint handlers for the application.



//		Packages

use crate::state::AppState;
use axum::{
	extract::State,
	response::Json,
};
use rubedo::sugar::s;
use std::sync::Arc;



//		Functions

//		get_index																
/// Provides data for the index route.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// 
pub async fn get_index(State(_state): State<Arc<AppState>>) -> Json<String> {
	Json(s!("Ok"))
}


