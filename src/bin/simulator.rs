use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embedded_graphics::geometry::{Size};
use embedded_graphics::mono_font::ascii::{
    FONT_7X13_BOLD,
};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::pixelcolor::{Rgb565};
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::Target;
use log::{info, LevelFilter};
use nostd_browser::browser::{
    handle_action, load_page, make_gui_scene, update_view_from_keyboard_input, AppState,
    GuiResponse, NetCommand, LIGHT_THEME, PAGE_VIEW,
};
use nostd_browser::page::Page;
use rust_embedded_gui::device::EmbeddedDrawingContext;
use rust_embedded_gui::geom::{Point as GPoint};
use rust_embedded_gui::scene::{
    click_at, draw_scene, event_at_focused, layout_scene,
};
use rust_embedded_gui::{EventType, KeyboardAction, Theme};
use reqwest::blocking::ClientBuilder;

static PAGE_BYTES: &[u8] = include_bytes!("homepage.html");

static PAGE_CHANNEL: Channel<ThreadModeRawMutex, Page, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner:Spawner) {
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

    PAGE_CHANNEL.send(Page::from_bytes(PAGE_BYTES, "homepage.html"));

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
                SimulatorEvent::Quit => {
                    std::process::exit(0);
                },
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => {
                    let evt: EventType = keydown_to_char(keycode, keymod);
                    if let Some((name, action)) = event_at_focused(&mut scene, &evt) {
                        println!("got input from {:?}", name);
                        if let Some(resp) = handle_action(&name, &action, &mut scene, &mut app) {
                            info!("gui response {:?}", resp);
                            handle_gui_response(resp, &mut app);
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
                            info!("gui response {:?}", resp);
                            handle_gui_response(resp, &mut app);
                        }
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn: _mouse_btn, point: _point } => {
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
        if let Ok(page) = PAGE_CHANNEL.try_receive() {
            load_page(&mut scene, page);
        }
    }
}

async fn handle_gui_response(gui_response: GuiResponse, _app: &mut AppState) {
    match gui_response {
        GuiResponse::Net(net) => {
            match net {
                NetCommand::Load(href) => {
                    let client = ClientBuilder::new()
                        .use_rustls_tls()
                        .build()
                        .unwrap();
                    let res = client.get(&href).send().unwrap();
                    let bytes = res.bytes().unwrap();
                    let page = Page::from_bytes(&bytes, &href);
                    info!("got result bytes {:?}", page);
                    PAGE_CHANNEL.send(page).await;
                }
            }
        }
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
