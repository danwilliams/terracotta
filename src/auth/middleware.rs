//! Authentication functionality.



//		Packages

use super::{
	handlers::get_login,
	state::StateProvider,
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
use core::{
	convert::Infallible,
	fmt::Debug,
};
use rubedo::sugar::s;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::info;



//		Constants

/// The key used to store the session's user ID.
const SESSION_USER_ID_KEY: &str = "_user_id";



//		Structs

//		Context																	
/// The authentication context.
/// 
/// This struct contains the current user and session data, to persist the
/// context of an authentication session.
/// 
#[derive(Clone, Debug)]
pub struct Context<U: User> {
	//		Public properties													
	/// The current user.
	pub current_user: Option<U>,
	
	//		Private properties													
	/// The active session.
	session:          Session,
}

//󰭅		Context																	
impl<U: User> Context<U> {
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
	/// * `state` - The application state.
	/// 
	pub async fn get_user<SP, UP>(&self, state: &Arc<SP>) -> Option<U>
	where
		SP: StateProvider,
		UP: UserProvider<User = U>,
	{
		if let Ok(Some(user_id)) = self.session.get::<String>(SESSION_USER_ID_KEY).await {
			if let Some(user)    = UP::find_by_id(state, &user_id) {
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
	pub async fn login(&mut self, user: &U) {
		let user_id       = &user.id();
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
impl<S, U> FromRequestParts<S> for Context<U>
where
	S: Send + Sync,
	U: User,
{
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
	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let Extension(auth_cx): Extension<Self> =
			Extension::from_request_parts(parts, state)
				.await
				.expect("Auth extension/layer missing")
		;
		Ok(auth_cx)
	}
}



//		Traits

//§		Credentials																
/// An instance of user data providing enough functionality for authentication.
/// 
/// This gets used to authenticate the user, notably being sent via POST from
/// the login form.
/// 
pub trait Credentials: Clone + Debug + for<'de> Deserialize<'de> + Send + Sync + 'static {}

//§		User																	
/// An instance of user data providing enough functionality for identification.
/// 
/// This gets stored in application state, so ideally should not be too large.
/// Just the basics for identification are usually sufficient.
/// 
pub trait User: Clone + Debug + Send + Sync + 'static {
	//		id																	
	/// The user's unique identifier.
	/// 
	/// This function gets the user's unique identifier for the purposes of
	/// authentication. This could be an ID, username, email, or similar.
	/// 
	fn id(&self) -> &String;
}

//§		UserProvider															
/// A trait for providing basic user data.
pub trait UserProvider: Debug + 'static {
	/// The credentials data type. This is the type that implements the
	/// [`Credentials`] trait.
	type Credentials: Credentials;
	
	/// The user data type. This is the type that implements the [`User`] trait.
	type User:        User;
	
	//		find_by_credentials													
	/// Finds a user by matching credentials.
	/// 
	/// Returns [`Some(User)`](Some) if the user exists and the credentials are
	/// correct, otherwise returns [`None`].
	/// 
	/// # Parameters
	/// 
	/// * `state`       - The application state.
	/// * `credentials` - The credentials to check.
	/// 
	fn find_by_credentials<SP: StateProvider>(
		state:       &Arc<SP>,
		credentials: &Self::Credentials,
	) -> Option<Self::User>;
	
	//		find_by_id															
	/// Finds a user by unique identifier.
	/// 
	/// The unique identifier could be an ID, username, email, or similar.
	/// 
	/// Returns [`Some(User)`](Some) if the user exists, otherwise returns
	/// [`None`].
	/// 
	/// # Parameters
	/// 
	/// * `state` - The application state.
	/// * `id`    - The identifying field to search for.
	/// 
	fn find_by_id<SP: StateProvider>(
		state: &Arc<SP>,
		id:    &str,
	) -> Option<Self::User>;
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
/// * `state`   - The application state.
/// * `session` - The session handle.
/// * `request` - The request.
/// * `next`    - The next middleware.
/// 
pub async fn auth_layer<SP, U, UP>(
	State(state):       State<Arc<SP>>,
	Extension(session): Extension<Session>,
	mut request:        Request<Body>,
	next:               Next,
) -> Response
where
	SP: StateProvider,
	U:  User,
	UP: UserProvider<User = U>,
{
	let mut auth_cx      = Context::<U>::new(session);
	let user             = auth_cx.get_user::<SP, UP>(&state).await;
	let mut username     = s!("none");
	if let Some(ref u) = user {
		username.clone_from(u.id());
	}
	info!("Current user: {username}");
	auth_cx.current_user = user;
	drop(request.extensions_mut().insert(auth_cx));
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
/// * `state`   - The application state.
/// * `auth_cx` - The authentication context.
/// * `uri`     - The request URI.
/// * `request` - The request.
/// * `next`    - The next middleware.
/// 
pub async fn protect<SP, U>(
	State(state):       State<Arc<SP>>,
	Extension(auth_cx): Extension<Context<U>>,
	uri:                Uri,
	request:            Request<Body>,
	next:               Next,
) -> Response
where
	SP: StateProvider,
	U:  User,
{
	match auth_cx.current_user {
		Some(_) => next.run(request).await,
		_       => {
			(
				StatusCode::UNAUTHORIZED,
				get_login(State(state), uri).await,
			).into_response()
		},
	}
}


