use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DensityMode {
    Standard,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentPreset {
    Blue,
    Teal,
    Indigo,
    Amber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontChoice {
    FzGwKai,
    MicrosoftYaHei,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme_mode: ThemeMode,
    pub density_mode: DensityMode,
    pub accent_preset: AccentPreset,
    pub font_choice: FontChoice,
    pub close_to_tray: bool,
    pub start_minimized_on_autostart: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Light,
            density_mode: DensityMode::Standard,
            accent_preset: AccentPreset::Blue,
            font_choice: FontChoice::FzGwKai,
            close_to_tray: true,
            start_minimized_on_autostart: true,
        }
    }
}

pub fn settings_path() -> PathBuf {
    if let Ok(custom) = std::env::var("UISHOWCASE_CONFIG_PATH")
        && !custom.trim().is_empty()
    {
        return PathBuf::from(custom);
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("澄明界面展示").join("settings.json")
}

pub fn load_settings() -> (AppSettings, Option<String>) {
    let path = settings_path();
    if !path.exists() {
        return (AppSettings::default(), None);
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
            Ok(s) => (s, None),
            Err(err) => (
                AppSettings::default(),
                Some(format!("配置文件格式异常，已回退默认设置（{err}）。")),
            ),
        },
        Err(err) => (
            AppSettings::default(),
            Some(format!("配置文件读取失败，已回退默认设置（{err}）。")),
        ),
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败：{e}"))?;
    }
    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化配置失败：{e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("写入配置文件失败：{e}"))
}
