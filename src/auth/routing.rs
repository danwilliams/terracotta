//! Routing functionality for authentication.



//		Packages																										

use super::{
	middleware::{User, UserProvider as UserProvider, auth_layer, protect, protected_error_layer},
	state::StateProvider,
};
use crate::app::state::StateProvider as AppStateProvider;
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
	#[must_use]
	fn add_authentication<SP, U, UP>(self, state: &Arc<SP>) -> Self
	where
		SP: StateProvider,
		U:  User,
		UP: UserProvider<User = U>,
	;
	
	//		add_protected_error_catcher											
	/// Adds an error handler that protects sensitive errors.
	/// 
	/// # Parameters
	/// 
	/// * `state` - The application state.
	/// 
	#[must_use]
	fn add_protected_error_catcher<SP, U>(self, state: &Arc<SP>) -> Self
	where
		SP: AppStateProvider,
		U:  User,
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
	/// * [`public_routes()`](crate::app::routing::RouterExt::public_routes())
	/// 
	#[must_use]
	fn protected_routes<SP, U>(self, routes: Vec<(&str, MethodRouter<S>)>, state: &Arc<SP>) -> Self
	where
		SP: StateProvider,
		U:  User,
	;
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
	
	//		add_protected_error_catcher											
	fn add_protected_error_catcher<SP, U>(self, state: &Arc<SP>) -> Self
	where
		SP: AppStateProvider,
		U:  User,
	{
		self
			.layer(from_fn_with_state(Arc::clone(state), protected_error_layer::<_, U>))
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
}


