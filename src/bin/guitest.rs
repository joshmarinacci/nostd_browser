#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use esp_hal::clock::CpuClock;

use log::{error, info};
use nostd_browser::tdeck::Wrapper;
use rust_embedded_gui::device::EmbeddedDrawingContext;
use rust_embedded_gui::geom::{Bounds, Point};
use rust_embedded_gui::grid::{make_grid_panel, GridLayoutState};
use rust_embedded_gui::label::make_label;
use rust_embedded_gui::scene::{click_at, draw_scene, event_at_focused, Scene};
use rust_embedded_gui::toggle_button::make_toggle_button;
use rust_embedded_gui::toggle_group::make_toggle_group;
use rust_embedded_gui::{Callback, EventType, Theme};

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
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    let mut wrapper = Wrapper::init(peripherals);

    let theme: Theme = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        selected_bg: Rgb565::WHITE,
        selected_fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_7X13,
        bold_font: FONT_7X13_BOLD,
    };
    let mut scene = make_gui_scene();
    let mut handlers: Vec<Callback> = vec![];
    handlers.push(|event| {
        info!("event happened {} {:?}", event.target, event.event_type);
        // show menu when tapping the button
        if let EventType::Tap(_point) = &event.event_type {
            if event.target == "button1" {
                event.scene.show_view("menuview");
                event.scene.set_focused("menuview");
            }
        }
    });

    let mut last_touch_event: Option<gt911::Point> = None;
    scene.mark_dirty_all();
    loop {
        if let Some(key) = wrapper.poll_keyboard() {
            event_at_focused(&mut scene, EventType::Keyboard(key));
        }
        if let Ok(point) = wrapper.touch.get_touch(&mut wrapper.i2c) {
            // stack allocated Vec containing 0-5 points
            if let None = &point {
                if let Some(point) = last_touch_event {
                    let pt = Point::new(320 - point.y as i32, 240 - point.x as i32);
                    click_at(&mut scene, &mut handlers, pt);
                }
            }
            last_touch_event = point;
        }
        {
            let mut ctx = EmbeddedDrawingContext::new(&mut wrapper.display);
            ctx.clip = scene.dirty_rect.clone();
            draw_scene(&mut scene, &mut ctx, &theme);
        }
        Timer::after(Duration::from_millis(20)).await;
    }
}

fn make_gui_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    // let panel = make_panel("panel", Bounds::new(20, 20, 260, 200));
    let mut panel = make_grid_panel("panel");
    panel.bounds.x = 20;
    panel.bounds.y = 20;
    panel.bounds.w = 300;
    panel.bounds.h = 200;
    scene.add_view_to_parent(
        make_label("label1", "Label 1").position_at(40, 30),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_label("label2", "Label 2").position_at(40, 30),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_label("label3", "Label 3").position_at(40, 30),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_label("label4", "Label 4").position_at(40, 30),
        &panel.name,
    );
    let mut layout = GridLayoutState::new_row_column(2, 30, 2, 80);
    layout.place_at_row_column("label1", 0, 0);
    layout.place_at_row_column("label2", 0, 1);
    layout.place_at_row_column("label3", 1, 0);
    layout.place_at_row_column("label4", 1, 1);

    scene.add_view_to_parent(
        make_toggle_button("toggle1", "Toggle").position_at(40, 70),
        &panel.name,
    );

    scene.add_view_to_parent(
        make_toggle_group("toggle2", vec!["Foo", "Bar", "Baz"], 0).position_at(40, 120),
        &panel.name,
    );

    scene.add_view_to_root(panel);

    // let mut button = make_button("button1", "A Button");
    // button.bounds.x = 40;
    // button.bounds.y = 60;
    // connect_parent_child(&mut scene, "panel", "button1");
    // scene.add_view(button);
    //
    // let mut text_input = make_text_input("textinput", "type text here");
    // text_input.bounds.x = 40;
    // text_input.bounds.y = 100;
    // connect_parent_child(&mut scene, "panel", &text_input.name);
    // scene.add_view(text_input);
    //
    // let mut menuview = make_menuview(
    //     "menuview",
    //     vec!["first".into(), "second".into(), "third".into()],
    // );
    // menuview.bounds.x = 100;
    // menuview.bounds.y = 30;
    // menuview.visible = false;
    // scene.add_view_to_root(menuview);

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
