use crate::config;

use std::error::Error;
use std::fmt;
use std::thread;

use rouille::{Request, Response, Server};
use serde::Deserialize;

use libpd_rs::functions::send::send_list_to;
use libpd_rs::Atom;

pub fn create_http_listener(config: config::HTTPConfig) -> Option<thread::JoinHandle<()>> {
    if !config.listen {
        println!("Not listening for HTTP requests");
        return None;
    }

    println!("Starting HTTP listener");

    let http_addr = format!("{}:{}", config.host, config.port);

    let server = match Server::new(http_addr, move |request| {
        rouille::router!(request,
                    (GET) (/) => {
                        Response::text("OK")
                    },
                    (GET) (/health) => {
                        Response::text("OK")
                    },
                    (POST) (/pd/osc) => {
                        handle_pd_message_request(request)
                    },
                    _ => Response::empty_404(),

        )
    }) {
        Ok(s) => {
            println!("Listening on {:?}", s.server_addr());
            s
        }
        Err(err) => {
            println!("Error creating server: {}", err);
            return None;
        }
    };
    let (handle, ..) = server.stoppable();

    Some(handle)
}

#[derive(Debug, Clone)]
struct OSCMessageError;

impl fmt::Display for OSCMessageError {
    fn fmt(&self, ome: &mut fmt::Formatter) -> fmt::Result {
        write!(ome, "No value given for OSC message")
    }
}

impl Error for OSCMessageError {}

#[derive(Deserialize)]
struct PdOSCMessageBody {
    address: String,
    number_value: Option<f64>,
    string_value: Option<String>,
}

fn handle_pd_message_request(request: &Request) -> Response {
    let data: PdOSCMessageBody = rouille::try_or_400!(rouille::input::json_input(request));

    let mut pdlist = data
        .address
        .split("/")
        .map(|s| s.into())
        .collect::<Vec<Atom>>();

    let number_value = data.number_value.map(|v| Atom::Float(v));
    let string_value = data.string_value.map(|v| Atom::Symbol(v));

    let pdvalue = rouille::try_or_400!(match (number_value, string_value) {
        (Some(nv), ..) => Ok(nv),
        (None, Some(sv)) => Ok(sv),
        _ => Err(OSCMessageError),
    });

    pdlist.remove(0);
    pdlist.push(pdvalue);
    match send_list_to("host", &pdlist[..]) {
        Ok(..) => println!("Sent message to PD: {:?}", pdlist),
        Err(err) => println!("Error sending message to PD: {}", err),
    }
    Response::text("OK")
}
