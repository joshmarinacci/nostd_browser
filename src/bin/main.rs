#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::{vec};
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::text::Text;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{BusTimeout, Config, I2c};
use esp_hal::rng::Rng;
use esp_hal::time::Rate;
use esp_hal::spi::{ master::{Spi, Config as SpiConfig } };
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::{ EspWifiController,  init };
use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState};
use log::{info, warn};
use reqwless::client::{HttpClient, TlsConfig};

use mipidsi::{models::ST7789, Builder};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use nostd_html_parser::{Tag, TagParser};
use static_cell::StaticCell;
use nostd_browser::common::TDeckDisplay;
use nostd_browser::gui::{CompoundMenu, GuiEvent, MenuView, Scene, VButton, VLabel};
use nostd_browser::textview::{break_lines, LineStyle, TextLine, TextRun, TextView};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

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

pub const LILYGO_KB_I2C_ADDRESS: u8 =     0x55;

static I2C:StaticCell<I2c<Blocking>> = StaticCell::new();

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
        Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Disabled),
    )
        .unwrap()
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO8);
    info!("initialized I2C keyboard");
    let ic2_ref = I2C.init(i2c);


    let max_chars = 30;
    let mut lines:Vec<TextLine> = vec![];
    lines.append(&mut break_lines("Thoughts on LLMs and the coming AI backlash", max_chars-4,LineStyle::Header));
    lines.push(TextLine {
        runs: vec![TextRun {
            style:LineStyle::Plain,
            text:"".into(),
        }]
    });
    lines.append(&mut break_lines(r#"I find Large Language Models fascinating.
    They are a very different approach to AI than most of the 60 years of
    AI research and show great promise. At the same time they are just technology.
    They aren't magic. They aren't even very good technology yet. LLM hype has vastly
    outpaced reality and I think we are due for a correction, possibly even a bubble pop.
    Furthermore, I think future AI progress is going to happen on the app / UX side,
    not on the core models, which are already starting to show their scaling limits.
    Let's dig in. Better pour a cup of coffee. This could be a long one."#, max_chars-4,LineStyle::Plain));

    let textview = TextView {
        dirty: true,
        lines: lines,
    };

    // set up the display
    {
        // set TFT CS to high
        let mut tft_cs = Output::new(peripherals.GPIO12, High, OutputConfig::default());
        tft_cs.set_high();
        let tft_miso = Input::new(peripherals.GPIO38, InputConfig::default().with_pull(Pull::Up));
        let tft_sck = peripherals.GPIO40;
        let tft_mosi = peripherals.GPIO41;
        let tft_dc = Output::new(peripherals.GPIO11, Low, OutputConfig::default());
        let mut tft_enable = Output::new(peripherals.GPIO42, High, OutputConfig::default());
        tft_enable.set_high();

        info!("creating spi device");
        info!("heap is {}", esp_alloc::HEAP.stats());
        let spi = Spi::new(peripherals.SPI2, SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
                           // .with_mode(Mode::_0)
        ).unwrap()
            .with_sck(tft_sck)
            .with_miso(tft_miso)
            .with_mosi(tft_mosi)
            ;
        static DISPLAY_BUF:StaticCell<[u8;512]> = StaticCell::new();
        let buffer = DISPLAY_BUF.init([0u8; 512]);

        info!("setting up the display");
        let spi_delay = Delay::new();
        let spi_device = ExclusiveDevice::new(spi, tft_cs, spi_delay).unwrap();
        let di = SpiInterface::new(spi_device, tft_dc, buffer);
        info!("building");
        let display = Builder::new(ST7789, di)
            // .reset_pin(tft_enable)
            .display_size(240,320)
            .invert_colors(ColorInversion::Inverted)
            .color_order(ColorOrder::Rgb)
            .orientation(Orientation::new().rotate(Rotation::Deg90))
            // .display_size(320,240)
            .init(&mut delay).unwrap();
        static DISPLAY:StaticCell<TDeckDisplay> = StaticCell::new();
        let display_ref = DISPLAY.init(display);
        info!("initialized display");

        let menu:CompoundMenu = setup_menu();
        let scene = make_gui_scene();
        spawner.spawn(update_display(display_ref, menu, ic2_ref, textview, scene)).ok();

    }


    let mut rng = Rng::new(peripherals.RNG);
    let timer_g0 = TimerGroup::new(peripherals.TIMG0);

    info!("made timer");
    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timer_g0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );
    info!("making controller");
    let (wifi_controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();
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

    info!("we are connected. on to the HTTP request");
    {
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
        info!("heap is {}", esp_alloc::HEAP.stats());
        let mut http_req = client
            .request(
                reqwless::request::Method::GET,
                // "https://joshondesign.com/2023/07/12/css_text_style_builder",
                // "https://jsonplaceholder.typicode.com/posts/1",
                "https://apps.josh.earth/",

            )
            .await
            .unwrap();
        let response = http_req.send(&mut buffer).await.unwrap();

        info!("Got response");
        let res = response.body().read_to_end().await.unwrap();

        // let content = core::str::from_utf8(res).unwrap();
        // info!("content {}", content);
        let mut parser:TagParser = TagParser::with_debug(res, false);
        let lines:Vec<TextLine> = make_lines(&mut parser);
        info!("=== rendered lines === ");
        for line in lines {
            for run in line.runs {
                info!("{:?}: {}", run.style, run.text);
            }
        }
        info!("heap is {}", esp_alloc::HEAP.stats());
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
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
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            if SSID.is_none() {
                warn!("SSID is none. did you forget to set the SSID environment variables");
            }
            if PASSWORD.is_none() {
                warn!("PASSWORD is none. did you forget to set the PASSWORD environment variables");
            }
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.unwrap().into(),
                password: PASSWORD.unwrap().into(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");

            info!("Scan");
            let result = controller.scan_n_async(10).await.unwrap();
            for ap in result {
                info!("{:?}", ap);
            }
        }
        info!("About to connect to ... {:?}",SSID);

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                info!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn update_display(display: &'static mut TDeckDisplay, mut menu: CompoundMenu<'static>, i2c:&'static mut I2c<'static, Blocking>, mut textview: TextView, mut scene:Scene) {
    loop {
        let mut data = [0u8; 1];
        let kb_res = (*i2c).read(LILYGO_KB_I2C_ADDRESS, &mut data);
        match kb_res {
            Ok(_) => {
                if data[0] != 0x00 {
                    info!("kb_res = {:?}", String::from_utf8_lossy(&data));
                    let evt:GuiEvent = GuiEvent::KeyEvent(data[0]);
                    scene.handle_event(evt);
                    // menu.handle_key_event(data[0]);
                    // if menu.is_menu_visible("main") {
                    //     menu.handle_key_event(data[0]);
                    // } else {
                    //     if data[0] == b' ' {
                    //         menu.open_menu("main");
                    //     }
                    // }
                }
            }
            Err(_) => {
                // info!("kb_res = {}", e);
            }
        }

         if scene.is_dirty() {
            display.clear(Rgb565::WHITE).unwrap();
            // textview.draw(display);
            // menu.draw(display);
            scene.draw(display);
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

fn make_lines(parser:&mut TagParser) -> Vec<TextLine> {
    let mut lines = Vec::new();
    let mut inside_paragraph = false;
    let mut para = TextLine {
        runs: Vec::new(),
    };
    for tag in parser {
        // info!("TAG: {:?} {}", tag, inside_paragraph);
        match tag {
            Tag::Comment(_) => {}
            Tag::Open(name) => {
                info!("TAG: {} {}", tag, inside_paragraph);
                let name = String::from_utf8_lossy(name);
                if name.eq_ignore_ascii_case("h1")
                    || name.eq_ignore_ascii_case("h2")
                    || name.eq_ignore_ascii_case("h3")
                    || name.eq_ignore_ascii_case("p")
                {
                    inside_paragraph = true;
                }
            }
            Tag::Close(name) => {
                info!("TAG: {} {}", tag, inside_paragraph);
                let name = String::from_utf8_lossy(name);
                if name.eq_ignore_ascii_case("h1")
                    || name.eq_ignore_ascii_case("h2")
                    || name.eq_ignore_ascii_case("h3")
                    || name.eq_ignore_ascii_case("p")
                {
                    inside_paragraph = false;
                    lines.push(para);
                    para = TextLine {
                        runs: vec![],
                    };
                }
            }
            Tag::Text(txt) => {
                info!("TAG: {} {}", tag, inside_paragraph);
                if inside_paragraph {
                    para.runs.push(TextRun {
                        text:String::from_utf8_lossy(txt).to_string(),
                        style: LineStyle::Header,
                    })
                }
            }
            _ => {}
        }
    }
    lines
}


fn setup_menu<'a>() -> CompoundMenu<'a> {
    let main_menu = MenuView {
        id:"main",
        dirty: true,
        items: vec!["Theme","Font","Wifi","Bookmarks","close"],
        position: Point::new(0,0),
        highlighted_index: 0,
        visible: true,
        callback: None,
    };
    let theme_menu = MenuView {
        id:"themes",
        dirty:true,
        items: vec!["Dark", "Light", "close"],
        position: Point::new(20,20),
        highlighted_index: 0,
        visible: false,
        callback: None,
    };
    let mut menu = CompoundMenu {
        menus: vec![],
        focused: "main",
        callback: Some(Box::new(|comp, menu, cmd| {
            info!("menu {} cmd {}",menu,cmd);
            if menu == "main" {
                if cmd == "Theme" {
                    comp.open_menu("themes");
                }
                if cmd == "Font" {
                    comp.open_menu("fonts");
                }
                if cmd == "Wifi" {
                    comp.open_menu("wifi");
                }
                if cmd == "Bookmarks" {
                    comp.open_menu("bookmarks");
                }
                if cmd == "close" {
                    comp.hide();
                }
            }
            if menu == "themes" {
                if cmd == "Dark" {
                }
                if cmd == "Light" {
                }
                if cmd == "close" {
                    comp.hide_menu("themes");
                }
            }
        })),
        dirty: true,
    };
    menu.add_menu(main_menu);
    menu.add_menu(theme_menu);
    menu
}

fn make_gui_scene<'a>() -> Scene {
    let mut scene = Scene::new();
    scene.views.push(VLabel::new("foo"));
    scene.views.push(VButton::new("bar"));
    scene.set_focused(0);
    scene
}














