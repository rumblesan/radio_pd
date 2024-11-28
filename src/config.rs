use std::error;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use shout;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub pd: PDConfig,
    pub audio: AudioConfig,
    pub shout: ShoutConfig,
    pub metadata: MetadataConfig,
    pub osc: OSCConfig,
    pub http: HTTPConfig,
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
pub enum ShoutProtocol {
    HTTP,
    XAudioCast,
    Icy,
    RoarAudio,
}

impl From<ShoutProtocol> for shout::ShoutProtocol {
    fn from(protocol: ShoutProtocol) -> Self {
        match protocol {
            ShoutProtocol::HTTP => shout::ShoutProtocol::HTTP,
            ShoutProtocol::XAudioCast => shout::ShoutProtocol::XAudioCast,
            ShoutProtocol::Icy => shout::ShoutProtocol::Icy,
            ShoutProtocol::RoarAudio => shout::ShoutProtocol::RoarAudio,
        }
    }
}

#[derive(Deserialize)]
pub enum ShoutFormat {
    Ogg,
    MP3,
    Webm,
    WebmAudio,
}

impl From<ShoutFormat> for shout::ShoutFormat {
    fn from(format: ShoutFormat) -> Self {
        match format {
            ShoutFormat::Ogg => shout::ShoutFormat::Ogg,
            ShoutFormat::MP3 => shout::ShoutFormat::MP3,
            ShoutFormat::Webm => shout::ShoutFormat::Webm,
            ShoutFormat::WebmAudio => shout::ShoutFormat::WebmAudio,
        }
    }
}

#[derive(Deserialize)]
pub struct ShoutConfig {
    pub host: String,
    #[serde(default = "shout_port_default")]
    pub port: u16,
    pub user: String,
    pub password: String,
    pub mount: String,
    #[serde(default = "shout_protocol_default")]
    pub protocol: ShoutProtocol,
    #[serde(default = "shout_format_default")]
    pub format: ShoutFormat,
}

fn shout_port_default() -> u16 {
    8000
}

fn shout_protocol_default() -> ShoutProtocol {
    ShoutProtocol::HTTP
}

fn shout_format_default() -> ShoutFormat {
    ShoutFormat::Ogg
}

#[derive(Deserialize)]
pub struct OSCConfig {
    #[serde(default = "osc_listen_default")]
    pub listen: bool,
    #[serde(default = "osc_host_default")]
    pub host: String,
    #[serde(default = "osc_port_default")]
    pub port: String,
}

fn osc_listen_default() -> bool {
    false
}

fn osc_host_default() -> String {
    "0.0.0.0".to_owned()
}

fn osc_port_default() -> String {
    "8080".to_owned()
}

#[derive(Deserialize)]
pub struct HTTPConfig {
    #[serde(default = "http_listen_default")]
    pub listen: bool,
    #[serde(default = "http_host_default")]
    pub host: String,
    #[serde(default = "http_port_default")]
    pub port: String,
}

fn http_listen_default() -> bool {
    true
}

fn http_host_default() -> String {
    "0.0.0.0".to_owned()
}

fn http_port_default() -> String {
    "9001".to_owned()
}

#[derive(Deserialize)]
pub struct MetadataConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub url: Option<String>,
}

impl MetadataConfig {
    pub fn to_shout_metadata(&self) -> Vec<shout::ShoutMeta> {
        vec![
            self.name
                .as_ref()
                .map(|name| shout::ShoutMeta::Name(name.to_string())),
            self.description
                .as_ref()
                .map(|description| shout::ShoutMeta::Description(description.to_string())),
            self.genre
                .as_ref()
                .map(|genre| shout::ShoutMeta::Genre(genre.to_string())),
            self.url
                .as_ref()
                .map(|url| shout::ShoutMeta::Url(url.to_string())),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

pub fn read(path: PathBuf) -> Result<Config, Box<dyn error::Error>> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let config: Config = toml::from_str(&data)?;
    return Ok(config);
}
