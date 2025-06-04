//! Utility functions and types for the application.



//		Packages																										

#[cfg(feature = "tera")]
use super::{
	config::LoadingBehavior,
	errors::AppError,
	state::StateProvider,
};
#[cfg(feature = "tera")]
use ::{
	tera::Context,
	tokio::fs,
};



//		Functions																										

//		render																	
/// Renders a template.
/// 
/// Renders a template with the given context and returns the result.
/// 
/// If the application has been configured to allow template overrides, the
/// local filesystem will be searched, and any matching templates found will be
/// used in preference to the baked-in ones.
/// 
/// # Parameters
/// 
/// * `state`    - The application state.
/// * `template` - The name of the template to render.
/// * `context`  - The context to render the template with.
/// 
/// # Errors
/// 
/// If the template cannot be loaded or rendered, an error will be returned.
/// 
#[cfg(feature = "tera")]
pub async fn render<SP, T>(
	state:    &SP,
	template: T,
	context:  &Context,
) -> Result<String, AppError>
where
	SP: StateProvider,
	T:  AsRef<str> + Send,
{
	let local_template = state.html_templates_config().local_path.join(format!("{}.tera.html", template.as_ref()));
	let local_layout   = state.html_templates_config().local_path.join("layout.tera.html");
	Ok(if state.html_templates_config().behavior == LoadingBehavior::Override {
		let mut tera   = state.tera().clone();
		if local_layout.exists() {
			tera.add_raw_template(
				"layout",
				&fs::read_to_string(&local_layout).await
					.map_err(|err| AppError::CouldNotLoadTemplate(local_layout.clone(), err))?
			).map_err(|err| AppError::CouldNotAddTemplate(local_layout, err))?;
		}
		if local_template.exists() {
			tera.add_raw_template(
				template.as_ref(),
				&fs::read_to_string(&local_template).await
					.map_err(|err| AppError::CouldNotLoadTemplate(local_template.clone(), err))?
			).map_err(|err| AppError::CouldNotAddTemplate(local_template, err))?;
		}
		tera.render(template.as_ref(), context)?
	} else {
		state.tera().render(template.as_ref(), context)?
	})
}


