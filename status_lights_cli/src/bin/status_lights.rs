use status_lights_cli::get_status_light_devices;

use structopt::StructOpt;
use rusb::{GlobalContext, Device};

#[derive(Debug, StructOpt)]
struct BackgroundOptions {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    device: Option<u8>,
}

#[derive(Debug, StructOpt)]
struct ForegroundOptions {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    seconds: Option<u8>,
    device: Option<u8>,
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

    let devices = get_status_light_devices();

    match opt {
        Opt::List => print_devices_addresses(devices),
        _ => {}
    }
}

fn print_devices_addresses(devices: Vec<Device<GlobalContext>>) {
    // Get just the addresses and sort them
    let mut addresses = devices.iter()
        .map(|d| d.address())
        .collect::<Vec<u8>>();
    addresses.sort();

    // Print the output
    print!("Found {} devices", addresses.len());
    if let Some((last, rest)) = addresses.split_last() {
        print!(" at addresses: ");
        for address in rest {
            print!("{}, ", address);
        }
        print!("{}", last);
    }
    println!();
}

