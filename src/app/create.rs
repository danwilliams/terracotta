//! Functionality to create the application.



//		Packages

use super::{
	errors::AppError,
	state::StateProvider,
};
use axum::Router;
use core::net::SocketAddr;
use std::sync::Arc;
use tokio::{
	net::TcpListener,
	task::JoinHandle as TaskHandle,
	spawn as spawn_async,
};

#[cfg(all(feature = "auth", feature = "stats"))]
use crate::{
	auth::{
		middleware::{User as AuthUser, UserProvider as AuthUserProvider},
		routing::RouterExt as AuthRouterExt,
		state::StateProvider as AuthStateProvider,
	},
	stats::{
		routing::RouterExt as StatsRouterExt,
		state::StateProvider as StatsStateProvider,
	},
};
#[cfg(feature = "errors")]
use super::routing::RouterExt;
#[cfg(feature = "errors")]
use crate::errors::{
	middleware::no_route,
	routing::RouterExt as ErrorsRouterExt,
};
#[cfg(feature = "errors")]
use ::{
	axum::routing::MethodRouter,
	utoipa::openapi::OpenApi,
};



//		Functions

//		app_full																
/// Creates the application router, with full functionality.
/// 
/// # Parameters
/// 
/// * `state`     - The application state.
/// * `protected` - The protected routes.
/// * `public`    - The public routes.
/// * `openapi`   - The OpenAPI documentation.
/// 
#[cfg(all(feature = "auth", feature = "stats"))]
pub fn app_full<SP, U, UP>(
	state:     &Arc<SP>,
	protected: Vec<(&str, MethodRouter<Arc<SP>>)>,
	public:    Vec<(&str, MethodRouter<Arc<SP>>)>,
	openapi:   OpenApi,
) -> Router
where
	SP: StateProvider + AuthStateProvider + StatsStateProvider,
	U:  AuthUser,
	UP: AuthUserProvider<User = U>,
{
	Router::new()
		.protected_routes::<_, U>(protected, state)
		.public_routes(public)
		.add_openapi("/api-docs", openapi)
		.fallback(no_route)
		.add_protected_error_catcher::<_, U>(state)
		.add_error_template(state)
		.add_authentication::<_, U, UP>(state)
		.add_stats_gathering(state)
		.with_state(Arc::clone(state))
		.add_http_logging()
		.add_error_catcher()
}

//		app_minimal																
/// Creates the application router, with minimal functionality.
/// 
/// # Parameters
/// 
/// * `state`   - The application state.
/// * `routes`  - The routes.
/// * `openapi` - The OpenAPI documentation.
/// 
#[cfg(feature = "errors")]
pub fn app_minimal<SP>(
	state:   &Arc<SP>,
	routes:  Vec<(&str, MethodRouter<Arc<SP>>)>,
	openapi: OpenApi,
) -> Router
where
	SP: StateProvider,
{
	#[cfg(feature = "tera")]
	{
		Router::new()
			.public_routes(routes)
			.add_openapi("/api-docs", openapi)
			.fallback(no_route)
			.add_error_template(state)
			.with_state(Arc::clone(state))
			.add_http_logging()
			.add_error_catcher()
	}
	#[cfg(not(feature = "tera"))]
	{
		Router::new()
			.public_routes(routes)
			.add_openapi("/api-docs", openapi)
			.fallback(no_route)
			.with_state(Arc::clone(state))
			.add_http_logging()
			.add_error_catcher()
	}
}

//		server																	
/// Creates the application server.
/// 
/// # Parameters
/// 
/// * `app`   - The application router.
/// * `state` - The application state.
/// 
/// # Errors
/// 
/// If the configured host and port cannot be bound to, or if the server cannot
/// be started, an error is returned.
/// 
pub async fn server<SP>(
	app:   Router,
	state: &Arc<SP>
) -> Result<TaskHandle<Result<(), AppError>>, AppError>
where
	SP: StateProvider,
{
	let listener = TcpListener::bind(SocketAddr::from((state.host(), state.port()))).await?;
	state.set_address(Some(listener.local_addr()?));
	Ok(spawn_async(async move {
		axum::serve(listener, app).await
			.map_err(AppError::CouldNotStartServer)
	}))
}


