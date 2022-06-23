#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use neo_trinkey as bsp;
use panic_halt as _;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use bsp::entry;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::timer::TimerCounter;
use hal::usb::UsbBus;

use smart_leds::SmartLedsWrite;
use status_lights_messages::{
    ErrorResponse, Request, Response, VersionNumber, DEVICE_MANUFACTURER, DEVICE_PRODUCT,
};
use ws2812_timer_delay::Ws2812;

use crate::led::{Color, ColorTimed};
use core::convert::TryFrom;

mod led;

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
const LOOP_WAIT: u32 = 500;
const NUM_LEDS: usize = 4;
static mut LED_BACKGROUND: [Color; NUM_LEDS] = [
    Color::default(),
    Color::default(),
    Color::default(),
    Color::default(),
];
static mut LED_FOREGROUND: [ColorTimed; NUM_LEDS] = [
    ColorTimed::default(),
    ColorTimed::default(),
    ColorTimed::default(),
    ColorTimed::default(),
];

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let pins = bsp::Pins::new(peripherals.PORT);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.PM);
    timer.start(3.mhz());
    let neo_pixel = pins.neo_pixel.into_push_pull_output();
    let mut ws2812 = Ws2812::new(timer, neo_pixel);

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(bsp::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            pins.usb_dm,
            pins.usb_dp,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x0, 0x0))
                .manufacturer(DEVICE_MANUFACTURER)
                .product(DEVICE_PRODUCT)
                .serial_number("Gisleburt Neo Trinkey Status Lights")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    loop {
        unsafe {
            let leds = [
                LED_FOREGROUND[0]
                    .to_rgb()
                    .or_else(|| LED_BACKGROUND[0].to_rgb())
                    .unwrap_or_default(),
                LED_FOREGROUND[1]
                    .to_rgb()
                    .or_else(|| LED_BACKGROUND[1].to_rgb())
                    .unwrap_or_default(),
                LED_FOREGROUND[2]
                    .to_rgb()
                    .or_else(|| LED_BACKGROUND[2].to_rgb())
                    .unwrap_or_default(),
                LED_FOREGROUND[3]
                    .to_rgb()
                    .or_else(|| LED_BACKGROUND[3].to_rgb())
                    .unwrap_or_default(),
            ];
            ws2812.write(leds.iter().cloned()).unwrap();
            LED_FOREGROUND
                .iter_mut()
                .for_each(|fg| fg.reduce_time(LOOP_WAIT));
        }

        delay.delay_ms(LOOP_WAIT);
    }
}

fn create_version_number_response() -> Response {
    Response::Version(VersionNumber {
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
    })
}

fn poll_usb() {
    unsafe {
        if let Some(usb_dev) = USB_BUS.as_mut() {
            if let Some(serial) = USB_SERIAL.as_mut() {
                usb_dev.poll(&mut [serial]);
                let mut buf = [0u8; 8];

                if let Ok(_count) = serial.read(&mut buf) {
                    // ToDo: Check count
                    let message = Request::try_from(buf);
                    match message {
                        Ok(Request::Version) => {
                            let response = create_version_number_response();
                            serial.write(&response.to_bytes()).ok();
                        }
                        Ok(Request::Background(led_color)) => {
                            LED_BACKGROUND[led_color.led as usize] = led_color.into();
                            let response = Response::Background;
                            serial.write(&response.to_bytes()).ok();
                        }
                        Ok(Request::Foreground(led_color_timed)) => {
                            LED_FOREGROUND[led_color_timed.led as usize] = led_color_timed.into();
                            let response = Response::Foreground;
                            serial.write(&response.to_bytes()).ok();
                        }
                        Err(error) => {
                            let response: Response =
                                ErrorResponse::UnknownRequestId(error.get_id()).into();
                            serial.write(&response.to_bytes()).ok();
                        }
                        _ => {}
                    }
                };
            };
        };
    };
}

#[interrupt]
fn USB() {
    poll_usb();
}
