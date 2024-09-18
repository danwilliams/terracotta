//! Routing functionality for statistics-gathering.



//		Packages

use super::{
	middleware::stats_layer,
	state::StateProvider,
};
use axum::{
	Router,
	middleware::from_fn_with_state,
};
use std::sync::Arc;



//		Traits

//§		RouterExt																
/// Error-handling extension methods for the Axum [`Router`].
pub trait RouterExt<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_stats_gathering													
	/// Adds a statistics-gathering layer.
	/// 
	/// # Parameters
	/// 
	/// * `shared_state` - The shared application state.
	/// 
	fn add_stats_gathering<SP: StateProvider>(self, shared_state: &Arc<SP>) -> Self;
}

//󰭅		RouterExt																
impl<S> RouterExt<S> for Router<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_stats_gathering													
	fn add_stats_gathering<SP: StateProvider>(self, shared_state: &Arc<SP>) -> Self {
		self.layer(from_fn_with_state(Arc::clone(shared_state), stats_layer))
	}
}


