use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;
use serde::Deserialize;

/*
Service responsible for managing the application
startup process.
 */
#[derive(Deserialize)]
struct Config {
    applications: HashMap<String, String>,
}

pub struct ApplicationService {
    config: Config,
}

impl ApplicationService {
    /*
    Constructor : create config hashmap
     */
    pub fn new(config_path: &str) -> Self {
        let content = fs::read_to_string(config_path).expect("Unable to read apps.toml");
        let config: Config = toml::from_str(&content).expect("apps.toml invalid");

        Self {
            config,
        }
    }

    /*
    Open application
     */
    pub fn open(&self, app_name: &str) -> String {
        match self.config.applications.get(app_name) {
            Some(env_var) => {
                let path = match env::var(env_var) {
                    Ok(p) => p,
                    Err(_) => return format!("{} missing is .env", env_var),
                };

                match Command::new(path).spawn() {
                    Ok(_) => format!("{} successfully launched !", app_name),
                    Err(_) => format!("Launch failure of {}.", app_name),
                }
            },
            None => String::from("Application not found."),
        }
    }
}