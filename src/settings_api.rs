use std::sync::Arc;

use crate::{model::{JsonReply, json_reply}, settings};

pub async fn get_settings(
    settings: Arc<settings::StrichlisteSetting>,
) -> Result<JsonReply<settings::SettingsResp>, warp::Rejection> {
    let result = settings::SettingsResp {
        settings: settings.as_ref().clone(),
    };

    Ok(json_reply(result))
}
