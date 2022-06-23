use status_lights_cli::{Client, ClientError};
use status_lights_messages::{LedColor, LedColorTimed, VersionNumber};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
struct BackgroundOptions {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    #[structopt(long)]
    device: Option<String>,
}

impl From<BackgroundOptions> for LedColor {
    fn from(bg: BackgroundOptions) -> Self {
        Self {
            led: bg.led,
            red: bg.red,
            green: bg.green,
            blue: bg.blue,
        }
    }
}

#[derive(Clone, Debug, StructOpt)]
struct ForegroundOptions {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    seconds: Option<u8>,
    #[structopt(long)]
    device: Option<String>,
}

impl From<ForegroundOptions> for LedColorTimed {
    fn from(fg: ForegroundOptions) -> Self {
        Self {
            led: fg.led,
            red: fg.red,
            green: fg.green,
            blue: fg.blue,
            seconds: fg.seconds.unwrap_or(0),
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

impl Opt {
    pub fn get_device(&self) -> Option<&String> {
        match self {
            Opt::List => None,
            Opt::Background(bg) => bg.device.as_ref(),
            Opt::Foreground(fg) => fg.device.as_ref(),
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    let device = opt.get_device();

    let mut clients: Vec<Client> = Client::collect_clients().unwrap()
        .into_iter()
        .filter(move |c| device.is_none() || Some(c.get_path()) == device)
        .collect();

    if clients.is_empty() {
        eprintln!("No devices found");
        std::process::exit(-1);
    }

    match opt {
        Opt::List => print_devices_addresses(clients),
        Opt::Background(background_options) => {
            let results = set_background(&mut clients, background_options);
            handle_results_and_exit(results);
        }
        Opt::Foreground(foreground_options) => {
            let results = set_foreground(&mut clients, foreground_options);
            handle_results_and_exit(results);
        }
    }
}

fn set_background(clients: &mut [Client], background_options: BackgroundOptions) -> Vec<Result<(), ClientError>> {
    clients
        .iter_mut()
        .map(|client| {
            println!("Changing device '{}' at '{}'", client.get_name(), client.get_path());
            client.request_background(background_options.clone().into())
        })
        .collect()
}

fn set_foreground(clients: &mut [Client], foreground_options: ForegroundOptions) -> Vec<Result<(), ClientError>> {
    clients
        .iter_mut()
        .map(|client| {
            println!("Changing device '{}' at '{}'", client.get_name(), client.get_path());
            client.request_foreground(foreground_options.clone().into())
        })
        .collect()
}

fn handle_results_and_exit<T>(results: Vec<Result<T, ClientError>>) {
    results
        .iter()
        .for_each(|r| {
            if let Err(e) = r.as_ref() {
                eprintln!("Error: {:?}", e)
            }
        });

    std::process::exit(results.len() as i32);
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
