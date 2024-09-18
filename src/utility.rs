//! Utility functions and types for the application.



//		Packages

use crate::{
	auth::{
		middleware::{User as AuthUser, UserProvider as AuthUserProvider},
		state::AuthStateProvider,
	},
	health::handlers as health,
	stats::handlers as stats,
};
use serde::Serialize;
use std::sync::Arc;
use utoipa::OpenApi;



//		Structs

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		health::get_ping,
		health::get_version,
		stats::get_stats,
		stats::get_stats_history,
		stats::get_stats_feed,
	),
	components(
		schemas(
			health::HealthVersionResponse,
			stats::StatsResponse,
			stats::StatsResponseForPeriod,
			stats::StatsHistoryResponse,
		),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	),
)]
pub struct ApiDoc;

//		User																	
/// User data functionality.
/// 
/// This struct contains the user fields used for authentication, and methods
/// for retrieving user data.
/// 
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct User {
	//		Private properties													
	/// The username.
	pub username: String,
	
	/// The password.
	pub password: String,
}

//󰭅		AuthUser																
impl AuthUser for User {
	//		id																	
	fn id(&self) -> &String {
		&self.username
	}
}

//󰭅		AuthUserProvider														
impl AuthUserProvider for User {
	type User = Self;
	
	//		find_by_credentials													
	fn find_by_credentials<S: AuthStateProvider>(
		state:    &Arc<S>,
		username: &str,
		password: &str,
	) -> Option<Self> {
		if state.users().contains_key(username) {
			let pass = state.users().get(username)?;
			if pass == password {
				return Some(Self {
					username: username.to_owned(),
					password: pass.clone(),
				});
			}
		}
		None
	}
	
	//		find_by_id															
	fn find_by_id<S: AuthStateProvider>(
		state: &Arc<S>,
		id:    &str,
	) -> Option<Self> {
		if state.users().contains_key(id) {
			let password = state.users().get(id)?;
			return Some(Self {
				username: id.to_owned(),
				password: password.clone(),
			});
		}
		None
	}
}


