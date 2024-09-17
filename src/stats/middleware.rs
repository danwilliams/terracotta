//! Statistics middleware.



//		Packages

use super::{
	state::StatsStateProvider,
	worker::{Endpoint, ResponseMetrics},
};
use axum::{
	Extension,
	async_trait,
	body::Body,
	extract::{FromRequestParts, State},
	http::{Request, request::Parts},
	middleware::Next,
	response::Response,
};
use chrono::{NaiveDateTime, Utc};
use core::{
	convert::Infallible,
	sync::atomic::Ordering,
};
use smart_default::SmartDefault;
use std::sync::Arc;
use tikv_jemalloc_ctl::stats::allocated as Malloc;
use tracing::error;



//		Structs

//		StatsContext															
/// The statistics context.
/// 
/// This struct contains statistics information specific to the current request.
/// 
#[derive(Clone, SmartDefault)]
pub struct StatsContext {
	//		Public properties													
	/// The date and time the request processing started.
	#[default(Utc::now().naive_utc())]
	pub started_at: NaiveDateTime,
}

//󰭅		FromRequestParts														
#[async_trait]
impl<State> FromRequestParts<State> for StatsContext
where State: Send + Sync {
	type Rejection = Infallible;
	
	//		from_request_parts													
	/// Creates a statistics context from the request parts.
	/// 
	/// # Parameters
	/// 
	/// * `parts` - The request parts.
	/// * `state` - The application state.
	/// 
	#[expect(clippy::expect_used, reason = "Misconfiguration, so hard quit")]
	async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
		let Extension(stats_cx): Extension<Self> =
			Extension::from_request_parts(parts, state)
				.await
				.expect("Stats extension/layer missing")
		;
		Ok(stats_cx)
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
/// * `appstate` - The application state.
/// * `request`  - The request.
/// * `next`     - The next middleware.
/// 
pub async fn stats_layer<S: StatsStateProvider>(
	State(appstate): State<Arc<S>>,
	mut request:     Request<Body>,
	next:            Next,
) -> Response {
	//	Create statistics context
	let stats_cx = StatsContext::default();
	_ = request.extensions_mut().insert(stats_cx.clone());
	
	//	Check if statistics are enabled
	if !appstate.stats_config().enabled {
		return next.run(request).await;
	}
	
	//	Obtain endpoint details
	let endpoint = Endpoint {
		path:      request.uri().path().to_owned(),
		method:    request.method().clone(),
	};
	
	//	Update requests counter
	let stats_state = appstate.stats_state().read().await;
	_ = stats_state.data.requests.fetch_add(1, Ordering::Relaxed);
	_ = stats_state.data.connections.fetch_add(1, Ordering::Relaxed);
	
	//	Process request
	let response = next.run(request).await;
	
	//	Add response time to the queue
	if let Some(ref queue) = stats_state.queue {
		#[expect(clippy::arithmetic_side_effects, reason = "Nothing interesting can happen here")]
		#[expect(clippy::cast_sign_loss,          reason = "We don't ever want a negative for time taken")]
		drop(queue.send(ResponseMetrics {
			endpoint,
			started_at:  stats_cx.started_at,
			time_taken:  (Utc::now().naive_utc() - stats_cx.started_at).num_microseconds().unwrap() as u64,
			status_code: response.status(),
			connections: stats_state.data.connections.load(Ordering::Relaxed) as u64,
			memory:	     Malloc::read().unwrap() as u64,
		}).inspect_err(|err| error!("Failed to send response time: {err}")));
	}
	
	_ = stats_state.data.connections.fetch_sub(1, Ordering::Relaxed);
	drop(stats_state);
	
	//	Return response
	response
}


