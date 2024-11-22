use std::error;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub pd: PDConfig,
    pub audio: AudioConfig,
    pub shout: ShoutConfig,
}

#[derive(Deserialize)]
pub struct PDConfig {
    pub patch: PathBuf,
}

#[derive(Deserialize)]
pub struct AudioConfig {
    #[serde(default = "audio_channels_default")]
    pub channels: i32,
    #[serde(default = "audio_samplerate_default")]
    pub samplerate: i32,
}

fn audio_channels_default() -> i32 {
    2
}

fn audio_samplerate_default() -> i32 {
    44100
}

#[derive(Deserialize)]
pub struct ShoutConfig {
    pub host: String,
    #[serde(default = "shout_port_default")]
    pub port: u16,
    pub user: String,
    pub password: String,
    pub mount: String,
}

fn shout_port_default() -> u16 {
    8000
}
pub fn read(path: PathBuf) -> Result<Config, Box<dyn error::Error>> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let config: Config = toml::from_str(&data)?;
    return Ok(config);
}
