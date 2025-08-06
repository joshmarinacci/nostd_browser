#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
extern crate alloc;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{format, vec};
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Runner, Stack, StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
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
use mipidsi::{models::ST7789, Builder};
use nostd_browser::brickbreaker::GameView;
use nostd_browser::common::{
    NetCommand, NetStatus, TDeckDisplay, NET_COMMANDS, NET_STATUS, PAGE_CHANNEL,
};
use nostd_browser::comps::{Button, Label, MenuView, OverlayLabel, Panel};
use nostd_browser::gui::{GuiEvent, Scene, View, DARK_THEME, LIGHT_THEME};
use nostd_browser::page::Page;
use nostd_browser::pageview::PageView;
use nostd_html_parser::blocks::{Block, BlockType};
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
        static DISPLAY: StaticCell<TDeckDisplay> = StaticCell::new();
        let display_ref = DISPLAY.init(display);
        info!("initialized display");

        let scene = make_gui_scene();
        spawner
            .spawn(update_display(display_ref, ic2_ref, scene))
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
        info!("wifi state is {:?}",esp_wifi::wifi::wifi_state());
        // DISCONNECTED
        info!("we are disconnected. is started = {:?}", controller.is_started());
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
            NET_STATUS.send(NetStatus::Info(format!("{ssid} not found"))).await;
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
async fn update_display(
    display: &'static mut TDeckDisplay,
    i2c: &'static mut I2c<'static, Blocking>,
    mut scene: Scene,
) {
    loop {
        let display_width = display.size().width;
        let font = FONT_9X15;
        let char_width = font.character_size.width as i32;
        let columns = ((display_width as i32) / char_width) as u32;
        // info!("width is {} char width = {} columns is {}", display_width, char_width, columns);
        if let Ok(page) = PAGE_CHANNEL.try_receive() {
            if let Some(tv) = scene.get_textview_mut("page") {
                tv.load_page(page, columns);
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
                    update_view_from_input(evt, &mut scene, display).await;
                }
            }
            Err(_) => {
                // info!("kb_res = {}", e);
            }
        }

        if let Ok(evt) = TRACKBALL_CHANNEL.try_receive() {
            update_view_from_input(evt, &mut scene, display).await;
        }

        if let Ok(status) = NET_STATUS.try_receive() {
            // info!("got the status {status:?}");
            let txt = match &status {
                NetStatus::Info(txt) => txt,
                _ => &format!("{:?}", status).to_string(),
            };
            scene.mutate_view("status",|view|{
                if let Some(overlay) = view.as_any_mut().downcast_mut::<OverlayLabel>() {
                    overlay.set_text(txt);
                };
            });
        }

        scene.draw(display);
        Timer::after(Duration::from_millis(20)).await;
    }
}

const MAIN_MENU: &'static str = "main";
const THEME_MENU: &'static str = "theme";
const PAGE_VIEW: &'static str = "page";

async fn update_view_from_input(event: GuiEvent, scene: &mut Scene, display: &TDeckDisplay) {
    // info!("update view from input {:?}", event);
    if scene.focused.is_none() {
        scene.focused = Some("".to_string());
    }
    if let Some(menu) = scene.get_menu(MAIN_MENU) {
        if menu.visible {
            scene.handle_event(event);
        } else {
            match event {
                GuiEvent::KeyEvent(evt) => {
                    if evt == b' ' {
                        scene.show_menu(MAIN_MENU);
                    } else {
                        scene.mutate_view(PAGE_VIEW, |view| {
                            view.handle_input(event);
                        });
                    }
                }
                _ => {
                    scene.handle_event(event);
                }
            }
        }
    }

    let panel_bounds = Rectangle::new(
        Point::new(20, 20),
        Size::new(
            display.bounding_box().size.width - 40,
            display.bounding_box().size.height - 40,
        ),
    );
    match event {
        GuiEvent::KeyEvent(key_event) => match key_event {
            13 => {
                // scene.info();
                if scene.is_focused(MAIN_MENU) {
                    if scene.menu_equals(MAIN_MENU, "Theme") {
                        scene.show_menu(THEME_MENU);
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "Font") {
                        scene.show_menu("font");
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "Wifi") {
                        let panel = Panel::new(panel_bounds);
                        let label1a = Label::new("SSID", Point::new(60, 80));
                        let label1b = Label::new(SSID.unwrap_or("----"), Point::new(150, 80));
                        let label2a = Label::new("PASSWORD", Point::new(60, 100));
                        let label2b = Label::new(PASSWORD.unwrap_or("----"), Point::new(150, 100));
                        let button = Button::new("done", Point::new(160 - 20, 200 - 20));

                        scene.add("wifi-panel", panel);
                        scene.add("wifi-label1a", label1a);
                        scene.add("wifi-label1b", label1b);
                        scene.add("wifi-label2a", label2a);
                        scene.add("wifi-label2b", label2b);
                        scene.add("wifi-button", button);
                        scene.hide(MAIN_MENU);
                        scene.set_focused("wifi-button");
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "Bookmarks") {
                        // show the bookmarks
                        NET_COMMANDS
                            .send(NetCommand::Load("bookmarks.html".to_string()))
                            .await;
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "Info") {
                        info!("showing the info panel");
                        let panel = Panel::new(panel_bounds);
                        scene.add("info-panel", panel);

                        let free = esp_alloc::HEAP.free();
                        let used = esp_alloc::HEAP.used();
                        scene.add("info-label1", Label::new("Heap", Point::new(120, 50)));
                        scene.add(
                            "info-label2a",
                            Label::new("Free  memory ", Point::new(60, 80)),
                        );
                        scene.add(
                            "info-label2b",
                            Label::new(&format!("{:?}", free), Point::new(200, 80)),
                        );
                        scene.add(
                            "info-label3a",
                            Label::new("Used  memory ", Point::new(60, 100)),
                        );
                        scene.add(
                            "info-label3b",
                            Label::new(&format!("{:?}", used), Point::new(200, 100)),
                        );
                        scene.add(
                            "info-label4a",
                            Label::new("Total memory", Point::new(60, 120)),
                        );
                        scene.add(
                            "info-label4b",
                            Label::new(&format!("{:?}", free + used), Point::new(200, 120)),
                        );

                        let button = Button::new("done", Point::new(160 - 20, 200 - 20));
                        scene.add("info-button", button);
                        scene.hide(MAIN_MENU);
                        scene.set_focused("info-button");
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "Font") {
                        scene.show_menu("font");
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "Bricks") {
                        scene.add("game", GameView::new());
                        scene.hide(MAIN_MENU);
                        scene.set_focused("game");
                        scene.set_auto_redraw(true);
                        scene.hide(PAGE_VIEW);
                        return;
                    }
                    if scene.menu_equals(MAIN_MENU, "close") {
                        scene.hide(MAIN_MENU);
                        return;
                    }
                    return;
                }
                if scene.is_focused("wifi-button") {
                    info!("clicked the button");
                    scene.remove("wifi-panel");
                    scene.remove("wifi-label1a");
                    scene.remove("wifi-label1b");
                    scene.remove("wifi-label2a");
                    scene.remove("wifi-label2b");
                    scene.remove("wifi-button");
                    return;
                }
                if scene.is_focused("info-button") {
                    info!("clicked the info button");
                    scene.remove("info-panel");
                    scene.remove("info-label1");
                    scene.remove("info-label2a");
                    scene.remove("info-label2b");
                    scene.remove("info-label3a");
                    scene.remove("info-label3b");
                    scene.remove("info-label4a");
                    scene.remove("info-label4b");
                    scene.remove("info-button");
                    return;
                }
                if scene.is_focused("theme") {
                    if scene.menu_equals(THEME_MENU, "Dark") {
                        scene.set_theme(DARK_THEME);
                        return;
                    }
                    if scene.menu_equals(THEME_MENU, "Light") {
                        scene.set_theme(LIGHT_THEME);
                        return;
                    }
                    if scene.menu_equals(THEME_MENU, "close") {
                        scene.hide(THEME_MENU);
                        scene.set_focused(MAIN_MENU);
                        return;
                    }
                }
                if scene.is_focused("font") {
                    // close
                    if scene.menu_equals("font", "close") {
                        scene.hide("font");
                        scene.set_focused(MAIN_MENU);
                        return;
                    }
                }
                if scene.is_focused("wifi") {
                    if scene.menu_equals("wifi", "close") {
                        scene.hide("wifi");
                        scene.set_focused(MAIN_MENU);
                        return;
                    }
                }
            }
            _ => {}
        },
        GuiEvent::ScrollEvent(_, _) => {}
        GuiEvent::ClickEvent() => {
            info!("clicked the button");
        }
    }
}

fn make_gui_scene<'a>() -> Scene {
    let mut scene = Scene::new();
    let full_screen_bounds = Rectangle {
            top_left: Point::new(0, 0),
            size: Size::new(320, 240),
        };
    let textview = PageView::new(full_screen_bounds,  Page::new() );
    scene.add(PAGE_VIEW, Box::new(textview));
    scene.add(
        MAIN_MENU,
        MenuView::start_hidden(
            vec![
                "Theme",
                "Font",
                "Wifi",
                "Bookmarks",
                "Bricks",
                "Info",
                "close",
            ],
            Point::new(0, 0),
        ),
    );
    scene.add(
        THEME_MENU,
        MenuView::start_hidden(vec!["Light", "Dark", "close"], Point::new(20, 20)),
    );
    scene.add(
        "font",
        MenuView::start_hidden(vec!["small", "medium", "big", "close"], Point::new(20, 20)),
    );
    scene.add(
        "wifi",
        MenuView::start_hidden(vec!["status", "scan", "close"], Point::new(20, 20)),
    );

    // set up a fake page
    if let Some(tv) = scene.get_textview_mut(PAGE_VIEW) {
        let mut blocks = vec![];
        blocks.push(Block::new_of_type(BlockType::Header, "Header Text"));
        blocks.push(Block::new_of_type(BlockType::ListItem, "list item"));
        blocks.push(Block::new_of_type(
            BlockType::Paragraph,
            "This is some long body text that needs to be broken into multiple lines",
        ));
        let page = Page {
            selection: 0,
            blocks,
            links: vec![],
            url: "".to_string(),
        };
        tv.load_page(page, 30);
    }

    let info_panel_bounds = Rectangle::new(Point::new(120, 210), Size::new(200, 30));
    scene.add("status", OverlayLabel::new("some info", info_panel_bounds));
    scene
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
            TRACKBALL_CHANNEL.send(GuiEvent::ScrollEvent(cursor, Point::new(1, 0))).await;
        }
        if tdeck_trackball_left.is_high() != last_left_high {
            // info!("left");
            last_left_high = tdeck_trackball_left.is_high();
            cursor.x -= 1;
            TRACKBALL_CHANNEL.send(GuiEvent::ScrollEvent(cursor, Point::new(-1, 0))).await;
        }
        if tdeck_trackball_up.is_high() != last_up_high {
            // info!("up");
            last_up_high = tdeck_trackball_up.is_high();
            cursor.y -= 1;
            TRACKBALL_CHANNEL.send(GuiEvent::ScrollEvent(cursor, Point::new(0, -1))).await;
        }
        if tdeck_trackball_down.is_high() != last_down_high {
            // info!("down");
            last_down_high = tdeck_trackball_down.is_high();
            cursor.y += 1;
            TRACKBALL_CHANNEL.send(GuiEvent::ScrollEvent(cursor, Point::new(0, 1))).await;
        }
        // wait one msec
        Timer::after(Duration::from_millis(1)).await;
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
                        PAGE_CHANNEL
                            .sender()
                            .send(Page::from_bytes(PAGE_BYTES, &href))
                            .await;
                    } else {
                        if !href.starts_with("http") {
                            info!("relative url");
                        }
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
                        info!("making the actual request");
                        // let url = "https://joshondesign.com/2023/07/12/css_text_style_builder";
                        let mut http_req = client
                            .request(
                                reqwless::request::Method::GET,
                                &href,
                                // "https://jsonplaceholder.typicode.com/posts/1",
                                // "https://apps.josh.earth/",
                            )
                            .await
                            .unwrap();
                        let response = http_req.send(&mut buffer).await.unwrap();
                        info!("Got response");
                        let res = response.body().read_to_end().await.unwrap();
                        PAGE_CHANNEL
                            .sender()
                            .send(Page::from_bytes(res, &href))
                            .await;
                        NET_STATUS.send(NetStatus::PageLoaded()).await;
                    }
                }
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
