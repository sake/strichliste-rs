use crate::common;
use log::error;
use serde::{Deserialize, Serialize};
use std::fs::File;

pub fn load_settings(
    settings_env: &str,
    settings_file_default: &str,
) -> Result<StrichlisteSetting, std::io::Error> {
    let settings_file = common::env_or(settings_env, settings_file_default);
    let file = File::open(&settings_file)?;

    return match serde_yaml::from_reader::<_, SettingsWrapper>(file) {
        Ok(v) => Ok(v.parameters.strichliste),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse YAML data: {}", e),
        )),
    };
}

pub fn get_stale_period(settings: &StrichlisteSetting) -> i64 {
    // seconds until user is counted as inactive
    let stale_val = settings.user.stale_period.as_str();
    let stale_period = match ms_converter::ms(stale_val) {
        Ok(v) => v * 1000,
        Err(err) => {
            error!(
                "Error evaluating stale, value. Using a 10 days default.\n  {}",
                err
            );
            // 10 days
            10 * 24 * 3600 * 1000
        }
    };
    return stale_period;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SettingsWrapper {
    pub parameters: Settings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub strichliste: StrichlisteSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StrichlisteSetting {
    pub article: ArticleSettings,
    pub common: CommonSettings,
    pub paypal: PaypalSetting,
    pub user: UserSetting,
    pub i18n: I18nSetting,
    pub account: AccountSetting,
    pub payment: PaymentSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleSettings {
    pub enabled: bool,
    #[serde(rename(serialize = "autoOpen", deserialize = "autoOpen"))]
    pub auto_open: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommonSettings {
    #[serde(rename(serialize = "idleTimeout", deserialize = "idleTimeout"))]
    pub idle_timeout: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaypalSetting {
    pub enabled: bool,
    pub recipient: String,
    pub fee: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSetting {
    #[serde(rename(serialize = "stalePeriod", deserialize = "stalePeriod"))]
    pub stale_period: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct I18nSetting {
    #[serde(rename(serialize = "dateFormat", deserialize = "dateFormat"))]
    pub date_format: String,
    pub timezone: String,
    pub language: String,
    pub currency: CurrencySetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrencySetting {
    pub name: String,
    pub symbol: String,
    pub alpha3: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountSetting {
    pub boundary: BoundarySetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoundarySetting {
    pub upper: i32,
    pub lower: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentSetting {
    pub undo: UndoSetting,
    pub boundary: BoundarySetting,
    pub transactions: TransactionSetting,
    #[serde(rename(serialize = "splitInvoice", deserialize = "splitInvoice"))]
    pub split_invoice: SplitInvoiceSetting,
    pub deposit: DepositSetting,
    pub dispense: DepositSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UndoSetting {
    pub enabled: bool,
    pub delete: bool,
    pub timeout: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionSetting {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SplitInvoiceSetting {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DepositSetting {
    pub enabled: bool,
    pub custom: bool,
    pub steps: Vec<i32>,
}
