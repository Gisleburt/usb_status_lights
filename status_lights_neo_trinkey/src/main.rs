#![no_std]
#![no_main]

use neo_trinkey as bsp;
use cortex_m::peripheral::NVIC;
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

use smart_leds::{hsv::RGB8, SmartLedsWrite};
use ws2812_timer_delay::Ws2812;
use status_lights_messages::{DEVICE_PRODUCT, Request, VersionNumber, DEVICE_MANUFACTURER, Response};

use core::convert::TryFrom;

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
const NUM_LEDS: usize = 4;
static mut LED_BACKGROUND: [RGB8; NUM_LEDS] = [
    RGB8 { r: 0, b: 0, g: 0 },
    RGB8 { r: 0, b: 0, g: 0 },
    RGB8 { r: 0, b: 0, g: 0 },
    RGB8 { r: 0, b: 0, g: 0 },
];
static mut LED_FOREGROUND: [RGB8; NUM_LEDS] = [
    RGB8 { r: 0, b: 0, g: 0 },
    RGB8 { r: 0, b: 0, g: 0 },
    RGB8 { r: 0, b: 0, g: 0 },
    RGB8 { r: 0, b: 0, g: 0 },
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

    unsafe {
        LED_FOREGROUND = [
            RGB8::new(5, 5, 0),
            RGB8::new(0, 5, 5),
            RGB8::new(5, 0, 5),
            RGB8::new(2, 2, 2),
        ];
    }

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
        USB_SERIAL = Some(SerialPort::new(&bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x0, 0x0))
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
        unsafe { ws2812.write(LED_BACKGROUND.iter().cloned()).unwrap(); }
        delay.delay_ms(500u16);
        unsafe { ws2812.write(LED_FOREGROUND.iter().cloned()).unwrap(); }
        delay.delay_ms(500u16);
    }
}

fn create_version_number_response() -> Response {
    Response::VersionResponse(VersionNumber {
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
    })
}

fn poll_usb() {
    unsafe {
        USB_BUS.as_mut().map(|usb_dev| {
            USB_SERIAL.as_mut().map(|serial| {
                usb_dev.poll(&mut [serial]);
                let mut buf = [0u8; 8];

                if let Ok(_count) = serial.read(&mut buf) { // ToDo: Check count
                    let message = Request::try_from(buf);
                    match message {
                        Ok(Request::VersionRequest) => {
                            let response = create_version_number_response();
                            serial.write(&response.to_bytes()).ok();
                        }
                        _ => {

                        }
                    }
                };
            });
        });
    };
}

#[interrupt]
fn USB() {
    poll_usb();
}
