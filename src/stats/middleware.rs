#![allow(clippy::unused_async, reason = "Middleware functions need to be async")]

//! Statistics middleware.



//		Packages

use super::{
	state::StateProvider,
	worker::{Endpoint, ResponseMetrics},
};
use axum::{
	Extension,
	async_trait,
	body::Body,
	extract::{FromRequestParts, State, rejection::ExtensionRejection},
	http::{Request, request::Parts},
	middleware::Next,
	response::Response,
};
use chrono::{NaiveDateTime, Utc};
use core::sync::atomic::Ordering;
use smart_default::SmartDefault;
use std::sync::Arc;
use tikv_jemalloc_ctl::stats::allocated as Malloc;
use tracing::{error, warn};



//		Structs

//		Context																	
/// The statistics context.
/// 
/// This struct contains statistics information specific to the current request.
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, SmartDefault)]
pub struct Context {
	//		Public properties													
	/// The date and time the request processing started.
	#[default(Utc::now().naive_utc())]
	pub started_at: NaiveDateTime,
}

//󰭅		FromRequestParts														
#[async_trait]
impl<S> FromRequestParts<S> for Context
where
	S: Send + Sync,
{
	type Rejection = ExtensionRejection;
	
	//		from_request_parts													
	/// Creates a statistics context from the request parts.
	/// 
	/// # Parameters
	/// 
	/// * `parts` - The request parts.
	/// * `state` - The application state.
	/// 
	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		Extension::<Self>::from_request_parts(parts, state).await.map(|Extension(stats_cx)| stats_cx)
	}
}



//		Functions

//		stats_layer																
/// A middleware to collect statistics about requests and responses.
/// 
/// This middleware sits in the request-response chain and collects statistics
/// about requests and responses, storing them in the application state.
/// 
/// # Parameters
/// 
/// * `state`   - The application state.
/// * `request` - The request.
/// * `next`    - The next middleware.
/// 
pub async fn stats_layer<SP: StateProvider>(
	State(state): State<Arc<SP>>,
	mut request:     Request<Body>,
	next:            Next,
) -> Response {
	//	Create statistics context
	let stats_cx = Context::default();
	_ = request.extensions_mut().insert(stats_cx);
	
	//	Check if statistics are enabled
	if !state.config().enabled {
		return next.run(request).await;
	}
	
	//	Obtain endpoint details
	let endpoint = Endpoint {
		path:      request.uri().path().to_owned(),
		method:    request.method().clone(),
	};
	
	//	Update requests counter
	let stats_state = state.state().read().await;
	_ = stats_state.data.requests.fetch_add(1, Ordering::Relaxed);
	_ = stats_state.data.connections.fetch_add(1, Ordering::Relaxed);
	
	//	Process request
	let response = next.run(request).await;
	
	//	Add response time to the queue
	if let Some(ref queue) = stats_state.queue {
		#[expect(clippy::cast_sign_loss, reason = "We don't ever want a negative for time taken")]
		drop(queue.send_async(ResponseMetrics {
			endpoint,
			started_at:  stats_cx.started_at,
			time_taken:  Utc::now()
				.naive_utc()
				.signed_duration_since(stats_cx.started_at)
				.num_microseconds()
				.unwrap_or(i64::MAX) as u64
			,
			status_code: response.status(),
			connections: stats_state.data.connections.load(Ordering::Relaxed) as u64,
			memory:	     Malloc::read()
				.inspect_err(|err| warn!("Could not read memory usage: {err}"))
				.unwrap_or_default() as u64
			,
		}).await.inspect_err(|err| error!("Failed to send response time: {err}")));
	}
	
	_ = stats_state.data.connections.fetch_sub(1, Ordering::Relaxed);
	drop(stats_state);
	
	//	Return response
	response
}


