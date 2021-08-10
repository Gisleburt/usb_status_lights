use serialport::{SerialPort, SerialPortType, UsbPortInfo};
use status_lights_messages::{Message, VersionNumber, DEVICE_MANUFACTURER, DEVICE_PRODUCT};
use std::convert::TryFrom;
use std::time::Duration;
use std::io::{Write, Read};

const USB_TIMEOUT: Duration = Duration::from_secs(5);

type ClientResult<T> = Result<T, ()>; // ToDo: Use a real error

#[derive(Debug)]
pub struct AvailableDevice {
    path: String,
    name: String,
}

impl TryFrom<AvailableDevice> for Client {
    type Error = ();

    fn try_from(device: AvailableDevice) -> Result<Self, Self::Error> {
        let serial = serialport::new(&device.path, 9600)
            .timeout(USB_TIMEOUT)
            .open()
            .unwrap();
        Ok(Client { serial, device })
    }
}

pub struct Client {
    serial: Box<dyn SerialPort>,
    device: AvailableDevice,
}

fn is_known_device(port_info: &UsbPortInfo) -> bool {
    port_info.manufacturer.as_deref() == Some(DEVICE_MANUFACTURER)
        && port_info.product.as_deref() == Some(DEVICE_PRODUCT)
}

impl Client {
    pub fn get_clients() -> ClientResult<Vec<Client>> {
        Ok(Self::get_available_devices()?
            .into_iter()
            .filter_map(|device| Client::try_from(device).ok())
            .collect())
    }

    pub fn get_path(&self) -> &str {
        self.device.path.as_str()
    }

    pub fn get_name(&self) -> &str {
        self.device.name.as_str()
    }

    fn get_available_devices() -> ClientResult<Vec<AvailableDevice>> {
        let available_devices = serialport::available_ports()
            .unwrap()
            .into_iter()
            .map(|port| (port.port_name, port.port_type))
            .filter_map(|(path, port_type)| {
                if let SerialPortType::UsbPort(port_info) = port_type {
                    if is_known_device(&port_info) {
                        if let Some(name) = port_info.serial_number {
                            return Some(AvailableDevice { path, name });
                        }
                    }
                }
                None
            })
            .collect();
        Ok(available_devices)
    }

    pub fn request_version(&mut self) -> ClientResult<VersionNumber> {
        let request = Message::VersionRequest;
        self.serial.write_all(&request.to_bytes()).unwrap();
        let mut buf = [0; 8];
        self.serial.read_exact(&mut buf).unwrap();
        if let Ok(message) = Message::try_from(buf) {
            if let Message::VersionResponse(version_number) = message {
                Ok(version_number)
            } else {
                panic!("Message not a version response: {:?}", message)
            }
        } else {
            panic!("Message unreadable")
        }
    }
}
