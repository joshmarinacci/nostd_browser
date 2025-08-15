#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::{vec};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{BusTimeout, Config, I2c};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Blocking;
use esp_hal::peripherals::Peripherals;
use gt911::Gt911Blocking;
use log::{error, info};

use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder};
use nostd_browser::common::{TDeckDisplay, TDeckDisplayWrapper};
use static_cell::StaticCell;

#[panic_handler]
fn panic(nfo: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {:?}", nfo);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

struct DrawContext {

}
struct LayoutContext {

}
struct View {
    name:String,
    children: Vec<String>,
    bounds: Rectangle,
}
impl View {
    fn draw(&self, ctx:&mut DrawContext) {

    }
    fn layout(&mut self, ctx:&LayoutContext, children:&mut Vec<&mut View>) {
        self.bounds = Rectangle::new(Point::new(10,10), Size::new(100,100));
        for child in children.iter_mut() {
            child.bounds.top_left.x = self.bounds.top_left.x + 5;
        }
    }
}

struct Scene {
    root: String,
    focused: Option<String>,
    views: Vec<View>
}

impl Scene {
    fn find_view(&self, name:&str) -> Option<View> {
        todo!()
    }
    fn find_view_mut(&self, name:&str) -> Option<&mut View> {
        todo!()
    }
    fn draw(&self) {
        let mut ctx = DrawContext {
        };
        self.draw_child(&mut ctx, &self.root)
    }
    fn draw_child(&self, ctx:&mut DrawContext, name:&str) {
        if let Some(view) = &self.find_view(name) {
            view.draw(ctx);
            for child in &view.children {
                self.draw_child(ctx,child)
            }
        }
    }
    fn layout(&self) {
        let mut ctx = LayoutContext {
        };
        self.layout_child(&mut ctx, &self.root)
    }
    fn layout_child(&self, ctx:&LayoutContext, name:&str) {
        if let Some(view) = self.find_view_mut(name) {
            let mut children:Vec<&mut View> = vec![];
            for ch in &view.children {
                let ch_view = self.find_view_mut(ch).unwrap();
                children.push(ch_view);
            }
            view.layout(ctx, &mut children);
            for child in &view.children {
                self.layout_child(ctx,child)
            }
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    info!("init-ting embassy");
    esp_hal_embassy::init(TimerGroup::new(peripherals.TIMG1).timer0);

    let delay = Delay::new();
    // have to turn on the board and wait 500ms before using the keyboard
    let mut board_power = Output::new(peripherals.GPIO10, High, OutputConfig::default());
    board_power.set_high();
    delay.delay_millis(1000);

}
