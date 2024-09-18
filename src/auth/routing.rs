//! Routing functionality for authentication.



//		Packages

use super::{
	middleware::{User, UserProvider as UserProvider, auth_layer, protect},
	state::StateProvider,
};
use axum::{
	Router,
	middleware::from_fn_with_state,
	routing::MethodRouter,
};
use std::sync::Arc;
use tower_sessions::{
	MemoryStore as SessionMemoryStore,
	SessionManagerLayer,
	cookie::Key as SessionKey,
};



//		Traits

//§		RouterExt																
/// Authentication extension methods for the Axum [`Router`].
pub trait RouterExt<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_authentication													
	/// Adds the authentication layer.
	/// 
	/// # Parameters
	/// 
	/// * `state` - The application state.
	/// 
	fn add_authentication<SP, U, UP>(self, state: &Arc<SP>) -> Self
	where
		SP: StateProvider,
		U:  User,
		UP: UserProvider<User = U>,
	;
	
	//		protected_routes													
	/// Adds protected routes to the router.
	/// 
	/// This is a convenience method that adds the given routes to the router,
	/// then adds a middleware layer to protect them.
	/// 
	/// # Parameters
	/// 
	/// * `routes` - The routes to add.
	/// * `state`  - The application state.
	/// 
	/// # See also
	/// 
	/// * [`public_routes()`](#method.public_routes)
	/// 
	fn protected_routes<SP, U>(self, routes: Vec<(&str, MethodRouter<S>)>, state: &Arc<SP>) -> Self
	where
		SP: StateProvider,
		U:  User,
	;
	
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
impl<S> RouterExt<S> for Router<S>
where
	S: Clone + Send + Sync + 'static,
{
	//		add_authentication													
	fn add_authentication<SP, U, UP>(self, state: &Arc<SP>) -> Self
	where
		SP: StateProvider,
		U:  User,
		UP: UserProvider<User = U>,
	{
		let session_key   = SessionKey::generate();
		let session_store = SessionMemoryStore::default();
		self
			.layer(from_fn_with_state(Arc::clone(state), auth_layer::<_, U, UP>))
			.layer(SessionManagerLayer::new(session_store).with_secure(false).with_signed(session_key))
	}
	
	//		protected_routes													
	fn protected_routes<SP, U>(self, routes: Vec<(&str, MethodRouter<S>)>, state: &Arc<SP>) -> Self
	where
		SP: StateProvider,
		U:  User,
	{
		let mut router = self;
		for (path, method_router) in routes {
			router = router.route(path, method_router);
		}
		router
			.route_layer(from_fn_with_state(Arc::clone(state), protect::<_, U>))
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


