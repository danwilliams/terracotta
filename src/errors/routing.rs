//! Routing functionality for error handling.



//		Packages

use super::middleware::{final_error_layer, graceful_error_layer};
use crate::{
	app::state::StateProvider as AppStateProvider,
	auth::middleware::User as AuthUser,
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
	#[must_use]
	fn add_error_template<SP, U>(self, state: &Arc<SP>) -> Self
	where
		SP: AppStateProvider,
		U:  AuthUser,
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
	fn add_error_template<SP, U>(self, state: &Arc<SP>) -> Self
	where
		SP: AppStateProvider,
		U:  AuthUser,
	{
		self
			.layer(CatchPanicLayer::new())
			.layer(from_fn_with_state(Arc::clone(state), graceful_error_layer::<_, U>))
	}
}


