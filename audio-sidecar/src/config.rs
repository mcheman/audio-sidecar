use std::str::FromStr;
use config::{Config, FileFormat};

pub struct ProgramConfig {
    pub interface: String, // search string for the audio interface to use
    pub window_width: u32,
    pub window_height: u32,
    pub log_file: String,
    pub log_level: String,
    pub existing_file_strategy: ExistingFileStrategy,
}

// todo should we be wrapping the config like this? It seems like the underlying Config is meant to be used directly to allow hot-reloading
impl ProgramConfig {
    pub fn from_file() -> Result<ProgramConfig, String> {
        let settings = Config::builder()
            .add_source(config::File::new(
                "./audio-sidecar-config",
                FileFormat::Toml,
            ))
            .build()
            .unwrap(); // todo this should have defaults and not panic if config file doesn't exist

        let interface: String = settings.get("Interface").unwrap_or(String::from(""));
        let window_width: u32 = settings.get("WindowWidth").unwrap_or(1200);
        let window_height: u32 = settings.get("WindowHeight").unwrap_or(600);
        let log_file: String = settings
            .get("LogFile")
            .unwrap_or(String::from("audioSidecar.log"));
        let log_level: String = settings.get("LogLevel").unwrap_or(String::from("debug"));

        let existing_file_strategy = ExistingFileStrategy::from_str(
            settings
                .get("ExistingFileStrategy")
                .unwrap_or(String::from(""))
                .as_str(),
        )
        .unwrap_or(ExistingFileStrategy::RenameToLast);

        Ok(ProgramConfig {
            interface,
            window_width,
            window_height,
            log_file,
            log_level,
            existing_file_strategy,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum ExistingFileStrategy {
    RenameToLast,
    RenameToFirst,
    Append,
    Replace,
    Ask,
}

impl FromStr for ExistingFileStrategy {
    type Err = ();
    fn from_str(s: &str) -> Result<ExistingFileStrategy, ()> {
        match s {
            "rename-to-last" => Ok(ExistingFileStrategy::RenameToLast),
            "rename-to-first" => Ok(ExistingFileStrategy::RenameToFirst),
            "append" => Ok(ExistingFileStrategy::Append),
            "replace" => Ok(ExistingFileStrategy::Replace),
            "ask" => Ok(ExistingFileStrategy::Ask),
            _ => Err(()),
        }
    }
}