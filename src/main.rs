use std::{env, path::{PathBuf, Path}};

use anyhow::{anyhow, Result};
use clap::Parser;
use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// config file path
    #[arg(short, long, default_value = "config.toml")]
    config_path: String,
}

#[derive(Debug, Clone,  Deserialize, Serialize)]
struct Client {
    #[serde(rename = "webPrefix")]
    web_prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct YamlConfig {
    client: Client,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileConfig {
    old_web_prefix: String,
    new_web_prefix: Option<String>,
    local_config_path: Option<String>,
    config_path: Option<String>,
    exclude_path: Option<Vec<String>>,
    source_path: String,
    output_path: String,
    file_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Setting {
    old_web_prefix: String,
    new_web_prefix: String,
    exclude_path: Option<Vec<String>>,
    source_path: String,
    output_path: String,
    file_types: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let setting = init_config().await?;
    map_files(setting).await?;
    Ok(())
}

async fn init_config() -> Result<Setting> {
    let args = Args::parse();
    let Args { config_path } = args;
    let config_path = env::current_exe()?.join(config_path);

    if !config_path.exists() {
        return Err(anyhow!("config file not exists"));
    }

    let setting: FileConfig = Config::builder()
        .add_source(config::File::from(config_path))
        .build()?
        .try_deserialize()?;

    map_file_config(setting)
}

async fn map_files(setting: Setting) -> Result<()> {
    let Setting {
        source_path,
        file_types,
        exclude_path,
        old_web_prefix,
        new_web_prefix,
        output_path,
    } = setting;
    let file_types = file_types.unwrap_or(vec![]);
    let mut exclude_path = exclude_path.unwrap_or(vec![]);
    exclude_path.push("\\.git$".to_owned());
    let path = PathBuf::from(&source_path);
    if !path.exists() {
        return Err(anyhow!("dir not exists"));
    }
    let walker = walkdir::WalkDir::new(&source_path)
        .into_iter()
        .filter_entry(|e| {
            let relative_path = e.path().strip_prefix(&source_path).unwrap();
            !exclude_path.iter().any(|p| {
                regex::Regex::new(p)
                    .unwrap()
                    .is_match(relative_path.to_str().unwrap_or_default())
            })
        });
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.is_file();
        if !is_file {
            continue;
        }
        let file_type = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        if !file_types.contains(&file_type) {
            continue;
        }
        let content = tokio::fs::read_to_string(path).await?;
        if !content.contains(&old_web_prefix) {
            continue;
        }
        let new_content = content.replace(&old_web_prefix, &new_web_prefix);
        let path = path.strip_prefix(&source_path)?;
        let out_dir = PathBuf::from(&output_path);
        let path = out_dir.join(path);
        let parent = path.parent().unwrap();
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, new_content).await?;
    }
    Ok(())
}


// FileConfig -> Setting
fn map_file_config(file_config: FileConfig) -> Result<Setting> {
    let FileConfig {
        old_web_prefix,
        new_web_prefix,
        local_config_path,
        config_path,
        exclude_path,
        source_path,
        output_path,
        file_types,
    } = file_config;

    if let Some(new_web_prefix) = new_web_prefix {
        return Ok(Setting {
                old_web_prefix,
                new_web_prefix,
                exclude_path,
                source_path,
                output_path,
                file_types,
            });
    }
    
    let is_empty = local_config_path.is_none();
    let mut local_config = local_config_path.map(|path| {
        get_yaml_config(&path)
    }).transpose();
    
    if is_empty || local_config.is_err() {
        local_config = config_path.map(|path| {
            get_yaml_config(&path)
        }).transpose();
    }

    let default_config = YamlConfig {
        client: Client {
            web_prefix: "".to_owned(),
        }
    };
   let yaml_config  =  local_config.unwrap_or(None).unwrap_or(default_config);
   let web_prefix = yaml_config.client.web_prefix;
     Ok(Setting {
        old_web_prefix,
        new_web_prefix: if web_prefix == "/" { "".to_owned() } else { web_prefix },
        exclude_path,
        source_path,
        output_path,
        file_types,
    })
}
   

fn get_yaml_config<P>(path: &P) -> Result<YamlConfig> where P: AsRef<Path> + ?Sized  {
    let path = path.as_ref();
    let setting = Config::builder()
        .add_source(config::File::from(path))
        .build()?
        .try_deserialize()?;
    Ok(setting)
}

