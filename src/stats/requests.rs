//! Request data for statistics functionality.



//		Packages

use chrono::NaiveDateTime;
use core::str::FromStr;
use serde::Deserialize;

#[cfg(feature = "utoipa")]
use utoipa::{IntoParams, ToSchema};



//		Enums

//		MeasurementType															
/// The type of measurement to get statistics for.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MeasurementType {
	/// Response times.
	Times,
	
	/// Active connections.
	Connections,
	
	/// Memory usage.
	Memory,
}

//󰭅		FromStr																	
impl FromStr for MeasurementType {
	type Err = ();
	
	//		from_str															
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"times"       => Ok(Self::Times),
			"connections" => Ok(Self::Connections),
			"memory"      => Ok(Self::Memory),
			_             => Err(()),
		}
	}
}



//		Structs

//		GetStatsHistoryParams													
/// The parameters for the [`get_stats_history()`](super::handlers::get_stats_history())
/// handler.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(IntoParams))]
#[non_exhaustive]
pub struct GetStatsHistoryParams {
	//		Public properties													
	/// The buffer to get the statistics for. The buffer items are returned in
	/// order of most-recent first.
	pub buffer: Option<MeasurementType>,
	
	/// The date and time to get the statistics from. This will apply from the
	/// given point in time until now, i.e. the check is, "is the time of the
	/// response item newer than or equal to the given time?". The expected
	/// format is `YYYY-MM-DDTHH:MM:SS`, e.g. `2023-10-18T06:08:34`.
	pub from:   Option<NaiveDateTime>,
	
	/// The number of buffer entries, i.e. the number of seconds, to get the
	/// statistics for. This will apply from now backwards, i.e. the count will
	/// start with the most-recent item and return up to the given number of
	/// items. If used with [`GetStatsHistoryParams::from`], this may seem
	/// somewhat counter-intuitive, as the item identified by that parameter may
	/// not be included in the results, but the items closest to the current
	/// time are the ones of most interest, and so asking for a maximum number
	/// of items is most likely to mean the X most-recent items rather than the
	/// X oldest items. Because the most-recent items are always returned first,
	/// the [`last_second`](super::responses::StatsResponse::last_second) /
	/// [`last_second`](super::responses::StatsHistoryResponse::last_second)
	/// property of the response will always be the time of the first item in
	/// the list.
	pub limit:  Option<usize>,
}

//		GetStatsFeedParams														
/// The parameters for the [`get_stats_feed()`](super::handlers::get_stats_feed())
/// handler.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(IntoParams))]
#[non_exhaustive]
pub struct GetStatsFeedParams {
	//		Public properties													
	/// The type of measurement to subscribe to statistics for.
	pub r#type: Option<MeasurementType>,
}


