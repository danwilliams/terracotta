//! Authentication functionality.



//		Packages

use crate::{
	auth::handlers::get_login,
	utility::AppState,
};
use axum::{
	Extension,
	async_trait,
	body::Body,
	extract::{FromRequestParts, State},
	http::{Request, StatusCode, Uri, request::Parts},
	middleware::Next,
	response::{IntoResponse, Response},
};
use core::convert::Infallible;
use rubedo::sugar::s;
use serde::Serialize;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::info;



//		Constants

/// The key used to store the session's user ID.
const SESSION_USER_ID_KEY: &str = "_user_id";



//		Structs

//		User																	
/// User data functionality.
/// 
/// This struct contains the user fields used for authentication, and methods
/// for retrieving user data.
/// 
#[derive(Clone, Debug, Serialize)]
pub struct User {
	//		Private properties													
	/// The username.
	pub username: String,
	
	/// The password.
	pub password: String,
}

//󰭅		User																	
impl User {
	//		find																
	/// Finds a user by username and password.
	/// 
	/// Returns [`Some(User)`](Some) if the user exists and the password is
	/// correct, otherwise returns [`None`].
	/// 
	/// # Parameters
	/// 
	/// * `state`    - The application state.
	/// * `username` - The username to search for.
	/// * `password` - The password to match.
	/// 
	pub fn find(state: &Arc<AppState>, username: &String, password: &String) -> Option<Self> {
		if state.config.users.contains_key(username) {
			let pass = state.config.users.get(username)?;
			if pass == password {
				return Some(Self {
					username: username.clone(),
					password: pass.clone(),
				});
			}
		}
		None
	}
	
	//		find_by_id															
	/// Finds a user by username.
	/// 
	/// Returns [`Some(User)`](Some) if the user exists, otherwise returns
	/// [`None`].
	/// 
	/// # Parameters
	/// 
	/// * `state`    - The application state.
	/// * `username` - The username to search for.
	/// 
	pub fn find_by_id(state: &Arc<AppState>, id: &String) -> Option<Self> {
		if state.config.users.contains_key(id) {
			let password = state.config.users.get(id)?;
			return Some(Self {
				username: id.clone(),
				password: password.clone(),
			});
		}
		None
	}
}

//		AuthContext																
/// The authentication context.
/// 
/// This struct contains the current user and session data, to persist the
/// context of an authentication session.
/// 
#[derive(Clone)]
pub struct AuthContext {
	//		Public properties													
	/// The current user.
	pub current_user: Option<User>,
	
	//		Private properties													
	/// The active session.
	session:          Session,
}

//󰭅		AuthContext																
impl AuthContext {
	//		new																	
	/// Creates a new authentication context.
	/// 
	/// # Parameters
	/// 
	/// * `session` - The active session.
	/// * `key`     - The HMAC key.
	/// 
	pub const fn new(session: Session) -> Self {
		Self {
			current_user: None,
			session,
		}
	}
	
	//		get_user															
	/// Gets the current user.
	/// 
	/// Retrieves the current user id from the session, obtains the user's data
	/// from the data store, and verifies the session's authentication ID.
	/// 
	/// # Parameters
	/// 
	/// * `appstate` - The application state.
	/// 
	pub async fn get_user(&self, appstate: &Arc<AppState>) -> Option<User> {
		if let Ok(Some(user_id)) = self.session.get::<String>(SESSION_USER_ID_KEY).await {
			if let Some(user)    = User::find_by_id(appstate, &user_id) {
				return Some(user);
			}
			self.logout().await;
		}
		None
	}
	
	//		login																
	/// Logs in a user.
	/// 
	/// Logs the user in by setting the session's authentication ID and user ID.
	/// It assumes that the user's credentials have already been verified.
	/// 
	/// # Parameters
	/// 
	/// * `user` - The user to log in.
	/// 
	pub async fn login(&mut self, user: &User) {
		let user_id       = &user.username;
		self.session.insert(SESSION_USER_ID_KEY, user_id).await.unwrap();
		self.current_user = Some(user.clone());
	}
	
	//		logout																
	/// Logs out the current user.
	/// 
	/// Logs the current user out by destroying the session.
	/// 
	pub async fn logout(&self) {
		self.session.clear().await;
	}
}

//󰭅		FromRequestParts														
#[async_trait]
impl<State> FromRequestParts<State> for AuthContext
where State: Send + Sync {
	type Rejection = Infallible;
	
	//		from_request_parts													
	/// Creates an authentication context from the request parts.
	/// 
	/// # Parameters
	/// 
	/// * `parts` - The request parts.
	/// * `state` - The application state.
	/// 
	#[expect(clippy::expect_used, reason = "Misconfiguration, so hard quit")]
	async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
		let Extension(auth_cx): Extension<Self> =
			Extension::from_request_parts(parts, state)
				.await
				.expect("Auth extension/layer missing")
		;
		Ok(auth_cx)
	}
}



//		Functions

//		auth_layer																
/// Prepare the authentication context.
/// 
/// This layer is a middleware that is used to set up the authentication
/// context. It retrieves the current user from the session, and stores it in
/// the request's extensions, so that it can be used by the route handlers.
/// 
/// # Parameters
/// 
/// * `appstate` - The application state.
/// * `session`  - The session handle.
/// * `request`  - The request.
/// * `next`     - The next middleware.
/// 
pub async fn auth_layer(
	State(appstate):    State<Arc<AppState>>,
	Extension(session): Extension<Session>,
	mut request:        Request<Body>,
	next:               Next,
) -> Response {
	let mut auth_cx      = AuthContext::new(session);
	let user             = auth_cx.get_user(&appstate).await;
	let mut username     = s!("none");
	if let Some(ref u) = user {
		username.clone_from(&u.username);
	}
	info!("Current user: {username}");
	auth_cx.current_user = user;
	drop(request.extensions_mut().insert(auth_cx.clone()));
	drop(request.extensions_mut().insert(auth_cx.current_user));
	next.run(request).await
}

//		protect																	
/// Protects a route from unauthorised access.
/// 
/// This middleware is used to protect routes from unauthorised access. It
/// retrieves the current user from the request's extensions, and if it is
/// present, it calls the next middleware. Otherwise, it returns a 401 response.
/// 
/// # Parameters
/// 
/// * `appstate` - The application state.
/// * `user`     - The current user.
/// * `uri`      - The request URI.
/// * `request`  - The request.
/// * `next`     - The next middleware.
/// 
pub async fn protect(
	State(appstate): State<Arc<AppState>>,
	Extension(user): Extension<Option<User>>,
	uri:             Uri,
	request:         Request<Body>,
	next:            Next,
) -> Response {
	match user {
		Some(_) => {
			//	let user = user.clone();
			//	request.extensions_mut().insert(user);
			next.run(request).await
		},
		_       => {
			(
				StatusCode::UNAUTHORIZED,
				get_login(State(appstate), uri).await,
			).into_response()
		},
	}
}


