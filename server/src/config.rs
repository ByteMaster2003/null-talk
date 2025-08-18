use config::{Config, File};
use serde::Deserialize;
use std::{env, error::Error, path::PathBuf};

/// Configuration for the server
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Hostname or IP address of the server
    pub host: String,

    /// Port number for the server  
    pub port: u16,
}

impl ServerConfig {
    /// Load the server configuration from a file
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let root_dir: PathBuf = env::current_dir()?;
        let mut config_path: PathBuf = root_dir.join("Config.toml");

        if !config_path.exists() {
            config_path = PathBuf::from("/etc/irc-server/Config.toml");
        }

        if !config_path.exists() {
            return Err(format!("No configuration file found!").into());
        }

        let file = File::with_name(&config_path.to_str().unwrap());
        let cfg = Config::builder().add_source(file).build().unwrap();

        let svr_cfg = match cfg.try_deserialize::<ServerConfig>() {
            Ok(map) => map,
            Err(err) => {
                eprintln!("Failed to deserialize configuration: {:?}", err);
                return Err(err.into());
            }
        };

        Ok(svr_cfg)
    }

    /// Get the server address as a string
    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
