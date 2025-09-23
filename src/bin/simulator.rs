use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::mono_font::ascii::{
    FONT_5X7, FONT_6X10, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD,
};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{
    Alignment, Baseline, Text, TextStyle as ETextStyle, TextStyleBuilder,
};
use embedded_graphics::Drawable;
use rust_embedded_gui::button::make_button;
use rust_embedded_gui::geom::{Bounds, Point as GPoint};
use rust_embedded_gui::scene::{
    click_at, draw_scene, event_at_focused, layout_scene, EventResult, Scene,
};
use rust_embedded_gui::toggle_button::make_toggle_button;
use rust_embedded_gui::toggle_group::{make_toggle_group, SelectOneOfState};
use rust_embedded_gui::{Action, Callback, EventType, KeyboardAction, Theme};
use std::ops::Add;
use uchan::{Sender};
#[cfg(feature = "std")]
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::fmt::style::Color::Rgb;
use env_logger::Target;
use log::{info, LevelFilter};
use nostd_browser::browser::{handle_action, load_page, make_gui_scene, update_view_from_keyboard_input, AppState, GuiResponse, NetCommand, LIGHT_THEME, PAGE_VIEW};
use nostd_browser::page::Page;
use nostd_browser::pageview::PageView;
use rust_embedded_gui::device::EmbeddedDrawingContext;
use rust_embedded_gui::gfx::{DrawingContext, HAlign, TextStyle, VAlign};
use rust_embedded_gui::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
use rust_embedded_gui::label::make_label;
use rust_embedded_gui::list_view::make_list_view;
use rust_embedded_gui::panel::{layout_hbox, layout_vbox, make_panel, PanelState};
use rust_embedded_gui::text_input::make_text_input;
use rust_embedded_gui::view::View;

static PAGE_BYTES: &[u8] = include_bytes!("homepage.html");

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let (page_sender, page_receiver) = uchan::channel::<Page>();
    let mut scene = make_gui_scene(page_sender.clone());
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

    page_sender.send(Page::from_bytes(PAGE_BYTES, "homepage.html")).unwrap();

    'running: loop {
        let mut ctx = EmbeddedDrawingContext::new(&mut display);
        ctx.clip = scene.dirty_rect.clone();
        theme.bg = app.theme.base_bg;
        theme.fg = app.theme.base_fg;
        theme.font = app.font.clone();
        theme.bold_font = app.font.clone();
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => {
                    let evt: EventType = keydown_to_char(keycode, keymod);
                    if let Some((name, action)) = event_at_focused(&mut scene, &evt) {
                        println!("got input from {:?}", name);
                        if let Some(resp) = handle_action(&name, &action, &mut scene, &mut app) {
                            info!("gui response {:?}",resp);
                            handle_gui_response(resp, &mut app, page_sender.clone());
                        }
                    }
                    update_view_from_keyboard_input(&mut scene, &evt);
                }
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("mouse button up {}", point);
                    let pt = GPoint::new(point.x, point.y);
                    if let Some((name, action)) = click_at(&mut scene, &vec![], pt) {
                        println!("got input from {:?}", name);
                        if let Some(resp) = handle_action(&name, &action, &mut scene, &mut app) {
                            info!("gui response {:?}",resp);
                            handle_gui_response(resp, &mut app, page_sender.clone());
                        }
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    println!("mouse down");
                }
                SimulatorEvent::MouseWheel {
                    scroll_delta,
                    direction,
                } => {
                    info!("mouse wheel {scroll_delta:?} {direction:?}");
                    if let Some(result) = event_at_focused(
                        &mut scene,
                        &EventType::Scroll(scroll_delta.x, scroll_delta.y),
                    ) {
                        println!("got input from {:?}", result);
                    }
                }
                _ => {}
            }
        }
        if let Ok(page) = page_receiver.try_recv() {
            load_page(&mut scene, page);
        }
    }
    Ok(())
}

fn handle_gui_response(gui_response: GuiResponse, x: &mut AppState, sender: Sender<Page>) {
    match gui_response {
        GuiResponse::Net(net) => {
            match net {
                NetCommand::Load(href) => {
                    let client = reqwest::blocking::ClientBuilder::new()
                        .use_rustls_tls()
                        .build().unwrap();
                    let res = client.get(&href).send().unwrap();
                    let bytes = res.bytes().unwrap();
                    let page = Page::from_bytes(&bytes, &href);
                    info!("got result bytes {:?}",page);
                    sender.send(page).unwrap();
                }
            }
        },
    }
}

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> EventType {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return EventType::Unknown;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch.is_alphabetic() {
            return if shifted {
                EventType::Keyboard(ch.to_ascii_uppercase() as u8)
            } else {
                EventType::Keyboard(ch.to_ascii_lowercase() as u8)
            };
        }
        if ch.is_ascii_graphic() {
            return EventType::Keyboard(ch as u8);
        }
    }
    match keycode {
        Keycode::Backspace => EventType::KeyboardAction(KeyboardAction::Backspace),
        Keycode::Return => EventType::KeyboardAction(KeyboardAction::Return),
        Keycode::LEFT => EventType::KeyboardAction(KeyboardAction::Left),
        Keycode::RIGHT => EventType::KeyboardAction(KeyboardAction::Right),
        Keycode::UP => EventType::KeyboardAction(KeyboardAction::Up),
        Keycode::DOWN => EventType::KeyboardAction(KeyboardAction::Down),
        Keycode::SPACE => EventType::Keyboard(b' '),
        _ => {
            println!("not supported: {keycode}");
            return EventType::Unknown;
        }
    }
}

