use status_lights_cli::get_status_light_devices;

fn main() {
    let devices = get_status_light_devices();
    for device in devices {
        println!("{:?}", device);
    }
}
