use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::thread;

use crate::config;

use rosc::{OscPacket, OscType};

use libpd_rs::send::send_list_to;
use libpd_rs::types::Atom;

pub fn osc_type_to_atom(osc_type: OscType) -> Option<Atom> {
    match osc_type {
        OscType::Int(v) => Some(Atom::Float(v.into())),
        OscType::Float(v) => Some(Atom::Float(v.into())),
        OscType::String(v) => Some(Atom::Symbol(v.into())),
        OscType::Blob(..) => None,
        OscType::Time(..) => None,
        OscType::Long(..) => None,
        OscType::Double(v) => Some(Atom::Float(v.into())),
        _ => None,
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
                .map(|el| osc_type_to_atom(el.clone()))
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

pub fn create_osc_listener(config: config::OSCConfig) -> Option<thread::JoinHandle<()>> {
    if !config.listen {
        println!("Not listening for OSC messages");
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
