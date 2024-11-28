mod config;
mod osc;
mod pdradio_error;
use crate::pdradio_error::PDRadioError;

use std::env;
use std::error;
use std::io;
use std::io::{Error, Write};
use std::num::{NonZeroU32, NonZeroU8};
use std::path::PathBuf;
//use std::sync::mpsc::channel;
use std::str::FromStr;
use std::thread;

use clap::Parser;
use rosc::{OscPacket, OscType};
use std::net::{SocketAddrV4, UdpSocket};

use libpd_rs::convenience::{calculate_ticks, PdGlobal};
use libpd_rs::receive::{on_print, receive_messages_from_pd};
use libpd_rs::send::send_list_to;
use libpd_rs::types::Atom;
use shout::ShoutConn;
use vorbis_rs::VorbisEncoderBuilder;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

struct ShoutConnWriter(ShoutConn);

impl Write for ShoutConnWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.0.send(buf) {
            Ok(..) => {
                self.0.sync();
                return Ok(buf.len());
            }
            Err(..) => Err(Error::other("Error writing to Shoutcast Connection")),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.sync();
        Ok(())
    }
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            println!("OSC arguments: {:?}", msg.args);
            let mut pdlist = msg.addr.split("/").map(|s| s.into()).collect::<Vec<Atom>>();
            pdlist.remove(0);
            let msgargs: Vec<OscType> = msg.args;
            let pdargs = msgargs
                .iter()
                .map(|el| osc::osc_type_to_atom(el.clone()))
                .filter(|el| el.is_some())
                .flatten()
                .collect::<Vec<Atom>>();
            pdlist.extend(pdargs);
            println!("List to send to PD: {:?}", pdlist);
            match send_list_to("host", &pdlist[..]) {
                Ok(..) => println!("Sent message to PD: {:?}", pdlist),
                Err(err) => println!("Error sending message to PD: {}", err),
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}

fn create_osc_listener(config: config::OSCConfig) -> Option<thread::JoinHandle<()>> {
    if !config.listen {
        println!("Not listening for OSC messaged");
        return None;
    }

    println!("Starting OSC listener");

    let address = format!("{}:{}", config.host, config.port);
    let osc_addr = match SocketAddrV4::from_str(&address) {
        Ok(addr) => addr,
        Err(_) => panic!("{}", "error with addr"),
    };
    let sock = UdpSocket::bind(osc_addr).unwrap();
    println!("Listening to {}", osc_addr);

    let osc_handler = thread::spawn(move || {
        let mut buf = [0u8; rosc::decoder::MTU];

        loop {
            match sock.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    println!("Received packet with size {} from: {}", size, addr);
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    handle_packet(packet);
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            }
        }
    });

    Some(osc_handler)
}

fn run(cli: CliArgs) -> Result<(), Box<dyn error::Error>> {
    let cwd = env::current_dir()?;

    let config = config::read(cwd.join(cli.config))?;

    let mut pd = PdGlobal::init_and_configure(0, config.audio.channels, config.audio.samplerate)?;

    let osc_coms_handler = create_osc_listener(config.osc);

    pd.open_patch(cwd.join(config.pd.patch))?;

    on_print(|value| {
        println!("{value}");
    });

    if !matches!(config.shout.format, config::ShoutFormat::Ogg) {
        return Err(Box::new(PDRadioError(
            "Only support OGG format for shoutcast currently".into(),
        )));
    }

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

    let conn_sink = ShoutConnWriter(conn);
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

    const BLOCK_SIZE: usize = 4096;
    let mut left_samps: [f32; BLOCK_SIZE] = [0.0; BLOCK_SIZE];
    let mut right_samps: [f32; BLOCK_SIZE] = [0.0; BLOCK_SIZE];
    let mut pd_output: [f32; BLOCK_SIZE * 2] = [0.0; BLOCK_SIZE * 2];
    loop {
        let ticks = calculate_ticks(2, pd_output.len() as i32);
        receive_messages_from_pd();
        libpd_rs::process::process_float(ticks, &[], &mut pd_output);
        for i in 0..BLOCK_SIZE {
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
