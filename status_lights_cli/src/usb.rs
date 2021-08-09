use std::time::Duration;
use rusb::{DeviceHandle, GlobalContext, Language, Device, Error as RusbError};
use status_lights_messages::{DEVICE_MANUFACTURER, DEVICE_PRODUCT};

const USB_TIMEOUT: Duration = Duration::from_micros(500);

fn get_any_lang(handle: &DeviceHandle<GlobalContext>) -> Result<Option<Language>, RusbError> {
    handle.read_languages(USB_TIMEOUT)
        .map(|languages| languages.first().map(|r| r.clone()))
}


pub fn get_status_light_devices() -> Vec<Device<GlobalContext>> {
    rusb::devices().unwrap().iter()
        .filter_map(|device| {
            if let Ok(desc) = device.device_descriptor() {
                Some((device, desc))
            } else {
                None
            }
        })
        .filter_map(|(device, desc)| {
            if let Ok(handle) = device.open() {
                Some((device, desc, handle))
            } else {
                None
            }
        })
        .filter_map(|(device, desc, handle)| {
            if let Ok(Some(lang)) = get_any_lang(&handle) {
                Some((device, desc, handle, lang))
            } else {
                None
            }
        })
        .filter_map(|(device, desc, handle, lang)| {
            let manufacturer = handle.read_manufacturer_string(lang, &desc, USB_TIMEOUT);
            let product = handle.read_product_string(lang, &desc, USB_TIMEOUT);
            if manufacturer == Ok(DEVICE_MANUFACTURER.to_string()) && product == Ok(DEVICE_PRODUCT.to_string()) {
                Some(device)
            } else {
                None
            }
        })
        .collect()
}
