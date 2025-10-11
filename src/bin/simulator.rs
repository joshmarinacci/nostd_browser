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
use iris_ui::geom::Point;
use iris_ui::input::{InputEvent, TextAction};
use iris_ui::scene::{click_at, draw_scene, event_at_focused, layout_scene};
use log::{info, LevelFilter};
use nostd_browser::browser::{
    handle_action, load_page, make_gui_scene, update_view_from_keyboard_input, AppState,
    GuiResponse, NetCommand, LIGHT_THEME, PAGE_VIEW,
};
use nostd_browser::page::Page;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::geom::{Point as GPoint};
use iris_ui::{Theme, ViewStyle};
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
        standard: ViewStyle {
            fill: Rgb565::WHITE,
            text: Rgb565::BLACK,
        },
        panel: ViewStyle {
            fill: Rgb565::CSS_LIGHT_GRAY,
            text: Rgb565::BLACK,
        },
        selected: ViewStyle {
            fill: Rgb565::BLUE,
            text: Rgb565::WHITE,
        },
        accented: ViewStyle {
            fill: Rgb565::BLUE,
            text: Rgb565::WHITE,
        },
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
        theme.standard.fill = app.theme.base_bg;
        theme.standard.text = app.theme.base_fg;
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
                    let evt = keydown_to_char(keycode, keymod);
                    if let Some(result) = event_at_focused(&mut scene, &InputEvent::Text(evt)) {
                        println!("got input from {:?}", result.source);
                        if let Some(resp) = handle_action(&result, &mut scene, &mut app) {
                            info!("gui response {:?}", resp);
                            handle_gui_response(resp, &mut app).await;
                        }
                    }
                    update_view_from_keyboard_input(&mut scene, &evt);
                }
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("mouse button up {}", point);
                    let pt = Point::new(point.x, point.y);
                    if let Some(result) = click_at(&mut scene, &vec![], pt) {
                        println!("got input from {:?}", result.source);
                        if let Some(resp) = handle_action(&result, &mut scene, &mut app) {
                            info!("gui response {:?}", resp);
                            handle_gui_response(resp, &mut app).await;
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
                        &InputEvent::Scroll(Point::new(scroll_delta.x, scroll_delta.y)),
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

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> TextAction {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return TextAction::Unknown;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch.is_alphabetic() {
            return if shifted {
                TextAction::TypedAscii(ch.to_ascii_uppercase() as u8)
            } else {
                TextAction::TypedAscii(ch.to_ascii_lowercase() as u8)
            };
        }
        if ch.is_ascii_graphic() {
            return TextAction::TypedAscii(ch as u8);
        }
    }
    match keycode {
        Keycode::Backspace => TextAction::BackDelete,
        Keycode::Return => TextAction::Enter,
        Keycode::LEFT => TextAction::Left,
        Keycode::RIGHT => TextAction::Right,
        Keycode::UP =>  TextAction::Up,
        Keycode::DOWN => TextAction::Down,
        Keycode::SPACE => TextAction::TypedAscii(b' '),
        _ => {
            println!("not supported: {keycode}");
            return TextAction::Unknown;
        }
    }
}
