#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use alloc::boxed::Box;
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::wifi::ScanTypeConfig::Active;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, ScanConfig, WifiController, WifiDevice, WifiEvent,
    WifiState,
};
use esp_wifi::{init, EspWifiController};
use gui2::{action_at_focused, click_at, draw_scene, layout_scene, scroll_at_focused, type_at_focused, Callback, Theme};
use log::{error, info, warn};
use reqwless::client::{HttpClient, TlsConfig};

use gui2::geom::Point as GPoint;
use nostd_browser::browser::{handle_action2, make_gui_scene, update_view_from_keyboard_input, AppState, LIGHT_THEME, PAGE_VIEW};
use nostd_browser::common::{NetCommand, NetStatus, NET_COMMANDS, NET_STATUS, PAGE_CHANNEL};
use nostd_browser::page::Page;
use nostd_browser::pageview::PageView;
use nostd_browser::tdeck::{EmbeddedDrawingContext, Wrapper};

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

const SSID: Option<&str> = option_env!("SSID");
const PASSWORD: Option<&str> = option_env!("PASSWORD");

const AUTO_CONNECT: Option<&str> = option_env!("AUTO_CONNECT");

static PAGE_BYTES: &[u8] = include_bytes!("homepage.html");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    let wrapper = Wrapper::init(peripherals);

    if AUTO_CONNECT.is_some() {
        // let mut rng = Rng::new(wrapper.rng);
        // let timer_g0 = TimerGroup::new(wrapper.timg0);
        //
        // info!("made timer");
        // let esp_wifi_ctrl = &*mk_static!(
        //     EspWifiController<'static>,
        //     init(timer_g0.timer0, rng.clone()).unwrap()
        // );
        // info!("making controller");
        // let (wifi_controller, interfaces) =
        //     esp_wifi::wifi::new(&esp_wifi_ctrl, wrapper.wifi).unwrap();
        // let wifi_interface = interfaces.sta;
        //
        // let config = embassy_net::Config::dhcpv4(Default::default());
        // let net_seed = (rng.random() as u64) << 32 | rng.random() as u64;
        // info!("made net seed {}", net_seed);
        // let tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
        // info!("made tls seed {}", tls_seed);
        //
        // info!("init-ing the network stack");
        // // Init network stack
        // let (network_stack, wifi_runner) = embassy_net::new(
        //     wifi_interface,
        //     config,
        //     mk_static!(StackResources<3>, StackResources::<3>::new()),
        //     net_seed,
        // );
        //
        // info!("spawning connection");
        // spawner.spawn(connection(wifi_controller)).ok();
        // info!("spawning net task");
        // spawner.spawn(net_task(wifi_runner)).ok();
        //
        // wait_for_connection(network_stack).await;
        //
        // spawner.spawn(page_downloader(network_stack, tls_seed)).ok();
        // info!("we are connected. on to the HTTP request");
    } else {
        // PAGE_CHANNEL
        //     .sender()
        //     .send(Page::from_bytes(PAGE_BYTES, "homepage.html"))
        //     .await;
    }

    spawner.spawn(update_display(wrapper)).ok();

    Timer::after(Duration::from_millis(1000)).await;
    PAGE_CHANNEL
        .sender()
        .send(Page::from_bytes(PAGE_BYTES, "homepage.html"))
        .await;
}
async fn wait_for_connection(stack: Stack<'_>) {
    info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("start connection task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                info!("waiting to be disconnected");
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        info!("wifi state is {:?}", esp_wifi::wifi::wifi_state());
        // DISCONNECTED
        info!(
            "we are disconnected. is started = {:?}",
            controller.is_started()
        );
        if !matches!(controller.is_started(), Ok(true)) {
            if SSID.is_none() {
                warn!("SSID is none. did you forget to set the SSID environment variables");
                NET_STATUS
                    .send(NetStatus::Info("SSID is missing".to_string()))
                    .await;
            }
            if PASSWORD.is_none() {
                warn!("PASSWORD is none. did you forget to set the PASSWORD environment variables");
                NET_STATUS
                    .send(NetStatus::Info("PASSWORD is missing".to_string()))
                    .await;
            }
            let client_config = Configuration::Client(ClientConfiguration {
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("Starting wifi");
            // initializing stack
            NET_STATUS.send(NetStatus::InitializingStack()).await;
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
        info!("Scan");
        NET_STATUS.send(NetStatus::Scanning()).await;
        // scan for longer and show hidden
        let active = Active {
            min: core::time::Duration::from_millis(50),
            max: core::time::Duration::from_millis(100),
        };
        // scanning
        let mut result = controller
            .scan_with_config_async(ScanConfig {
                show_hidden: true,
                scan_type: active,
                ..Default::default()
            })
            .await
            .unwrap();
        // sort by best signal strength first
        result.sort_by(|a, b| a.signal_strength.cmp(&b.signal_strength));
        result.reverse();
        // for ap in result.iter() {
        //     // info!("found AP: {:?}", ap);
        // }
        // pick the first that matches the passed in SSID
        let ap = result
            .iter()
            .filter(|ap| ap.ssid.eq_ignore_ascii_case(SSID.unwrap()))
            .next();
        if let Some(ap) = ap {
            info!("using the AP {:?}", ap);
            // set the config to use for connecting
            controller
                .set_configuration(&Configuration::Client(ClientConfiguration {
                    ssid: ap.ssid.to_string(),
                    password: PASSWORD.unwrap().into(),
                    ..Default::default()
                }))
                .unwrap();

            info!("About to connect");
            NET_STATUS.send(NetStatus::Connecting()).await;
            match controller.connect_async().await {
                Ok(_) => {
                    info!("Wifi connected!");
                    NET_STATUS.send(NetStatus::Connected()).await;
                    loop {
                        info!("checking if we are still connected");
                        if let Ok(conn) = controller.is_connected() {
                            if conn {
                                info!("Connected successfully");
                                info!("sleep until we aren't connected anymore");
                                Timer::after(Duration::from_millis(5000)).await
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to connect to wifi: {e:?}");
                    Timer::after(Duration::from_millis(5000)).await
                }
            }
        } else {
            let ssid = SSID.unwrap();
            info!("did not find the ap for {ssid}");
            NET_STATUS
                .send(NetStatus::Info(format!("{ssid} not found")))
                .await;
            info!("looping around");
        }
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn update_display(mut wrapper: Wrapper) {
    let mut scene = make_gui_scene();
    let mut app:AppState = AppState {
        theme: &LIGHT_THEME,
        font: &FONT_7X13,
        bold_font: &FONT_7X13_BOLD,
    };

    let handlers: Vec<Callback<Rgb565, MonoFont>> = vec![];

    let mut last_touch_event: Option<gt911::Point> = None;
    scene.set_focused(PAGE_VIEW);
    loop {

        if let Ok(page) = PAGE_CHANNEL.try_receive() {
            if let Some(state) = scene.get_view_state::<PageView>(PAGE_VIEW) {
                info!("page got a new page: {:?}", page);
                state.load_page(page);
            }
            scene.mark_dirty_view(PAGE_VIEW);
            info!("heap is {}", esp_alloc::HEAP.stats());
        }
        if let Ok(status) = NET_STATUS.try_receive() {
            info!("got the status {status:?}");
            let txt = match &status {
                NetStatus::Info(txt) => txt,
                _ => &format!("{:?}", status).to_string(),
            };
            if let Some(overlay) = scene.get_view_mut("overlay-status") {
                overlay.title = txt.into();
            }
        }

        if let Ok(point) = wrapper.touch.get_touch(&mut wrapper.i2c) {
            if let None = &point {
                if let Some(point) = last_touch_event {
                    let pt = GPoint::new(320 - point.y as i32, 240 - point.x as i32);
                    // scene.mark_dirty_view("touch-overlay");
                    // if let Some(overlay) = scene.get_view_mut("touch-overlay") {
                    //     overlay.bounds = overlay.bounds.center_at(pt.x, pt.y);
                    //     scene.mark_dirty_view("touch-overlay");
                    // }
                    let res = click_at(&mut scene, &vec![], pt);
                    if let Some((target, action)) = res {
                        handle_action2(&target, &action, &mut scene, &mut app)
                    }
                }
            }
            last_touch_event = point;
        }
        if let Some(key) = wrapper.poll_keyboard() {
            if let Some((target, action)) = type_at_focused(&mut scene, &vec![], key) {
                handle_action2(&target, &action, &mut scene, &mut app)
            }
            update_view_from_keyboard_input(&mut scene, key);
        }

        wrapper.poll_trackball();
        if wrapper.click.changed {
            if let Some((target, action)) = action_at_focused(&mut scene, &handlers) {
                handle_action2(&target, &action, &mut scene, &mut app)
            }
        }
        if wrapper.up.changed {
            scroll_at_focused(&mut scene, &handlers, 0, -1);
        }
        if wrapper.down.changed {
            scroll_at_focused(&mut scene, &handlers, 0, 1);
        }
        let mut ctx: EmbeddedDrawingContext = EmbeddedDrawingContext::new(&mut wrapper.display);
        ctx.clip = scene.dirty_rect.clone();
        let theme:Theme<Rgb565, MonoFont> = Theme {
            bg: app.theme.base_bg,
            fg: app.theme.base_fg,
            panel_bg: app.theme.base_bg,
            font: app.font.clone(),
            bold_font: app.bold_font.clone(),
        };
        layout_scene(&mut scene);
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
