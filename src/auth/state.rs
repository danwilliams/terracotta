//! State for the authentication functionality.



//		Packages

use crate::utility::AppStateProvider;
use std::collections::HashMap;



//		Traits

//§		AuthStateProvider														
/// A trait for providing the application state aspects for authentication.
pub trait AuthStateProvider: AppStateProvider + Send + Sync + 'static {
	//		users																
	/// Gets list of users.
	fn users(&self) -> &HashMap<String, String>;
}


