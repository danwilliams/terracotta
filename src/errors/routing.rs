//! Routing functionality for error handling.



//		Packages

use super::middleware::final_error_layer;
use axum::{
	Router,
	middleware::from_fn,
};
use tower_http::catch_panic::CatchPanicLayer;

#[cfg(feature = "tera")]
use super::middleware::graceful_error_layer;
#[cfg(feature = "tera")]
use crate::app::state::StateProvider as AppStateProvider;
#[cfg(feature = "tera")]
use ::{
	axum::middleware::from_fn_with_state,
	std::sync::Arc,
};



//		Traits

//§		RouterExt																
/// Error-handling extension methods for the Axum [`Router`].
pub trait RouterExt<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_error_catcher													
	/// Adds a final error handler that catches all errors.
	#[must_use]
	fn add_error_catcher(self) -> Self;
	
	//		add_error_template													
	/// Adds a graceful error handler that attempts to render an error template.
	/// 
	/// # Parameters
	/// 
	/// * `state` - The application state.
	/// 
	#[cfg(feature = "tera")]
	#[must_use]
	fn add_error_template<SP>(self, state: &Arc<SP>) -> Self
	where
		SP: AppStateProvider,
	;
}

//󰭅		RouterExt																
impl<S> RouterExt<S> for Router<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_error_catcher													
	fn add_error_catcher(self) -> Self {
		self
			.layer(CatchPanicLayer::new())
			.layer(from_fn(final_error_layer))
	}
	
	//		add_error_template													
	#[cfg(feature = "tera")]
	fn add_error_template<SP>(self, state: &Arc<SP>) -> Self
	where
		SP: AppStateProvider,
	{
		self
			.layer(CatchPanicLayer::new())
			.layer(from_fn_with_state(Arc::clone(state), graceful_error_layer))
	}
}


