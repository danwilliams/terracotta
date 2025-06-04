//! State for the authentication functionality.



//		Packages																										

use crate::app::state::StateProvider as AppStateProvider;
use std::collections::HashMap;



//		Traits																											

//§		StateProvider															
/// A trait for providing the application state aspects for authentication.
pub trait StateProvider: AppStateProvider + Send + Sync + 'static {
	//		users																
	/// Gets list of users.
	fn users(&self) -> &HashMap<String, String>;
}


