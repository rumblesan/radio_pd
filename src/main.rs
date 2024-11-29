mod broadcast;
mod config;
mod http;
mod osc;
mod pdradio_error;
use crate::pdradio_error::PDRadioError;

use std::env;
use std::error;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::PathBuf;

use clap::Parser;

use libpd_rs::convenience::{calculate_ticks, PdGlobal};
use libpd_rs::receive::{on_print, receive_messages_from_pd};
use vorbis_rs::VorbisEncoderBuilder;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

fn run(cli: CliArgs) -> Result<(), Box<dyn error::Error>> {
    let cwd = env::current_dir()?;

    let config = config::read(cwd.join(cli.config))?;

    if !matches!(config.shout.format, config::ShoutFormat::Ogg) {
        return Err(Box::new(PDRadioError(
            "Only support OGG format for shoutcast currently".into(),
        )));
    }

    let mut pd = PdGlobal::init_and_configure(0, config.audio.channels, config.audio.samplerate)?;

    let osc_coms_handler = osc::create_osc_listener(config.osc);

    let http_coms_handler = http::create_http_listener(config.http);

    pd.open_patch(cwd.join(config.pd.patch))?;

    on_print(|value| {
        println!("{value}");
    });

    let mut conn_builder = shout::ShoutConnBuilder::new()
        .host(config.shout.host)
        .port(config.shout.port)
        .user(config.shout.user)
        .password(config.shout.password)
        .mount(config.shout.mount)
        .protocol(config.shout.protocol.into())
        .format(config.shout.format.into());

    for metadata in config.metadata.to_shout_metadata() {
        conn_builder = conn_builder.add_meta(metadata);
    }

    let conn = conn_builder
        .build()
        .map_err(|e| -> PDRadioError { e.into() })?;

    let conn_sink = broadcast::ShoutConnWriter(conn);
    println!("Connected to server");

    let samplerate: NonZeroU32 = u32::try_from(config.audio.samplerate)
        .and_then(|v| NonZeroU32::try_from(v))
        .map_err(|err| PDRadioError(format!("Error setting encoder samplerate: {err}").into()))?;
    let output_channels: NonZeroU8 = u8::try_from(config.audio.channels)
        .and_then(|v| NonZeroU8::try_from(v))
        .map_err(|err| PDRadioError(format!("Error setting encoder channels: {err}").into()))?;

    let mut encoder_builder = VorbisEncoderBuilder::new(samplerate, output_channels, conn_sink)?;
    let mut vencoder = encoder_builder.build()?;

    pd.activate_audio(true)?;

    let mut left_samps = vec![0.0; config.audio.blocksize];
    let mut right_samps = vec![0.0; config.audio.blocksize];
    let mut pd_output = vec![0.0; config.audio.blocksize * 2];
    loop {
        let ticks = calculate_ticks(2, pd_output.len() as i32);
        receive_messages_from_pd();
        libpd_rs::process::process_float(ticks, &[], &mut pd_output);
        for i in 0..config.audio.blocksize {
            left_samps[i] = pd_output[i * 2];
            right_samps[i] = pd_output[(i * 2) + 1];
        }
        if vencoder
            .encode_audio_block([&left_samps, &right_samps])
            .map_err(|err| {
                println!("{err}");
                err
            })
            .is_err()
        {
            break;
        }
    }
    println!("Finished!");

    osc_coms_handler.map(|h| h.join());
    http_coms_handler.map(|h| h.join());

    pd.activate_audio(false)?;

    pd.close_patch()?;

    Ok(())
}

fn main() {
    let args = CliArgs::parse();

    if let Err(e) = run(args) {
        println!("{}", e); // "There is an error: Oops"
    }
}
