use directories::ProjectDirs;
use std::path::PathBuf;
use convert_case::{Case, Casing};
use itertools::Itertools;

use serde::de::DeserializeOwned;

pub struct ConfigurationLoaderBuilder {
    loader: ConfigurationLoader,
}

impl ConfigurationLoaderBuilder {

    pub fn new(qualifier: &str, organization: &str, application: &str)
               -> ConfigurationLoaderBuilder {
        return ConfigurationLoaderBuilder {
            loader: ConfigurationLoader {
                qualifier: qualifier.to_string(),
                organization: organization.to_string(),
                application: application.to_string(),
                base_name: "config".to_string(),
                search_paths: ConfigurationLoader::default_search_paths(
                    qualifier.to_string(), organization.to_string(), application.to_string()
                ),
                file_suffixes: ConfigurationLoader::default_file_suffixes()
            },
        }
    }

    pub fn build(&mut self) -> ConfigurationLoader {
        return self.loader.clone();
    }

    fn load_active_profiles_from_env(var_name: &str) -> Vec<String> {
        match std::env::var(var_name) {
            Ok(value) => {
                let result: Vec<String> = value.split(",").map(|v| v.trim().to_string()).collect_vec();
                return result;
            },
            Err(_err) => {
                return vec![];
            }
        }
    }

    pub fn base_name(&mut self, value: &str) -> &mut Self {
        self.loader.base_name = value.to_string();
        return self;
    }

    pub fn search_paths(&mut self, value: Vec<String>) -> &mut Self {
        self.loader.search_paths = value;
        return self;
    }

    pub fn file_suffixes(&mut self, value: Vec<String>) -> &mut Self {
        self.loader.file_suffixes = value;
        return self;
    }
}

#[derive(Clone, Debug)]
pub struct ConfigurationLoader {
    qualifier: String,
    organization: String,
    application: String,
    base_name: String,
    search_paths: Vec<String>,
    file_suffixes: Vec<String>,
}

impl ConfigurationLoader {

    pub fn default_file_suffixes() -> Vec<String> {
        return vec![".yaml".to_string(), ".json".to_string(), ".toml".to_string(), ".properties".to_string()];
    }

    pub fn default_search_paths(qualifier: String, organization: String, application: String)
                                -> Vec<String> {
        let mut paths: Vec<String> = vec![];

        let project_conf_path = ProjectDirs::from(qualifier.as_str(), organization.as_str(), application.as_str())
            .map(|dirs| dirs.config_dir().to_path_buf());

        if project_conf_path.is_some() {
            paths.push(String::from(project_conf_path.unwrap().to_str().unwrap()));
        }

        let cwd = std::env::current_dir();
        match cwd {
            Ok(path) => {
                paths.push(path.clone().join("etc").to_str().unwrap().to_string());
                paths.push(path.clone().join("conf").to_str().unwrap().to_string());
            },
            Err(_err) => {
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "solaris", target_os = "illumos", target_os = "freebsd", target_os = "openbsd"))]
        {
            paths.push(format!("/etc/{}", application.clone()));
            paths.push(format!("/etc/{}", application.clone().to_lowercase()));
            paths.push(format!("/etc/{}", application.clone().as_str().to_case(Case::Kebab)));
            paths.push(format!("/etc/{}", application.clone().as_str().to_case(Case::Snake)));
        }

        return paths;
    }

    pub fn load_configuration<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        for search_path in self.search_paths.iter().rev() {
            match self.load_configuration_for_path(search_path.as_str()) {
                Ok(result) => {
                    return Ok(result);
                },
                Err(_err) => {
                    continue;
                }
            }
        }
        return Err(anyhow::Error::msg("No configuration file found."));
    }

    fn load_configuration_for_path<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        println!("looking for config file in path: {}", path);
        for suffix in &self.file_suffixes {
            let result = self.load_configuration_for_suffix(path, suffix.as_str());
            match result {
                Ok(result) => {
                    return Ok(result)
                },
                Err(_err) => {
                    continue;
                }
            };
        }
        return Err(anyhow::Error::msg(format!("No configuration file found for path \"{}\"", path)));
    }

    fn load_configuration_for_suffix<T: DeserializeOwned>(&self, path: &str, suffix: &str) -> anyhow::Result<T> {
        let filename = format!("{}{}",
                               self.base_name,
                               suffix
        );

        let path = std::path::PathBuf::new().join(path).join(filename);
        return Self::try_load_single_configuration(&path, suffix);
    }

    fn try_load_single_configuration<T: DeserializeOwned>(path: &PathBuf, suffix: &str) -> anyhow::Result<T> {
        match suffix.to_lowercase().as_str() {
            ".json" | ".yaml" => {
                let content = ConfigurationLoader::try_read_content(path)?;
                let result: anyhow::Result<T> = serde_yaml::from_str(content.as_str())
                    .or_else(|err| Err(anyhow::Error::from(err)));
                return result;
            },
            ".toml" => {
                let content = ConfigurationLoader::try_read_content(path)?;
                let result: anyhow::Result<T> = toml::from_str(content.as_str())
                    .or_else(|err| Err(anyhow::Error::from(err)));
                return result;
            },
            other => {
                return Err(anyhow::Error::msg(format!("Cannot read config files with suffix {}", other)));
            }
        }
    }

    fn try_read_content(path: &PathBuf) -> anyhow::Result<String> {
        match std::fs::read_to_string(path) {
            Ok(content) => return Ok(content),
            Err(err) => return Err(anyhow::Error::from(err))
        };
    }
}