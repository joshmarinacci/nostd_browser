#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Runner, Stack, StackResources};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::wifi::ScanTypeConfig::Active;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, ScanConfig, WifiController, WifiDevice, WifiEvent,
    WifiState,
};
use esp_wifi::{init, EspWifiController};
use iris_ui::{Callback, Theme, ViewStyle};
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::geom::Point;
use iris_ui::input::{InputEvent, TextAction};
use iris_ui::input::InputAction::FocusSelect;
use iris_ui::input::InputEvent::Text;
use iris_ui::scene::{click_at, draw_scene, event_at_focused, layout_scene};
use log::{error, info, warn};
use reqwless::client::{HttpClient, TlsConfig};

use nostd_browser::browser::{handle_action, make_gui_scene, update_view_from_keyboard_input, AppState, GuiResponse, LIGHT_THEME, PAGE_VIEW};
use nostd_browser::page::Page;
use nostd_browser::pageview::PageView;
use device::common::{NetCommand, NetStatus, NET_COMMANDS, NET_STATUS};
use device::tdeck::Wrapper;

#[panic_handler]
fn panic(nfo: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {:?}", nfo);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}


const AUTO_CONNECT: Option<&str> = option_env!("AUTO_CONNECT");

static PAGE_BYTES: &[u8] = include_bytes!("homepage.html");

static PAGE_CHANNEL: Channel<CriticalSectionRawMutex, Page, 1> = Channel::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    let mut wrapper = Wrapper::init(peripherals);

    if AUTO_CONNECT.is_some() {
        wrapper.start_wifi(&spawner).await;
    } else {
        PAGE_CHANNEL
            .sender()
            .send(Page::from_bytes(PAGE_BYTES, "homepage.html"))
            .await;
    }

    spawner.spawn(update_display(wrapper)).ok();

    Timer::after(Duration::from_millis(1000)).await;
}
#[embassy_executor::task]
async fn update_display(mut wrapper: Wrapper) {
    let mut scene = make_gui_scene();
    let mut app: AppState = AppState {
        theme: &LIGHT_THEME,
        font: &FONT_7X13,
        bold_font: &FONT_7X13_BOLD,
    };

    let handlers: Vec<Callback> = vec![];
    
    let mut last_touch_event: Option<gt911::Point> = None;
    scene.set_focused(PAGE_VIEW);
    loop {
        // if let Ok(page) = PAGE_CHANNEL.try_receive() {
        //     if let Some(state) = scene.get_view_state::<PageView>(PAGE_VIEW) {
        //         info!("page got a new page: {:?}", page);
        //         state.load_page(page);
        //     }
        //     scene.mark_dirty_view(PAGE_VIEW);
        //     info!("heap is {}", esp_alloc::HEAP.stats());
        // }
        // if let Ok(status) = NET_STATUS.try_receive() {
        //     info!("got the status {status:?}");
        //     let txt = match &status {
        //         NetStatus::Info(txt) => txt,
        //         _ => &format!("{:?}", status).to_string(),
        //     };
        //     if let Some(overlay) = scene.get_view_mut("overlay-status") {
        //         overlay.title = txt.into();
        //     }
        // }

        if let Ok(point) = wrapper.touch.get_touch(&mut wrapper.i2c) {
            if let None = &point {
                if let Some(point) = last_touch_event {
                    let pt = Point::new(320 - point.y as i32, 240 - point.x as i32);
                    let res = click_at(&mut scene, &vec![], pt);
                    if let Some(result) = res {
                        handle_action(&result, &mut scene, &mut app);
                    }
                }
            }
            last_touch_event = point;
        }
        if let Some(key) = wrapper.poll_keyboard() {
            let text_action = if key == b' ' {
                info!("doing a space as an action");
                update_view_from_keyboard_input(&mut scene, &TextAction::TypedAscii(key));
                TextAction::Enter
            } else {
                TextAction::TypedAscii(key)
            };
            if let Some(result) = event_at_focused(&mut scene, &InputEvent::Text(text_action)) {
                if let Some(resp) = handle_action(&result, &mut scene, &mut app) {
                    info!("gui response {:?}",resp);
                    handle_gui_response(resp, &mut app).await;
                }
            }
        }

        wrapper.poll_trackball();
        if wrapper.click.changed {
            if let Some(result) = event_at_focused(&mut scene, &InputEvent::Action(FocusSelect)) {
                if let Some(resp) = handle_action(&result, &mut scene, &mut app) {
                    info!("gui response {:?}",resp);
                    handle_gui_response(resp, &mut app);
                }
            }
        }
        if wrapper.up.changed {
            event_at_focused(&mut scene, &InputEvent::Scroll(Point::new(0, -1)));
        }
        if wrapper.down.changed {
            event_at_focused(&mut scene, &InputEvent::Scroll(Point::new(0, 1)));
        }
        let mut ctx = EmbeddedDrawingContext::new(&mut wrapper.display);
        ctx.clip = scene.dirty_rect.clone();
        let theme: Theme = Theme {
            standard: ViewStyle {
                fill: app.theme.base_bg,
                text: app.theme.base_fg,
            },
            selected: ViewStyle {
                fill: app.theme.base_bg,
                text: app.theme.base_fg,
            },
            accented: ViewStyle {
                fill: app.theme.base_bg,
                text: app.theme.base_fg,
            },
            panel: ViewStyle {
                fill: app.theme.base_bg,
                text: app.theme.base_fg,
            },
            font: app.font.clone(),
            bold_font: app.bold_font.clone(),
        };
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        Timer::after(Duration::from_millis(20)).await;
    }
}

async fn load_file_url(href: &str) -> &[u8] {
    PAGE_BYTES
}
async fn handle_file_url(href: &str) {
    info!("sdcard url {}", href);
    let path = &href[5..];
    info!("loading path {}", path);

    let bytes = load_file_url(&href).await;
    PAGE_CHANNEL
        .sender()
        .send(Page::from_bytes(bytes, &href))
        .await;
}

async fn handle_bookmarks(href: &str) {
    PAGE_CHANNEL
        .sender()
        .send(Page::from_bytes(PAGE_BYTES, &href))
        .await;
}
async fn handle_gui_response(gui_response: GuiResponse, _app: &mut AppState) {
    match gui_response {
        GuiResponse::Net(net) => match net {
            nostd_browser::browser::NetCommand::Load(href) => {
                NET_COMMANDS.send(NetCommand::Load(href)).await;
            }
        },
    }
}


async fn handle_http_url(href: &str, network_stack: Stack<'static>, tls_seed: u64) {
    NET_STATUS.send(NetStatus::LoadingPage()).await;
    let mut rx_buffer = [0; 4096 * 2];
    let mut tx_buffer = [0; 4096 * 2];
    let dns = DnsSocket::new(network_stack);
    let tcp_state = TcpClientState::<1, 4096, 4096>::new();
    let tcp = TcpClient::new(network_stack, &tcp_state);

    let tls = TlsConfig::new(
        tls_seed,
        &mut rx_buffer,
        &mut tx_buffer,
        reqwless::client::TlsVerify::None,
    );

    let mut client = HttpClient::new_with_tls(&tcp, &dns, tls);
    // let mut client = HttpClient::new(&tcp, &dns);
    let mut buffer = [0u8; 4096 * 5];
    info!("making the actual request to {}", href);
    // let url = "https://joshondesign.com/2023/07/12/css_text_style_builder";
    let mut http_req = client
        .request(reqwless::request::Method::GET, &href)
        .await
        .unwrap();
    let resp = http_req.send(&mut buffer).await;
    match resp {
        Ok(response) => {
            info!("Got response");
            let res = response.body().read_to_end().await.unwrap();
            PAGE_CHANNEL
                .sender()
                .send(Page::from_bytes(res, &href))
                .await;
            NET_STATUS.send(NetStatus::PageLoaded()).await;
        }
        Err(err) => {
            info!("Got error: {:?}", err);
            NET_STATUS
                .send(NetStatus::Error(format!("{:?}", err)))
                .await;
        }
    }
}
#[embassy_executor::task]
async fn page_downloader(network_stack: Stack<'static>, tls_seed: u64) {
    loop {
        if let Ok(cmd) = NET_COMMANDS.try_receive() {
            info!("Network command: {:?}", cmd);
            match cmd {
                NetCommand::Load(href) => {
                    info!("Loading page: {}", href);
                    if href == "bookmarks.html" {
                        handle_bookmarks(&href).await;
                    } else if href.starts_with("file:") {
                        handle_file_url(&href).await;
                    } else {
                        // if !href.starts_with("http") {
                        //     info!("relative url");
                        // }
                        handle_http_url(&href, network_stack, tls_seed).await;
                    }
                }
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
