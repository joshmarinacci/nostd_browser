use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::{FONT_5X7, FONT_6X10, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Text, TextStyleBuilder, TextStyle as ETextStyle, Baseline};
use rust_embedded_gui::button::make_button;
use rust_embedded_gui::geom::{Bounds, Point as GPoint};
use rust_embedded_gui::scene::{
    click_at, draw_scene, event_at_focused, layout_scene, EventResult, Scene,
};
use rust_embedded_gui::toggle_button::make_toggle_button;
use rust_embedded_gui::toggle_group::{make_toggle_group, SelectOneOfState};
use rust_embedded_gui::{Action, Callback, EventType, Theme};
use std::ops::Add;

#[cfg(feature = "std")]
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::fmt::style::Color::Rgb;
use env_logger::Target;
use log::{info, LevelFilter};
use rust_embedded_gui::device::EmbeddedDrawingContext;
use rust_embedded_gui::gfx::{DrawingContext, HAlign, TextStyle, VAlign};
use rust_embedded_gui::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
use rust_embedded_gui::label::make_label;
use rust_embedded_gui::list_view::make_list_view;
use rust_embedded_gui::panel::{layout_hbox, layout_vbox, make_panel, PanelState};
use rust_embedded_gui::text_input::make_text_input;
use rust_embedded_gui::view::View;
use nostd_browser::browser::{handle_action2, make_gui_scene, update_view_from_keyboard_input, AppState, LIGHT_THEME, PAGE_VIEW};

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut scene = make_gui_scene();
    let mut theme = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        selected_bg: Rgb565::BLUE,
        selected_fg: Rgb565::WHITE,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_7X13,
        bold_font: FONT_7X13_BOLD,
    };
    scene.set_focused(PAGE_VIEW);


    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Simulator Test", &output_settings);
    let mut app: AppState = AppState {
        theme: &LIGHT_THEME,
        font: &embedded_graphics::mono_font::ascii::FONT_7X13,
        bold_font: &FONT_7X13_BOLD,
    };

    'running: loop {
        let mut ctx = EmbeddedDrawingContext::new(&mut display);
        ctx.clip = scene.dirty_rect.clone();
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => {
                    let key: u8 = keydown_to_char(keycode, keymod);
                    println!(
                        "keyboard event {} {} {:?}",
                        keycode.name(),
                        key,
                        String::from(key as char)
                    );
                    if key > 0 {
                        if let Some(result) = event_at_focused(&mut scene, EventType::Keyboard(key))
                        {
                            println!("got input from {:?}", result);
                            handle_events(result, &mut scene, &mut theme, &mut app);
                        }
                        update_view_from_keyboard_input(&mut scene, key);
                    }
                }
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("mouse button up {}", point);
                    if let Some(result) =
                        click_at(&mut scene, &vec![], GPoint::new(point.x, point.y))
                    {
                        handle_events(result, &mut scene, &mut theme, &mut app);
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    println!("mouse down");
                }
                SimulatorEvent::MouseWheel {scroll_delta, direction} => {
                    info!("mouse wheel {scroll_delta:?} {direction:?}");
                    if let Some(result) = event_at_focused(&mut scene,EventType::Scroll(scroll_delta.x,scroll_delta.y)) {
                        println!("got input from {:?}", result);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> u8 {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return 0;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch.is_alphabetic() {
            return if shifted {
                ch.to_ascii_uppercase() as u8
            } else {
                ch.to_ascii_lowercase() as u8
            };
        }
        if ch.is_ascii_graphic() {
            return ch as u8;
        }
    }
    match keycode {
        Keycode::Backspace => 8,
        Keycode::SPACE => b' ',
        _ => {
            println!("not supported: {keycode}");
            0
        }
    }
}

fn handle_events(result: EventResult, scene: &mut Scene, theme: &mut Theme, app: &mut AppState) {
    let (name, action) = result;
    println!("result of event {:?} from {name}", action);
    handle_action2(&name, &action, scene, app);
}
