//! Routing functionality for error handling.



//		Packages

use crate::{
	errors::middleware::{final_error_layer, graceful_error_layer},
	state::AppStateProvider,
};
use axum::{
	Router,
	middleware::{from_fn, from_fn_with_state},
};
use std::sync::Arc;
use tower_http::catch_panic::CatchPanicLayer;



//		Traits

//§		RouterExt																
/// Error-handling extension methods for the Axum [`Router`].
pub trait RouterExt<S: Clone + Send + Sync + 'static> {
	//		add_error_catcher													
	/// Adds a final error handler that catches all errors.
	fn add_error_catcher(self) -> Self;
	
	//		add_error_template													
	/// Adds a graceful error handler that attempts to render an error template.
	/// 
	/// # Parameters
	/// 
	/// * `shared_state` - The shared application state.
	/// 
	fn add_error_template<P: AppStateProvider>(self, shared_state: &Arc<P>) -> Self;
}

//󰭅		RouterExt																
impl<S: Clone + Send + Sync + 'static> RouterExt<S> for Router<S> {
	//		add_error_catcher													
	fn add_error_catcher(self) -> Self {
		self
			.layer(CatchPanicLayer::new())
			.layer(from_fn(final_error_layer))
	}
	
	//		add_error_template													
	fn add_error_template<P: AppStateProvider>(self, shared_state: &Arc<P>) -> Self {
		self
			.layer(CatchPanicLayer::new())
			.layer(from_fn_with_state(Arc::clone(shared_state), graceful_error_layer))
	}
}


