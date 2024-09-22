#![allow(clippy::exhaustive_structs, reason = "Handlers have auto-generated OpenAPI documentation")]
#![allow(clippy::unused_async,       reason = "Handler functions need to be async")]

//! Endpoint handlers for authentication functionality.



//		Packages

use super::{
	errors::AuthError,
	middleware::{Context, Credentials, User, UserProvider},
	requests::PostLogin,
	state::StateProvider,
	utility::{build_uri, extract_uri_query_parts},
};
use crate::app::{
	errors::AppError,
	state::StateProvider as AppStateProvider,
};
use axum::{
	Form,
	extract::State,
	http::Uri,
	response::{Html, Redirect},
};
use rubedo::sugar::s;
use std::sync::Arc;
use tera::Context as Template;
use tracing::{info, warn};



//		Functions

//		get_login																
/// Shows the login page.
/// 
/// Renders the login template.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `uri`   - The request URI.
/// 
/// # Errors
/// 
/// If there is an error rendering the login page, an error will be returned.
/// 
pub async fn get_login<SP: AppStateProvider>(
	State(state): State<Arc<SP>>,
	mut uri:      Uri,
) -> Result<Html<String>, AppError> {
	let mut params = extract_uri_query_parts(&uri);
	let mut failed = false;
	if params.contains_key("failed") {
		failed     = true;
		drop(params.remove("failed"));
	}
	uri              = build_uri(uri.path(), &params).map_err(AuthError::from)?;
	let mut template = Template::new();
	template.insert("Title",   &state.title());
	template.insert("PageURL", &uri.path_and_query().map_or_else(|| s!("/"), ToString::to_string));
	template.insert("Failed",  &failed);
	Ok(Html(state.render("login", &template).await?))
}

//		post_login																
/// Processes the login form.
/// 
/// Logs the user in if the credentials are valid, and redirects to the
/// requested page. Otherwise, it redirects back to the login page with a
/// `failed` parameter.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `auth`  - The authentication context.
/// * `login` - The login form.
/// 
/// # Errors
/// 
/// If there is an error processing the login form, an error will be returned.
/// 
pub async fn post_login<SP, C, U, UP>(
	State(state): State<Arc<SP>>,
	mut auth:     Context<U>,
	Form(login):  Form<PostLogin<C>>,
) -> Result<Redirect, AuthError>
where
	SP: StateProvider,
	C:  Credentials,
	U:  User,
	UP: UserProvider<Credentials = C, User = U>,
{
	let uri        = login.uri.parse::<Uri>()?;
	let mut params = extract_uri_query_parts(&uri);
	if let Some(ref user) = UP::find_by_credentials(&*state, &login.credentials) {
		info!("Logging in user: {}", user.to_loggable_string());
		auth.login(user).await?;
	} else {
		drop(params.insert(s!("failed"), s!("")));
		warn!("Failed login attempt for user: {}", &login.credentials.to_loggable_string());
	}
	Ok(Redirect::to(
		&build_uri(uri.path(), &params)?
			.path_and_query()
			.map_or_else(|| s!("/"), ToString::to_string)
	))
}

//		get_logout																
/// Logs the user out.
/// 
/// Logs the user out, and redirects to the home page.
/// 
/// # Parameters
/// 
/// * `auth` - The authentication context.
/// 
pub async fn get_logout<U: User>(
	auth: Context<U>,
) -> Redirect {
	if let Some(ref user) = auth.current_user {
		info!("Logging out user: {}", user.to_loggable_string());
	}
	auth.logout().await;
	Redirect::to("/")
}


