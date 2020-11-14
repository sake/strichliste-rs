use std::sync::Arc;

use crate::settings;

pub async fn get_settings(
    settings: Arc<settings::StrichlisteSetting>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let result = settings::SettingsWrapper {
        parameters: settings::Settings {
            strichliste: settings.as_ref().clone(),
        },
    };
    return Ok(Box::new(warp::reply::json(&result)));
}
