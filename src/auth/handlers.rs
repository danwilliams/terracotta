//! Endpoint handlers for authentication functionality.



//		Packages

use super::{
	middleware::{Context, Credentials, User, UserProvider},
	state::StateProvider,
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
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use tera::Context as Template;
use tracing::{info, warn};



//		Structs

//		PostLogin																
/// The data sent by the login form.
/// 
/// This is consumed by the [`post_login()`] handler.
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostLogin<C: Credentials> {
	//		Private properties													
	/// The user credentials needed to log in.
	credentials: C,
	
	/// The URL to redirect to after logging in.
	uri:         String,
}

//󰭅		Deserialize																
impl<'de, C> Deserialize<'de> for PostLogin<C>
where
	C: Credentials,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[allow(clippy::allow_attributes,              reason = "The allow below doesn't work with an expect")]
		#[allow(clippy::missing_docs_in_private_items, reason = "Internal helper struct")]
		#[derive(Deserialize)]
		struct Helper<C> {
			#[serde(flatten)]
			credentials: C,
			uri:         String,
		}
		
		let helper = Helper::deserialize(deserializer)?;
		
		Ok(Self {
			credentials: helper.credentials,
			uri:         helper.uri,
		})
	}
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
	let mut template = Template::new();
	template.insert("Title",   &state.title());
	template.insert("PageURL", &uri.path_and_query().unwrap().to_string());
	template.insert("Failed",  &failed);
	Html(state.render("login", &template).unwrap())
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
pub async fn post_login<SP, C, U, UP>(
	State(state): State<Arc<SP>>,
	mut auth:     Context<U>,
	Form(login):  Form<PostLogin<C>>,
) -> Redirect
where
	SP: StateProvider,
	C:  Credentials,
	U:  User,
	UP: UserProvider<Credentials = C, User = U>,
{
	let uri        = login.uri.parse::<Uri>().unwrap();
	let mut params = extract_uri_query_parts(&uri);
	if let Some(ref user) = UP::find_by_credentials(&state, &login.credentials) {
		info!("Logging in user: {}", user.to_loggable_string());
		auth.login(user).await;
	} else {
		drop(params.insert(s!("failed"), s!("")));
		warn!("Failed login attempt for user: {}", &login.credentials.to_loggable_string());
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
	auth: Context<U>,
) -> Redirect {
	if let Some(ref user) = auth.current_user {
		info!("Logging out user: {}", user.to_loggable_string());
	}
	auth.logout().await;
	Redirect::to("/")
}


