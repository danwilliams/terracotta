#![allow(non_snake_case)]

//		Packages

use crate::handlers;
use axum::http::{StatusCode, Uri};
use chrono::NaiveDateTime;
use ring::hmac;
use serde::{Deserialize, Serialize, Serializer};
use smart_default::SmartDefault;
use std::{
	collections::{BTreeMap, HashMap},
	net::IpAddr,
	path::PathBuf,
	sync::atomic::AtomicUsize,
};
use tera::Tera;
use url::form_urlencoded;
use utoipa::OpenApi;
use velcro::hash_map;



//		Enums

//		LoadingBehavior															
/// The possible options for loading local, non-baked-in resources.
#[derive(Debug, Deserialize, Serialize)]
pub enum LoadingBehavior {
	/// Deny loading of local resources.
	Deny,
	
	/// Load local resources if the baked-in resources are not present.
	Supplement,
	
	/// Load local resources if they exist, otherwise load baked-in resources.
	Override,
}



//		Structs

//		Config																	
/// The main configuration options for the application.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct Config {
	//		Public properties													
	/// The host to listen on.
	#[default(IpAddr::from([127, 0, 0, 1]))]
	pub host:          IpAddr,
	
	/// The port to listen on.
	#[default = 8000]
	pub port:          u16,
	
	/// The directory to store log files in.
	#[default = "log"]
	pub logdir:        String,
	
	/// The title of the application.
	#[default = "Terracotta"]
	pub title:         String,
	
	/// The loading behaviour for local, non-baked-in resources. This allows
	/// local resources to be used to complement the baked-in resources.
	pub local_loading: LocalLoading,
	
	/// The paths for local, non-baked-in resources.
	pub local_paths:   LocalPaths,
	
	/// The configuration options for serving static files.
	pub static_files:  StaticFiles,
	
	/// A list of users and their passwords.
	#[default(HashMap::new())]
	pub users:         HashMap<String, String>,
}

//		LocalLoading															
/// The loading behaviour for local, non-baked-in resources.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct LocalLoading {
	//		Public properties													
	/// The loading behaviour for protected static assets.
	#[default(LoadingBehavior::Deny)]
	pub protected_assets: LoadingBehavior,
	
	/// The loading behaviour for public static assets.
	#[default(LoadingBehavior::Deny)]
	pub public_assets:    LoadingBehavior,
}

//		LocalPaths																
/// The local paths for non-baked-in resources.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct LocalPaths {
	//		Public properties													
	/// The path to the protected static assets.
	#[default = "content"]
	pub protected_assets: PathBuf,
	
	/// The path to the public static assets.
	#[default = "static"]
	pub public_assets:    PathBuf,
}

//		StaticFiles																
#[derive(Deserialize, Serialize, SmartDefault)]
/// The configuration options for serving static files.
pub struct StaticFiles {
	//		Public properties													
	/// The file size at which to start streaming, in KB. Below this size, the
	/// file will be read into memory and served all at once.
	#[default = 1000]
	pub stream_threshold: usize,
	
	/// The size of the stream buffer to use when streaming files, in KB.
	#[default = 256]
	pub stream_buffer:    usize,
	
	/// The size of the read buffer to use when streaming files, in KB.
	#[default = 128]
	pub read_buffer:      usize,
}

//		AppState																
/// The application state.
/// 
/// This is used to store global state information that is shared between
/// requests.
/// 
#[allow(dead_code)]
pub struct AppState {
	//		Public properties													
	/// The application configuration.
	pub Config:   Config,
	
	/// The application statistics.
	pub Stats:    AppStats,
	
	/// The application secret.
	pub Secret:   [u8; 64],
	
	/// The HMAC key used to sign and verify sessions.
	pub Key:      hmac::Key,
	
	/// The Tera template engine.
	pub Template: Tera,
}

//		AppStats																
/// Various application statistics.
#[derive(SmartDefault)]
pub struct AppStats {
	//		Public properties													
	/// The date and time the application was started.
	pub started_at: NaiveDateTime,
	
	/// The number of requests that have been handled.
	pub requests:   AtomicUsize,
	
	/// The number of responses that have been handled.
	pub responses:  AppStatsResponses,
}

//		AppStatsResponses														
/// Counts of response status codes.
#[derive(SmartDefault)]
pub struct AppStatsResponses {
	//		Public properties													
	/// The counts of responses.
	#[default(AppStatsResponseCounts::new())]
	pub counts: AppStatsResponseCounts,
}

//		AppStatsResponseCounts													
/// Counts of response status codes.
#[derive(SmartDefault)]
pub struct AppStatsResponseCounts {
	//		Public properties													
	/// The total number of responses that have been handled.
	pub total:     AtomicUsize,
	
	/// The number of responses that have been handled, by status code.
	pub codes:     HashMap<StatusCode, AtomicUsize>,
	
	/// The number of untracked responses that have been handled, i.e. where the
	/// code does not match any of the ones in this struct.
	pub untracked: AtomicUsize,
}

impl AppStatsResponseCounts {
	//		new																	
	/// Creates a new instance of the struct.
	pub fn new() -> Self {
		Self {
			total:     AtomicUsize::new(0),
			codes:     hash_map!{
				StatusCode::OK:                    AtomicUsize::new(0),
				StatusCode::UNAUTHORIZED:          AtomicUsize::new(0),
				StatusCode::NOT_FOUND:             AtomicUsize::new(0),
				StatusCode::INTERNAL_SERVER_ERROR: AtomicUsize::new(0),
			},
			untracked: AtomicUsize::new(0),
		}
	}
}

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		handlers::get_ping,
		handlers::get_stats,
	),
	components(
		schemas(handlers::StatsResponse),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	)
)]
pub struct ApiDoc;



//		Functions

//		extract_uri_query_parts													
/// Extracts the query parts from a URI.
/// 
/// Extracts the query parts of a [`Uri`] and returns them as a [`HashMap`].
/// 
/// # Parameters
/// 
/// * `uri` - The URI to extract the query parts from.
/// 
pub fn extract_uri_query_parts(uri: Uri) -> HashMap<String, String> {
	uri
		.query()
		.map(|v| {
			form_urlencoded::parse(v.as_bytes())
				.into_owned()
				.collect()
		})
		.unwrap_or_else(HashMap::new)
}

//		build_uri																
/// Builds a URI from a path and a set of query parameters.
/// 
/// # Parameters
/// 
/// * `path`   - The path to build the URI from.
/// * `params` - The query parameters to add to the URI.
/// 
pub fn build_uri(path: String, params: HashMap<String, String>) -> Uri {
	Uri::builder()
		.path_and_query(format!("{}?{}",
			path,
			params
				.iter()
				.map(|(k, v)| format!("{}={}", k, v))
				.collect::<Vec<String>>()
				.join("&")
		))
		.build()
		.unwrap()
}

//		serialize_status_codes													
/// Returns a list of serialised status code entries and their values.
/// 
/// This function is used by [`serde`] to serialise a list of status codes and
/// their associated values. It returns the list sorted by status code.
/// 
/// # Parameters
/// 
/// * `status_codes` - The status codes to serialise, as keys, against values.
/// * `serializer`   - The serialiser to use.
/// 
pub fn serialize_status_codes<S>(status_codes: &HashMap<StatusCode, u64>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let codes: BTreeMap<String, u64> = status_codes
		.iter()
		.map(|(key, value)| (key.to_string(), *value))
		.collect()
	;
	codes.serialize(serializer)
}


