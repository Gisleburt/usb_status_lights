use serialport::{SerialPort, SerialPortInfo, SerialPortType, UsbPortInfo};
use status_lights_messages::{
    LedColor, LedColorTimed, Request, Response, ResponseError, VersionNumber, DEVICE_MANUFACTURER,
    DEVICE_PRODUCT,
};
use thiserror::Error;

use std::convert::TryFrom;
use std::io::{Read, Write};
use std::time::Duration;

const USB_TIMEOUT: Duration = Duration::from_secs(5);

type ClientResult<T> = Result<T, ClientError>; // ToDo: Use a real error

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error Received from device")]
    ErrorReceivedFromDevice(ResponseError),
    #[error("No Response from device: {0}")]
    NoResponseReceived(String),
    #[error("Unable to write to device: {0}")]
    DeviceWriteError(String),
    #[error("Unable to read from device: {0}")]
    DeviceReadError(String),
    // ToDo: Make better use of SerialPorts error type.
    #[error("Serial device error")]
    GeneralSerialError,
}

impl From<ResponseError> for ClientError {
    fn from(error: ResponseError) -> Self {
        Self::ErrorReceivedFromDevice(error)
    }
}

impl From<serialport::Error> for ClientError {
    fn from(_error: serialport::Error) -> Self {
        Self::GeneralSerialError
    }
}

#[derive(Debug)]
pub struct AvailableDevice {
    path: String,
    name: String,
}

impl TryFrom<AvailableDevice> for Client {
    type Error = serialport::Error;

    fn try_from(device: AvailableDevice) -> Result<Self, Self::Error> {
        let serial = serialport::new(&device.path, 9600)
            .timeout(USB_TIMEOUT)
            .open()?;
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
    pub fn collect_clients() -> ClientResult<Vec<Client>> {
        Ok(Self::collect_available_devices()?
            .into_iter()
            .filter_map(|device| Client::try_from(device).ok())
            .collect())
    }

    pub fn get_path(&self) -> &String {
        &self.device.path
    }

    pub fn get_name(&self) -> &str {
        self.device.name.as_str()
    }

    pub fn list_all_usb_devices() -> ClientResult<Vec<SerialPortInfo>> {
        Ok(serialport::available_ports()?)
    }

    fn collect_available_devices() -> ClientResult<Vec<AvailableDevice>> {
        let available_devices = serialport::available_ports()?
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

    fn send(&mut self, request: &Request) -> ClientResult<Response> {
        self.serial
            .write_all(&request.to_bytes())
            .map_err(|_| ClientError::DeviceWriteError(self.device.path.clone()))?;
        let mut buf = [0; 8];
        self.serial
            .read_exact(&mut buf)
            .map_err(|_| ClientError::DeviceWriteError(self.device.path.clone()))?;
        let message = Response::try_from(buf)?;
        Ok(message)
    }

    pub fn request_version(&mut self) -> ClientResult<VersionNumber> {
        let request = Request::Version;
        let message = self.send(&request)?;
        if let Response::Version(version_number) = message {
            Ok(version_number)
        } else {
            panic!("Message not a version response: {:?}", message)
        }
    }

    pub fn request_background(&mut self, led_color: LedColor) -> ClientResult<()> {
        let request = Request::Background(led_color);
        let message = self.send(&request)?;
        if message == Response::Background {
            Ok(())
        } else {
            panic!("Message not a version response: {:?}", message)
        }
    }

    pub fn request_foreground(&mut self, led_color_timed: LedColorTimed) -> ClientResult<()> {
        let request = Request::Foreground(led_color_timed);
        let message = self.send(&request)?;
        if message == Response::Foreground {
            Ok(())
        } else {
            panic!("Message not a version response: {:?}", message)
        }
    }
}
