#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
use gui::View;
extern crate alloc;
use alloc::string::ToString;
use alloc::{format};
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Runner, Stack, StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_9X15};
use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{BusTimeout, Config, I2c};
use esp_hal::rng::Rng;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Blocking;
use esp_hal::xtensa_lx::interrupt::disable;
use esp_wifi::wifi::ScanTypeConfig::Active;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, ScanConfig, WifiController, WifiDevice, WifiEvent,
    WifiState,
};
use esp_wifi::{init, EspWifiController};
use log::{error, info, warn};
use reqwless::client::{HttpClient, TlsConfig};

use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder, Display, NoResetPin};
use nostd_browser::common::{NetCommand, NetStatus, TDeckDisplay, TDeckDisplayWrapper, NET_COMMANDS, NET_STATUS, PAGE_CHANNEL};
use nostd_browser::page::Page;
use static_cell::StaticCell;
use gui::{GuiEvent, Scene};
use gui::comps::OverlayLabel;
use nostd_browser::browser::{make_gui_scene, update_view_from_input};
use nostd_browser::pageview::PageView;

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

pub const LILYGO_KB_I2C_ADDRESS: u8 = 0x55;

static I2C: StaticCell<I2c<Blocking>> = StaticCell::new();

static PAGE_BYTES: &[u8] = include_bytes!("homepage.html");

pub(crate) static DISPLAY: StaticCell<TDeckDisplay> = StaticCell::new();
static DISPLAY_REF: StaticCell<&mut TDeckDisplay> = StaticCell::new();
static TRACKBALL_CHANNEL: Channel<CriticalSectionRawMutex, GuiEvent, 2> = Channel::new();
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 128 * 1024);
    info!("heap is {}", esp_alloc::HEAP.stats());

    info!("init-ting embassy");
    let timer_g1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_g1.timer0);

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
    let ic2_ref = I2C.init(i2c);

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
        info!("building");
        let display = Builder::new(ST7789, di)
            // .reset_pin(tft_enable)
            .display_size(240, 320)
            .invert_colors(ColorInversion::Inverted)
            .color_order(ColorOrder::Rgb)
            .orientation(Orientation::new().rotate(Rotation::Deg90))
            // .display_size(320,240)
            .init(&mut delay)
            .unwrap();
        info!("initialized display");

        let scene = make_gui_scene();
        spawner
            .spawn(update_display(display, ic2_ref, scene))
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
    info!("AUTO_CONNECT is {:?}", AUTO_CONNECT);
    if AUTO_CONNECT.is_some() {
        let mut rng = Rng::new(peripherals.RNG);
        let timer_g0 = TimerGroup::new(peripherals.TIMG0);

        info!("made timer");
        let esp_wifi_ctrl = &*mk_static!(
            EspWifiController<'static>,
            init(timer_g0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
        );
        info!("making controller");
        let (wifi_controller, interfaces) =
            esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();
        let wifi_interface = interfaces.sta;

        let config = embassy_net::Config::dhcpv4(Default::default());
        let net_seed = (rng.random() as u64) << 32 | rng.random() as u64;
        info!("made net seed {}", net_seed);
        let tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
        info!("made tls seed {}", tls_seed);

        info!("init-ing the network stack");
        // Init network stack
        let (network_stack, wifi_runner) = embassy_net::new(
            wifi_interface,
            config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            net_seed,
        );

        info!("spawning connection");
        spawner.spawn(connection(wifi_controller)).ok();
        info!("spawning net task");
        spawner.spawn(net_task(wifi_runner)).ok();

        wait_for_connection(network_stack).await;

        spawner.spawn(page_downloader(network_stack, tls_seed)).ok();
        info!("we are connected. on to the HTTP request");
    } else {
        PAGE_CHANNEL
            .sender()
            .send(Page::from_bytes(PAGE_BYTES, "homepage.html"))
            .await;
    }
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


fn get_pageview_mut<'a>(scene:&'a mut Scene, name:&str) -> Option<&'a mut PageView>  {
    if let Some(view) = scene.get_view_mut(name) {
        if let Some(tv) = view.as_any_mut().downcast_mut::<PageView>() {
            return Some(tv)
        }
    }
    None
}

    #[embassy_executor::task]
async fn update_display(
    display: TDeckDisplay,
    i2c: &'static mut I2c<'static, Blocking>,
    mut scene: Scene,
) {
    let display = DISPLAY.init(display);
    let mut wrapper = TDeckDisplayWrapper::new(display);
    loop {
        let display_width = wrapper.display.size().width;
        let font = FONT_9X15;
        let char_width = font.character_size.width as i32;
        let columns = ((display_width as i32) / char_width) as u32;
        // info!("width is {} char width = {} columns is {}", display_width, char_width, columns);
        if let Ok(page) = PAGE_CHANNEL.try_receive() {
            if let Some(tv) = get_pageview_mut(&mut scene,"page") {
                tv.load_page(page);
                let bounds = tv.bounds();
                scene.mark_dirty(bounds);
            }
            info!("heap is {}", esp_alloc::HEAP.stats());
        }
        let mut data = [0u8; 1];
        let kb_res = (*i2c).read(LILYGO_KB_I2C_ADDRESS, &mut data);
        match kb_res {
            Ok(_) => {
                if data[0] != 0x00 {
                    let evt: GuiEvent = GuiEvent::KeyEvent(data[0]);
                    update_view_from_input(evt, &mut scene, wrapper.display).await;
                }
            }
            Err(_) => {
                // info!("kb_res = {}", e);
            }
        }

        if let Ok(evt) = TRACKBALL_CHANNEL.try_receive() {
            update_view_from_input(evt, &mut scene, wrapper.display).await;
        }

        if let Ok(status) = NET_STATUS.try_receive() {
            // info!("got the status {status:?}");
            let txt = match &status {
                NetStatus::Info(txt) => txt,
                _ => &format!("{:?}", status).to_string(),
            };
            scene.mutate_view("status", |view| {
                if let Some(overlay) = view.as_any_mut().downcast_mut::<OverlayLabel>() {
                    overlay.set_text(txt);
                };
            });
        }

        scene.draw(&mut wrapper);
        Timer::after(Duration::from_millis(20)).await;
    }
}
#[embassy_executor::task]
async fn handle_trackball(
    tdeck_track_click: Input<'static>,
    tdeck_trackball_left: Input<'static>,
    tdeck_trackball_right: Input<'static>,
    tdeck_trackball_up: Input<'static>,
    tdeck_trackball_down: Input<'static>,
) {
    let mut last_click_low = false;
    let mut last_right_high = false;
    let mut last_left_high = false;
    let mut last_up_high = false;
    let mut last_down_high = false;
    info!("monitoring the trackball");
    let mut cursor = Point::new(50, 50);
    loop {
        if tdeck_track_click.is_low() != last_click_low {
            info!("click");
            last_click_low = tdeck_track_click.is_low();
            TRACKBALL_CHANNEL.send(GuiEvent::ClickEvent()).await;
        }
        // info!("button pressed is {} ", tdeck_track_click.is_low());
        if tdeck_trackball_right.is_high() != last_right_high {
            // info!("right");
            last_right_high = tdeck_trackball_right.is_high();
            cursor.x += 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(1, 0)))
                .await;
        }
        if tdeck_trackball_left.is_high() != last_left_high {
            // info!("left");
            last_left_high = tdeck_trackball_left.is_high();
            cursor.x -= 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(-1, 0)))
                .await;
        }
        if tdeck_trackball_up.is_high() != last_up_high {
            // info!("up");
            last_up_high = tdeck_trackball_up.is_high();
            cursor.y -= 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(0, -1)))
                .await;
        }
        if tdeck_trackball_down.is_high() != last_down_high {
            // info!("down");
            last_down_high = tdeck_trackball_down.is_high();
            cursor.y += 1;
            TRACKBALL_CHANNEL
                .send(GuiEvent::ScrollEvent(cursor, Point::new(0, 1)))
                .await;
        }
        // wait one msec
        Timer::after(Duration::from_millis(1)).await;
    }
}

async fn load_file_url(href:&str) -> &[u8] {
    PAGE_BYTES
}
async  fn handle_file_url(href:&str) {
    info!("sdcard url {}",href);
    let path = &href[5..];
    info!("loading path {}", path);


    let bytes = load_file_url(&href).await;
    PAGE_CHANNEL.sender().send(Page::from_bytes(bytes,&href)).await;
}

async fn handle_bookmarks(href:&str) {
    PAGE_CHANNEL
        .sender()
        .send(Page::from_bytes(PAGE_BYTES, &href))
        .await;
}

async fn handle_http_url(href:&str, network_stack: Stack<'static>, tls_seed: u64) {
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
        },
        Err(err) => {
            info!("Got error: {:?}", err);
            NET_STATUS.send(NetStatus::Error(format!("{:?}",err))).await;
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
