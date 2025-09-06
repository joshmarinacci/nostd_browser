#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::{vec};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use embassy_executor::Spawner;
// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
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
// use esp_hal::peripherals::Peripherals;
use gt911::Gt911Blocking;
use gui2::{click_at, connect_parent_child, draw_scene, pick_at, type_at_focused, Callback, DrawingContext, EventType, GuiEvent, Scene, Theme, View};
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::geom::{Bounds, Point as GPoint};
use log::{error, info};

use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder};
use nostd_browser::common::{TDeckDisplay};
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

// static TRACKBALL_CHANNEL: Channel<CriticalSectionRawMutex, GuiEvent<Rgb565>, 2> = Channel::new();
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
    mut scene: Scene<Rgb565>,
) {
    let touch = Gt911Blocking::default();
    touch.init(i2c).unwrap();
    let theme:Theme<Rgb565> = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
    };

    let mut ctx:EmbeddedDrawingContext = EmbeddedDrawingContext::new(display);
    let mut handlers: Vec<Callback<Rgb565>> = vec![];
    handlers.push(|event| {
        info!("event happened {} {:?}", event.target, event.event_type);
        // show menu when tapping the button
        if let EventType::Tap(point) = &event.event_type {
            if event.target == "button1" {
                event.scene.set_visible("menuview");
                event.scene.set_focused("menuview");
            }
        }
    });

    let mut last_touch_event:Option<gt911::Point> = None;
    loop {
        if let Ok(point) = touch.get_touch(i2c) {
            // emit tap when the touch event ends
            if let None = &point {
                if let Some(point) = last_touch_event {
                    let pt = GPoint::new(320 - point.y as i32, 240-point.x as i32);
                    click_at(&mut scene,&mut handlers,pt);
                }
            }
            last_touch_event = point;
        }
        let mut data = [0u8; 1];
        let kb_res = (*i2c).read(LILYGO_KB_I2C_ADDRESS, &mut data);
        match kb_res {
            Ok(_) => {
                if data[0] != 0x00 {
                    type_at_focused(&mut scene, &handlers, data[0])
                }
            }
            Err(_) => {
                // info!("kb_res = {}", e);
            }
        }
        draw_scene(&mut scene,&mut ctx,&theme);
        Timer::after(Duration::from_millis(20)).await;
    }
}

struct EmbeddedDrawingContext {
    pub display:&'static mut TDeckDisplay
}

impl EmbeddedDrawingContext {
    fn new(display: &'static mut TDeckDisplay) -> EmbeddedDrawingContext {
        EmbeddedDrawingContext {
            display,
        }
    }
}

impl DrawingContext<Rgb565> for EmbeddedDrawingContext {
    fn clear(&mut self, color: &Rgb565) {
        self.display.clear(*color).unwrap();
    }

    fn fillRect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let pt = Point::new(bounds.x,bounds.y);
        let size = Size::new(bounds.w as u32, bounds.h as u32);
        Rectangle::new(pt,size)
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(self.display).unwrap();

    }

    fn strokeRect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let pt = Point::new(bounds.x,bounds.y);
        let size = Size::new(bounds.w as u32, bounds.h as u32);
        Rectangle::new(pt,size)
            .into_styled(PrimitiveStyle::with_stroke(*color,1))
            .draw(self.display).unwrap();
    }

    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &Rgb565) {
        let style = MonoTextStyle::new(&FONT_6X10, *color);
        let mut pt = Point::new(bounds.x, bounds.y);
        pt.y += bounds.h / 2;
        pt.y += (FONT_6X10.baseline as i32)/2;
        let w = (FONT_6X10.character_size.width as i32) * (text.len() as i32);
        pt.x += (bounds.w - w) / 2;
        Text::new(text, pt, style)
            .draw(self.display)
            .unwrap();
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
            // TRACKBALL_CHANNEL.send(GuiEvent::ClickEvent()).await;
        }
        // info!("button pressed is {} ", tdeck_track_click.is_low());
        if right.is_high() != last_right_high {
            // info!("right");
            last_right_high = right.is_high();
            cursor.x += 1;
            // TRACKBALL_CHANNEL
            //     .send(GuiEvent::ScrollEvent(cursor, Point::new(1, 0)))
            //     .await;
        }
        if left.is_high() != last_left_high {
            // info!("left");
            last_left_high = left.is_high();
            cursor.x -= 1;
            // TRACKBALL_CHANNEL
            //     .send(GuiEvent::ScrollEvent(cursor, Point::new(-1, 0)))
            //     .await;
        }
        if up.is_high() != last_up_high {
            // info!("up");
            last_up_high = up.is_high();
            cursor.y -= 1;
            // TRACKBALL_CHANNEL
            //     .send(GuiEvent::ScrollEvent(cursor, Point::new(0, -1)))
            //     .await;
        }
        if down.is_high() != last_down_high {
            // info!("down");
            last_down_high = down.is_high();
            cursor.y += 1;
            // TRACKBALL_CHANNEL
            //     .send(GuiEvent::ScrollEvent(cursor, Point::new(0, 1)))
            //     .await;
        }
        // wait one msec
        Timer::after(Duration::from_millis(1)).await;
    }
}


// const TEXT_INPUT: &str = "textinput";

async fn make_gui_scene() -> Scene<Rgb565> {
    let mut scene = Scene::new();


    let panel = make_panel("panel", Bounds::new(20,20,260,200));
    scene.add_view_to_root(panel);

    let mut label = make_label("label1","A label");
    label.bounds.x = 40;
    label.bounds.y = 30;
    connect_parent_child(&mut scene, "panel", &label.name);
    scene.add_view(label);

    let mut button = make_button("button1","A Button");
    button.bounds.x = 40;
    button.bounds.y = 60;
    connect_parent_child(&mut scene, "panel", "button1");
    scene.add_view(button);


    let mut text_input = make_text_input("textinput","type text here");
    text_input.bounds.x = 40;
    text_input.bounds.y = 100;
    connect_parent_child(&mut scene, "panel", &text_input.name);
    scene.add_view(text_input);

    let mut menuview = make_menuview(vec!["first".into(), "second".into(), "third".into()]);
    menuview.bounds = Bounds::new(100,30,150,80);
    menuview.name = "menuview".into();
    menuview.visible = false;
    scene.add_view_to_root(menuview);

    // let mut button = make_button("panel button");
    // button.bounds = Bounds::new(20,60,100,30);
    // button.name = "panel-button".into();
    // button.visible = false;
    // connect_parent_child(&mut scene, &root_id, &button.name);
    // scene.add_view(button);
    // if let Some(view)= scene.get_view_mut("panel") {
    //     if let Some(panel)= view.as_any_mut().downcast_mut::<Panel>() {
    //         panel.add_child("panel-button".to_string());
    //     }
    // }
    scene
}

struct MenuState {
    data:Vec<String>,
    selected:usize,
}
fn make_menuview<C>(data:Vec<String>) -> View<C> {
    View {
        name: "somemenu".into(),
        title: "somemenu".into(),
        bounds: Bounds {
            x:0,
            y:0,
            w:100,
            h:200,
        },
        visible:true,
        children: vec![],
        draw: Some(|view, ctx, theme| {
            ctx.fillRect(&view.bounds, &theme.bg);
            ctx.strokeRect(&view.bounds, &theme.fg);
            if let Some(state) = &view.state {
                if let Some(state) = state.downcast_ref::<MenuState>() {
                    info!("menu state is {:?}",state.data);
                    for (i,item) in (&state.data).iter().enumerate() {
                        let b = Bounds {
                            x: view.bounds.x,
                            y: view.bounds.y + (i as i32) * 30,
                            w: view.bounds.w,
                            h: 30,
                        };
                        if state.selected == i {
                            ctx.fillRect(&b,&theme.fg);
                            ctx.fillText(&b,item.as_str(),&theme.bg);
                        }else {
                            ctx.fillText(&b, item.as_str(), &theme.fg);
                        }
                    }
                }
            }
        }),
        input: Some(|event|{
            info!("menu clicked at");
            match &event.event_type {
                EventType::Tap(pt) => {
                    info!("tapped at {:?}",pt);
                    if let Some(view) = event.scene.get_view_mut(event.target) {
                        info!("the view is {} at {:?}",view.name, view.bounds);
                        let name = view.name.clone();
                        if view.bounds.contains(pt) {
                            info!("I was clicked on. index is {}", pt.y/30);
                            let selected = pt.y/30;
                            if let Some(state) = &mut view.state {
                                if let Some(state) = state.downcast_mut::<MenuState>() {
                                    info!("menu state is {:?}",state.data);
                                    if selected >= 0 && selected < state.data.len() as i32 {
                                        state.selected = selected as usize;
                                        event.scene.set_focused(&name);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    info!("unknown event type");
                }
            }
        }),
        layout: Some(|scene, name|{
            if let Some(parent) = scene.get_view_mut(name) {
                if let Some(state) = &parent.state {
                    if let Some(state) = state.downcast_ref::<MenuState>() {
                        parent.bounds.h = 30 * (state.data.len() as i32)
                    }
                }
            };
        }),
        state: Some(Box::new(MenuState{data,selected:0})),
    }
}