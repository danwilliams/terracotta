//! Endpoint handlers for authentication functionality.



//		Packages

use super::{
	middleware::{AuthContext, User, UserProvider},
	state::AuthStateProvider,
	utility::{build_uri, extract_uri_query_parts},
};
use crate::state::AppStateProvider;
use axum::{
	Form,
	extract::State,
	http::Uri,
	response::{Html, Redirect},
};
use rubedo::sugar::s;
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;
use tracing::info;



//		Structs

//		PostLogin																
/// The data sent by the login form.
/// 
/// This is consumed by the [`post_login()`] handler.
/// 
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PostLogin {
	//		Private properties													
	/// The username.
	username: String,
	
	/// The password.
	password: String,
	
	/// The URL to redirect to after logging in.
	uri:      String,
}



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
pub async fn get_login<SP: AppStateProvider>(
	State(state): State<Arc<SP>>,
	mut uri:      Uri,
) -> Html<String> {
	let mut params  = extract_uri_query_parts(&uri);
	let mut failed  = false;
	if params.contains_key("failed") {
		failed      = true;
		drop(params.remove("failed"));
	}
	uri             = build_uri(uri.path(), &params);
	let mut context = Context::new();
	context.insert("Title",   &state.title());
	context.insert("PageURL", &uri.path_and_query().unwrap().to_string());
	context.insert("Failed",  &failed);
	Html(state.render("login", &context).unwrap())
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
pub async fn post_login<SP, U, UP>(
	State(state): State<Arc<SP>>,
	mut auth:     AuthContext<U>,
	Form(login):  Form<PostLogin>,
) -> Redirect
where
	SP: AuthStateProvider,
	U:  User,
	UP: UserProvider<User = U>,
{
	let uri        = login.uri.parse::<Uri>().unwrap();
	let mut params = extract_uri_query_parts(&uri);
	let user       = UP::find_by_credentials(&state, &login.username, &login.password);
	if user.is_some() {
		info!("Logging in user: {}", user.as_ref().unwrap().id());
		auth.login(user.as_ref().unwrap()).await;
	} else {
		drop(params.insert(s!("failed"), s!("")));
		info!("Failed login attempt for user: {}", &login.username);
	}
	Redirect::to(build_uri(uri.path(), &params).path_and_query().unwrap().to_string().as_str())
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
	auth: AuthContext<U>,
) -> Redirect {
	if auth.current_user.is_some() {
		info!("Logging out user: {}", auth.current_user.as_ref().unwrap().id());
	}
	auth.logout().await;
	Redirect::to("/")
}


