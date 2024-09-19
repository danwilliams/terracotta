//! Response data for health check functionality.



//		Packages

use serde::Serialize;
use utoipa::ToSchema;



//		Structs

//		HealthVersionResponse													
/// The current version information returned by the `/api/version` endpoint.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, ToSchema)]
pub struct HealthVersionResponse {
	//		Public properties													
	/// The current version of the application.
	pub version: String,
}


