#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::{vec};
use alloc::string::ToString;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
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
use nostd_browser::common::{
    TDeckDisplay,
};
use nostd_browser::gui::comps::{Button, Label, MenuView, Panel, TextInput};
use nostd_browser::gui::Scene;
use nostd_browser::gui::GuiEvent;
use static_cell::StaticCell;

#[panic_handler]
fn panic(nfo: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {:?}", nfo);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
// macro_rules! mk_static {
//     ($t:ty,$val:expr) => {{
//         static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
//         #[deny(unused_attributes)]
//         let x = STATIC_CELL.uninit().write(($val));
//         x
//     }};
// }

pub const LILYGO_KB_I2C_ADDRESS: u8 = 0x55;

static I2C: StaticCell<I2c<Blocking>> = StaticCell::new();

static TRACKBALL_CHANNEL: Channel<CriticalSectionRawMutex, GuiEvent, 2> = Channel::new();
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    info!("init-ting embassy");
    esp_hal_embassy::init(TimerGroup::new(peripherals.TIMG1).timer0);

    let mut delay = Delay::new();
    // have to turn on the board and wait 500ms before using the keyboard
    let mut board_power = Output::new(peripherals.GPIO10, High, OutputConfig::default());
    board_power.set_high();
    delay.delay_millis(1000);

    // set up the keyboard
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default()
            .with_frequency(Rate::from_khz(100))
            .with_timeout(BusTimeout::Disabled),
    )
        .unwrap()
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO8);
    info!("initialized I2C keyboard");
    let i2c_ref = I2C.init(i2c);

    // set up the display
    {
        // set TFT CS to high
        let mut tft_cs = Output::new(peripherals.GPIO12, High, OutputConfig::default());
        tft_cs.set_high();
        let tft_miso = Input::new(
            peripherals.GPIO38,
            InputConfig::default().with_pull(Pull::Up),
        );
        let tft_sck = peripherals.GPIO40;
        let tft_mosi = peripherals.GPIO41;
        let tft_dc = Output::new(peripherals.GPIO11, Low, OutputConfig::default());
        let mut tft_enable = Output::new(peripherals.GPIO42, High, OutputConfig::default());
        tft_enable.set_high();

        info!("creating spi device");
        let spi = Spi::new(
            peripherals.SPI2,
            SpiConfig::default()
                .with_mode(Mode::_3)
                .with_frequency(Rate::from_mhz(80)), // .with_mode(Mode::_0)
        )
            .unwrap()
            .with_sck(tft_sck)
            .with_miso(tft_miso)
            .with_mosi(tft_mosi);
        static DISPLAY_BUF: StaticCell<[u8; 512]> = StaticCell::new();
        let buffer = DISPLAY_BUF.init([0u8; 512]);

        info!("setting up the display");
        let spi_delay = Delay::new();
        let spi_device = ExclusiveDevice::new(spi, tft_cs, spi_delay).unwrap();
        let di = SpiInterface::new(spi_device, tft_dc, buffer);
        let display = Builder::new(ST7789, di)
            // .reset_pin(tft_enable)
            .display_size(240, 320)
            .invert_colors(ColorInversion::Inverted)
            .color_order(ColorOrder::Rgb)
            .orientation(Orientation::new().rotate(Rotation::Deg90))
            // .display_size(320,240)
            .init(&mut delay)
            .unwrap();
        static DISPLAY: StaticCell<TDeckDisplay> = StaticCell::new();
        let display_ref = DISPLAY.init(display);
        let scene = make_gui_scene().await;
        spawner
            .spawn(update_display(display_ref, i2c_ref, scene))
            .ok();
    }

    // setup trackball
    {
        let trackball_click = Input::new(
            peripherals.GPIO0,
            InputConfig::default().with_pull(Pull::Up),
        );
        // connect to the left and right trackball pins
        let trackball_right = Input::new(
            peripherals.GPIO2,
            InputConfig::default().with_pull(Pull::Up),
        );
        let trackball_left = Input::new(
            peripherals.GPIO1,
            InputConfig::default().with_pull(Pull::Up),
        );
        let trackball_up = Input::new(
            peripherals.GPIO3,
            InputConfig::default().with_pull(Pull::Up),
        );
        let trackball_down = Input::new(
            peripherals.GPIO15,
            InputConfig::default().with_pull(Pull::Up),
        );
        spawner
            .spawn(handle_trackball(
                trackball_click,
                trackball_left,
                trackball_right,
                trackball_up,
                trackball_down,
            ))
            .ok();
    }
}
#[embassy_executor::task]
async fn update_display(
    display: &'static mut TDeckDisplay,
    i2c: &'static mut I2c<'static, Blocking>,
    mut scene: Scene,
) {
    let touch = Gt911Blocking::default();
    touch.init(i2c).unwrap();
    loop {
        if let Ok(points) = touch.get_touch(i2c) {
            // stack allocated Vec containing 0-5 points
            info!("{:?}", points)
        }
        let mut data = [0u8; 1];
        let kb_res = (*i2c).read(LILYGO_KB_I2C_ADDRESS, &mut data);
        match kb_res {
            Ok(_) => {
                if data[0] != 0x00 {
                    let evt: GuiEvent = GuiEvent::KeyEvent(data[0]);
                    handle_input(evt, &mut scene, display).await;
                }
            }
            Err(_) => {
                // info!("kb_res = {}", e);
            }
        }

        if let Ok(evt) = TRACKBALL_CHANNEL.try_receive() {
            handle_input(evt, &mut scene, display).await;
        }

        scene.draw(display);
        Timer::after(Duration::from_millis(20)).await;
    }
}
#[embassy_executor::task]
async fn handle_trackball(
    click: Input<'static>,
    left: Input<'static>,
    right: Input<'static>,
    up: Input<'static>,
    down: Input<'static>,
) {
    let mut last_click_low = false;
    let mut last_right_high = false;
    let mut last_left_high = false;
    let mut last_up_high = false;
    let mut last_down_high = false;
    info!("monitoring the trackball");
    let mut cursor = Point::new(50, 50);
    loop {
        if click.is_low() != last_click_low {
            info!("click");
            last_click_low = click.is_low();
            TRACKBALL_CHANNEL.send(GuiEvent::ClickEvent()).await;
        }
        // info!("button pressed is {} ", tdeck_track_click.is_low());
        if right.is_high() != last_right_high {
            // info!("right");
            last_right_high = right.is_high();
            cursor.x += 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(1, 0)))
                .await;
        }
        if left.is_high() != last_left_high {
            // info!("left");
            last_left_high = left.is_high();
            cursor.x -= 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(-1, 0)))
                .await;
        }
        if up.is_high() != last_up_high {
            // info!("up");
            last_up_high = up.is_high();
            cursor.y -= 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(0, -1)))
                .await;
        }
        if down.is_high() != last_down_high {
            // info!("down");
            last_down_high = down.is_high();
            cursor.y += 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(0, 1)))
                .await;
        }
        // wait one msec
        Timer::after(Duration::from_millis(1)).await;
    }
}


const TEXT_INPUT: &str = "textinput";
async fn handle_input(event: GuiEvent, scene: &mut Scene, display: &mut TDeckDisplay) {
    info!("handling input event: {:?}", event);
    if scene.get_focused_view().is_none() {
    scene.set_focused(TEXT_INPUT);
        }
    scene.handle_input(event);
}

async fn make_gui_scene() -> Scene {
    let mut scene = Scene::new();

    let label = Label::new("A label", Point::new(10, 30));
    scene.add("label1", label);

    let button = Button::new("A Button", Point::new(10,60));
    scene.add("button1", button);

    let textinput = TextInput::new("type text here", Rectangle::new(Point::new(10,90),Size::new(200,30)));
    scene.add(TEXT_INPUT,textinput);

    let menuview = MenuView::new(vec!["first","second"], Point::new(100,30));
    scene.add("menuview", menuview);


    let panel = Panel::new(Rectangle::new(Point::new(20,20),Size::new(200,200)));
    scene.add("panel", panel);
    let button = Button::new("panel button", Point::new(20,60));
    scene.add("panel-button", button);
    if let Some(view)= scene.get_view_mut("panel") {
        if let Some(panel)= view.as_any_mut().downcast_mut::<Panel>() {
            panel.add_child("panel-button".to_string());
        }
    }
    scene
}