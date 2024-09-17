//! Routing functionality for statistics-gathering.



//		Packages

use crate::stats::{
	middleware::stats_layer,
	state::StatsStateProvider,
};
use axum::{
	Router,
	middleware::from_fn_with_state,
};
use std::sync::Arc;



//		Traits

//§		RouterExt																
/// Error-handling extension methods for the Axum [`Router`].
pub trait RouterExt<S: Clone + Send + Sync + 'static> {
	//		add_stats_gathering													
	/// Adds a statistics-gathering layer.
	/// 
	/// # Parameters
	/// 
	/// * `shared_state` - The shared application state.
	/// 
	fn add_stats_gathering<P: StatsStateProvider>(self, shared_state: &Arc<P>) -> Self;
}

//󰭅		RouterExt																
impl<S: Clone + Send + Sync + 'static> RouterExt<S> for Router<S> {
	//		add_stats_gathering													
	fn add_stats_gathering<P: StatsStateProvider>(self, shared_state: &Arc<P>) -> Self {
		self.layer(from_fn_with_state(Arc::clone(shared_state), stats_layer))
	}
}


