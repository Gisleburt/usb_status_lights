use status_lights_cli::Client;

use status_lights_messages::{VersionNumber, LedColor, LedColorTimed};
use structopt::StructOpt;

#[derive(Copy, Clone, Debug, StructOpt)]
struct BackgroundOptions {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    device: Option<u8>,
}

impl Into<LedColor> for BackgroundOptions {
    fn into(self) -> LedColor {
        LedColor {
            led: self.led,
            red: self.red,
            green: self.green,
            blue: self.blue,
        }
    }
}

#[derive(Copy, Clone, Debug, StructOpt)]
struct ForegroundOptions {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    seconds: Option<u8>,
    device: Option<u8>,
}

impl Into<LedColorTimed> for ForegroundOptions {
    fn into(self) -> LedColorTimed {
        LedColorTimed {
            led: self.led,
            red: self.red,
            green: self.green,
            blue: self.blue,
            seconds: self.seconds.unwrap_or(0),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "status_lights", about = "Control status lights")]
enum Opt {
    List,
    Background(BackgroundOptions),
    Foreground(ForegroundOptions),
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let mut clients = Client::collect_clients().unwrap();

    match opt {
        Opt::List => print_devices_addresses(clients),
        Opt::Background(background_options) => {
            clients.iter_mut().for_each(|client| {
                client.request_background(background_options.into()).unwrap();
            })
        }
        Opt::Foreground(foreground_options) => {
            clients.iter_mut().for_each(|client| {
                client.request_foreground(foreground_options.into()).unwrap();
            })
        }
        _ => {}
    }
}

fn format_version_number(version: &VersionNumber) -> String {
    format!("v{}.{}.{}", version.major, version.minor, version.patch)
}

fn print_devices_addresses(mut clients: Vec<Client>) {
    println!("Found {} devices", clients.len());
    clients.iter_mut().for_each(|client| {
        if let Ok(version_number) = client.request_version() {
            println!(
                "{}, {}, {}",
                client.get_path(),
                client.get_name(),
                format_version_number(&version_number)
            )
        }
    });
}
