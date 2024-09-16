//! Routing functionality for authentication.



//		Packages

use crate::{
	auth::middleware::protect,
	utility::AppState,
};
use axum::{
	Router,
	middleware::from_fn_with_state,
	routing::MethodRouter,
};
use std::sync::Arc;



//		Traits

//§		RouterExt																
/// Authentication extension methods for the Axum [`Router`].
pub trait RouterExt<S: Clone + Send + Sync + 'static> {
	//		protected_routes													
	/// Adds protected routes to the router.
	/// 
	/// This is a convenience method that adds the given routes to the router,
	/// then adds a middleware layer to protect them.
	/// 
	/// # Parameters
	/// 
	/// * `routes`       - The routes to add.
	/// * `shared_state` - The shared application state.
	/// 
	/// # See also
	/// 
	/// * [`public_routes()`](#method.public_routes)
	/// 
	fn protected_routes(self, routes: Vec<(&str, MethodRouter<S>)>, shared_state: Arc<AppState>) -> Self;
	
	//		public_routes														
	/// Adds public routes to the router.
	/// 
	/// This is a convenience method that adds the given routes to the router.
	/// It is useful when combined with [`protected_routes()`](#method.protected_routes)
	/// to clearly separate public and protected routes.
	/// 
	/// # Parameters
	/// 
	/// * `routes` - The routes to add.
	/// 
	/// # See also
	/// 
	/// * [`protected_routes()`](#method.protected_routes)
	/// 
	fn public_routes(self, routes: Vec<(&str, MethodRouter<S>)>) -> Self;
}

//󰭅		RouterExt																
#[expect(clippy::similar_names, reason = "Not too similar")]
impl<S: Clone + Send + Sync + 'static> RouterExt<S> for Router<S> {
	//		protected_routes													
	fn protected_routes(self, routes: Vec<(&str, MethodRouter<S>)>, shared_state: Arc<AppState>) -> Self {
		let mut router = self;
		for (path, method_router) in routes {
			router = router.route(path, method_router);
		}
		router
			.route_layer(from_fn_with_state(Arc::clone(&shared_state), protect))
	}
	
	//		public_routes														
	fn public_routes(self, routes: Vec<(&str, MethodRouter<S>)>) -> Self {
		let mut router = self;
		for (path, method_router) in routes {
			router = router.route(path, method_router);
		}
		router
	}
}


