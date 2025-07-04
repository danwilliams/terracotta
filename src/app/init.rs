//! Functionality to prepare the application.



//		Packages																										

use super::errors::AppError;
use figment::{
	Figment,
	providers::{Env, Format as _, Serialized, Toml},
};
use std::io::stdout;
use serde::{Serialize, de::DeserializeOwned};
use tracing::Level;
use tracing_appender::{self, non_blocking, non_blocking::WorkerGuard, rolling::daily};
use tracing_subscriber::{
	EnvFilter,
	fmt::{layer, writer::MakeWriterExt as _},
	layer::SubscriberExt as _,
	registry,
	util::SubscriberInitExt as _,
};

#[cfg(feature = "tera")]
use ::{
	include_dir::Dir,
	std::sync::Arc,
	tera::Tera,
};



//		Functions																										

//		load_config																
/// Loads the application configuration.
/// 
/// This function loads the configuration from the `Config.toml` file and the
/// environment variables.
/// 
/// # Errors
/// 
/// If there is a problem loading the configuration, or if the configuration is
/// invalid, an error will be returned.
/// 
#[expect(clippy::result_large_err, reason = "Size doesn't matter here as it's only for config")]
pub fn load_config<T>() -> Result<T, AppError>
where
	T: Default + DeserializeOwned + Serialize,
{
	Ok(Figment::from(Serialized::defaults(T::default()))
		.merge(Toml::file("Config.toml"))
		.merge(Env::raw())
		.extract()?)
}

//		setup_logging															
/// Sets up logging for the application.
/// 
/// This function sets up logging to the terminal and to a file in the specified
/// directory.
/// 
/// # Parameters
/// 
/// * `logdir` - The directory to write the log files to.
/// 
pub fn setup_logging<S: AsRef<str>>(logdir: S) -> WorkerGuard {
	let (non_blocking_appender, guard) = non_blocking(
		daily(logdir.as_ref(), "general.log")
	);
	registry()
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| format!(
					"info,{}=debug,terracotta=debug,tower_http=debug",
					env!("CARGO_PKG_NAME"),
				).into()),
		)
		.with(
			layer()
				.with_writer(stdout.with_max_level(Level::DEBUG))
		)
		.with(
			layer()
				.with_writer(non_blocking_appender.with_max_level(Level::INFO))
		)
		.init()
	;
	guard
}

//		setup_tera																
/// Sets up the Tera template engine.
/// 
/// This function reads all the `.tera.html` files in the specified directory
/// and adds them to the Tera template engine.
/// 
/// # Parameters
/// 
/// * `template_dir` - The directory containing the HTML templates. This is
///                    wrapped inside an [`Arc`] to support reusability across
///                    the application if required.
/// 
/// # Errors
/// 
/// If there is a problem reading the template files, or if there is an error
/// parsing the template, an error will be returned.
/// 
#[cfg(feature = "tera")]
#[expect(clippy::result_large_err, reason = "Size doesn't matter here as it's only for config")]
pub fn setup_tera(template_dir: &Arc<Dir<'static>>) -> Result<Tera, AppError> {
	let templates = template_dir
		.find("**/*.tera.html")?
		.map(|file| {
			Ok((
				file
					.path()
					.file_name()
					.and_then(|f| f.to_str())
					.and_then(|f| f.strip_suffix(".tera.html"))
					.ok_or_else(|| AppError::InvalidTemplatePath(file.path().to_path_buf()))?
					.to_owned()
				,
				template_dir
					.get_file(file.path())
					.ok_or_else(|| AppError::TemplateFileNotFound(file.path().to_path_buf()))?
					.contents_utf8()
					.ok_or_else(|| AppError::InvalidTemplateEncoding(file.path().to_path_buf()))?
				,
			))
		}).collect::<Result<Vec<_>, AppError>>()?
	;
	let mut tera = Tera::default();
	tera.add_raw_templates(templates)?;
	tera.autoescape_on(vec![".tera.html", ".html"]);
	Ok(tera)
}


