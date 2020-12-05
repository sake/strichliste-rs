use std::sync::Arc;

use crate::{model::{JsonReply, json_reply}, settings};

pub async fn get_settings(
    settings: Arc<settings::StrichlisteSetting>,
) -> Result<JsonReply<settings::SettingsWrapper>, warp::Rejection> {
    let result = settings::SettingsWrapper {
        parameters: settings::Settings {
            strichliste: settings.as_ref().clone(),
        },
    };

    Ok(json_reply(result))
}
