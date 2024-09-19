//! Utility functions and types for the application.



//		Packages

use crate::{
	auth::{
		middleware::{User as AuthUser, Credentials as AuthCredentials, UserProvider as AuthUserProvider},
		state::StateProvider as AuthStateProvider,
	},
	health,
	stats,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::OpenApi;



//		Structs

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		health::handlers::get_ping,
		health::handlers::get_version,
		stats::handlers::get_stats,
		stats::handlers::get_stats_history,
		stats::handlers::get_stats_feed,
	),
	components(
		schemas(
			health::responses::HealthVersionResponse,
			stats::responses::StatsResponse,
			stats::responses::StatsResponseForPeriod,
			stats::responses::StatsHistoryResponse,
		),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	),
)]
pub struct ApiDoc;

//		Credentials																
/// The data required to authenticate a user.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Credentials {
	//		Private properties													
	/// The username.
	username: String,
	
	/// The password.
	password: String,
}

//󰭅		AuthCredentials															
impl AuthCredentials for Credentials {
	//		to_loggable_string													
	fn to_loggable_string(&self) -> String {
		self.username.clone()
	}
}

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
}

//󰭅		AuthUser																
impl AuthUser for User {
	type Id = String;
	
	//		id																	
	fn id(&self) -> &Self::Id {
		&self.username
	}
	
	//		to_loggable_string													
	fn to_loggable_string(&self) -> String {
		self.username.clone()
	}
}

//󰭅		AuthUserProvider														
impl AuthUserProvider for User {
	type Credentials = Credentials;
	type User        = Self;
	
	//		find_by_credentials													
	fn find_by_credentials<SP: AuthStateProvider>(
		state:       &Arc<SP>,
		credentials: &Self::Credentials,
	) -> Option<Self> {
		state.users()
			.get(&credentials.username)
			.filter(|&pass| pass == &credentials.password)
			.map(|_| Self { username: credentials.username.clone() })
	}
	
	//		find_by_id															
	fn find_by_id<SP: AuthStateProvider>(
		state: &Arc<SP>,
		id:    &<Self::User as AuthUser>::Id,
	) -> Option<Self> {
		state.users()
			.get(id)
			.map(|_| Self { username: id.to_owned() })
	}
}


