#![no_std]
#![no_main]

use core::cell::RefCell;

use epd_waveshare::{
    epd2in13_v2::{Display2in13 as EpdBuffer, Epd2in13 as EpdDisplay},
    prelude::WaveshareDisplay as _,
};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{self, Io},
    peripherals::Peripherals,
    prelude::*,
    rtc_cntl::Rtc,
    spi::{master::Spi, SpiMode},
    system::SystemControl,
};

mod draw;

extern crate alloc;

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(32 * 1024);

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let cs = gpio::Output::new(io.pins.gpio15, gpio::Level::High);
    let busy = gpio::Input::new(io.pins.gpio25, gpio::Pull::None);
    let rst = gpio::Output::new(io.pins.gpio26, gpio::Level::High);
    let dc = gpio::Output::new(io.pins.gpio27, gpio::Level::Low);

    let spi = RefCell::new(
        Spi::new(peripherals.SPI2, 8.MHz(), SpiMode::Mode0, &clocks).with_pins(
            Some(io.pins.gpio13), // sclk
            Some(io.pins.gpio14), // mosi
            gpio::NO_PIN,
            gpio::NO_PIN,
        ),
    );

    let mut spi_bus = embedded_hal_bus::spi::RefCellDevice::new(&spi, cs, delay).unwrap();

    let mut epd = EpdDisplay::new(&mut spi_bus, busy, dc, rst, &mut delay, None)
        .expect("EPaper should be present");

    let mut display = EpdBuffer::default();

    draw::clear_display(&mut display);
    draw::text_to_display(&mut display, "Hello");

    epd.update_and_display_frame(&mut spi_bus, display.buffer(), &mut delay)
        .expect("EPaper should accept update/display requests");

    epd.sleep(&mut spi_bus, &mut delay)
        .expect("EPaper should never fail to sleep");

    Rtc::new(peripherals.LPWR, None).sleep_deep(&[], &mut delay)
}
