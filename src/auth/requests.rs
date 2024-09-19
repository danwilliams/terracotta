//! Request data for authentication functionality.



//		Packages

use super::middleware::Credentials;
use serde::{Deserialize, Deserializer};



//		Structs

//		PostLogin																
/// The data sent by the login form.
/// 
/// This is consumed by the [`post_login()`](super::handlers::post_login())
/// handler.
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostLogin<C: Credentials> {
	//		Private properties													
	/// The user credentials needed to log in.
	pub credentials: C,
	
	/// The URL to redirect to after logging in.
	pub uri:         String,
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


