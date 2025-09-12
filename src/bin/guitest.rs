#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::geom::{Bounds, Point as GPoint};
use gui2::{
    click_at, connect_parent_child, draw_scene, pick_at, scroll_at_focused, type_at_focused,
    Callback, DrawingContext, EventType, GuiEvent, HAlign, Scene, Theme, View,
};
use log::{error, info};

use nostd_browser::menuview::make_menuview;
use nostd_browser::tdeck::{EmbeddedDrawingContext, Wrapper};
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

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    let mut wrapper = Wrapper::init(peripherals);

    let theme: Theme<Rgb565, MonoFont> = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_7X13,
        bold_font: FONT_7X13_BOLD,
    };
    let mut scene = make_gui_scene();
    let mut handlers: Vec<Callback<Rgb565, MonoFont>> = vec![];
    handlers.push(|event| {
        info!("event happened {} {:?}", event.target, event.event_type);
        // show menu when tapping the button
        if let EventType::Tap(point) = &event.event_type {
            if event.target == "button1" {
                event.scene.show_view("menuview");
                event.scene.set_focused("menuview");
            }
        }
    });

    let mut last_touch_event: Option<gt911::Point> = None;
    loop {
        if let Some(key) = wrapper.poll_keyboard() {
            type_at_focused(&mut scene, &handlers, key);
        }
        if let Ok(point) = wrapper.touch.get_touch(&mut wrapper.i2c) {
            // stack allocated Vec containing 0-5 points
            if let None = &point {
                if let Some(point) = last_touch_event {
                    let pt = GPoint::new(320 - point.y as i32, 240 - point.x as i32);
                    click_at(&mut scene, &mut handlers, pt);
                }
            }
            last_touch_event = point;
        }
        {
            let mut ctx: EmbeddedDrawingContext = EmbeddedDrawingContext::new(&mut wrapper.display);
            draw_scene(&mut scene, &mut ctx, &theme);
        }
        Timer::after(Duration::from_millis(20)).await;

        // loop {
        //     let mut data = [0u8; 1];
        //     let kb_res = (*i2c).read(LILYGO_KB_I2C_ADDRESS, &mut data);
        //     match kb_res {
        //         Ok(_) => {
        //             if data[0] != 0x00 {
        //                 type_at_focused(&mut scene, &handlers, data[0])
        //             }
        //         }
        //         Err(_) => {
        //             // info!("kb_res = {}", e);
        //         }
        //     }
        //
        //     {
        //
        //         if click.is_low() != last_click_low {
        //             last_click_low = click.is_low();
        //         }
        //         if right.is_high() != last_right_high {
        //             last_right_high = right.is_high();
        //             cursor.x += 1;
        //         }
        //         if left.is_high() != last_left_high {
        //             last_left_high = left.is_high();
        //             cursor.x -= 1;
        //         }
        //         if up.is_high() != last_up_high {
        //             last_up_high = up.is_high();
        //             cursor.y -= 1;
        //             scroll_at_focused(&mut scene, &handlers, 0,-1);
        //         }
        //         if down.is_high() != last_down_high {
        //             last_down_high = down.is_high();
        //             cursor.y += 1;
        //             scroll_at_focused(&mut scene, &handlers, 0,1);
        //         }
        //     }
        //
        // }
    }
}

// #[embassy_executor::task]
// async fn handle_trackball(
//     click: Input<'static>,
//     left: Input<'static>,
//     right: Input<'static>,
//     up: Input<'static>,
//     down: Input<'static>,
// ) {
//     let mut last_click_low = false;
//     let mut last_right_high = false;
//     let mut last_left_high = false;
//     let mut last_up_high = false;
//     let mut last_down_high = false;
//     info!("monitoring the trackball");
//     let mut cursor = Point::new(50, 50);
//     loop {
//         if click.is_low() != last_click_low {
//             info!("click");
//             last_click_low = click.is_low();
//             // TRACKBALL_CHANNEL.send(GuiEvent::ClickEvent()).await;
//         }
//         // info!("button pressed is {} ", tdeck_track_click.is_low());
//         if right.is_high() != last_right_high {
//             // info!("right");
//             last_right_high = right.is_high();
//             cursor.x += 1;
//             // TRACKBALL_CHANNEL
//             //     .send(GuiEvent::ScrollEvent(cursor, Point::new(1, 0)))
//             //     .await;
//         }
//         if left.is_high() != last_left_high {
//             // info!("left");
//             last_left_high = left.is_high();
//             cursor.x -= 1;
//             // TRACKBALL_CHANNEL
//             //     .send(GuiEvent::ScrollEvent(cursor, Point::new(-1, 0)))
//             //     .await;
//         }
//         if up.is_high() != last_up_high {
//             // info!("up");
//             last_up_high = up.is_high();
//             cursor.y -= 1;
//             // TRACKBALL_CHANNEL
//             //     .send(GuiEvent::ScrollEvent(cursor, Point::new(0, -1)))
//             //     .await;
//         }
//         if down.is_high() != last_down_high {
//             // info!("down");
//             last_down_high = down.is_high();
//             cursor.y += 1;
//             // TRACKBALL_CHANNEL
//             //     .send(GuiEvent::ScrollEvent(cursor, Point::new(0, 1)))
//             //     .await;
//         }
//         // wait one msec
//         Timer::after(Duration::from_millis(1)).await;
//     }
// }
//
//
// const TEXT_INPUT: &str = "textinput";

fn make_gui_scene() -> Scene<Rgb565, MonoFont<'static>> {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let panel = make_panel("panel", Bounds::new(20, 20, 260, 200));
    scene.add_view_to_root(panel);

    let mut label = make_label("label1", "A label");
    label.bounds.x = 40;
    label.bounds.y = 30;
    connect_parent_child(&mut scene, "panel", &label.name);
    scene.add_view(label);

    let mut button = make_button("button1", "A Button");
    button.bounds.x = 40;
    button.bounds.y = 60;
    connect_parent_child(&mut scene, "panel", "button1");
    scene.add_view(button);

    let mut text_input = make_text_input("textinput", "type text here");
    text_input.bounds.x = 40;
    text_input.bounds.y = 100;
    connect_parent_child(&mut scene, "panel", &text_input.name);
    scene.add_view(text_input);

    let mut menuview = make_menuview(
        "menuview",
        vec!["first".into(), "second".into(), "third".into()],
    );
    menuview.bounds.x = 100;
    menuview.bounds.y = 30;
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
