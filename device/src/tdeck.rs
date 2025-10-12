use alloc::format;
use alloc::string::ToString;
use crate::common::{NetStatus, TDeckDisplay, NET_STATUS};
use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_wifi::{init};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTargetExt, Point as EGPoint, Primitive, Size as EGSize};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_hal_bus::spi::RefCellDevice;
use esp_hal::analog::adc::{Adc, AdcConfig, AdcPin, Attenuation};
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{BusTimeout, Config, Error, I2c};
use esp_hal::peripherals::Peripherals;
use esp_hal::peripherals::{ADC1, GPIO4, RNG, TIMG0, WIFI};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Blocking;
use gt911::{Error as Gt911Error, Gt911Blocking, Point as TouchPoint};
use heapless::Vec;
use log::{error, info, warn};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::Builder;
use iris_ui::geom::Bounds;
use static_cell::StaticCell;

use esp_hal::rng::Rng;
use esp_wifi::EspWifiController;
use esp_wifi::wifi::{ClientConfiguration, Configuration, ScanConfig, WifiController, WifiDevice, WifiEvent, WifiState};
use esp_wifi::wifi::ScanTypeConfig::Active;

const LILYGO_KB_I2C_ADDRESS: u8 = 0x55;

const SSID: Option<&str> = option_env!("SSID");
const PASSWORD: Option<&str> = option_env!("PASSWORD");

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

pub struct Wrapper {
    pub display: TDeckDisplay,
    pub i2c: I2c<'static, Blocking>,
    pub delay: Delay,
    adc: Adc<'static, ADC1<'static>, Blocking>,
    battery_pin: AdcPin<GPIO4<'static>, ADC1<'static>>,
    pub left: TrackballPin,
    pub right: TrackballPin,
    pub up: TrackballPin,
    pub down: TrackballPin,
    pub click: TrackballPin,
    pub touch: Gt911Blocking<I2c<'static, Blocking>>,
    pub wifi: Option<WIFI<'static>>,
    pub timg0: Option<TIMG0<'static>>,
    pub rng: Option<RNG<'static>>,
    // pub volume_mgr: VolumeManager<SdCard<RefCellDevice<'static, Spi<'static, Blocking>,Output<'static>, Delay>,Delay>, DummyTimesource>,
}

pub struct TrackballPin {
    pin: Input<'static>,
    prev: bool,
    pub changed: bool,
}
impl TrackballPin {
    fn poll(&mut self) {
        self.changed = false;
        if self.pin.is_high() != self.prev {
            self.prev = self.pin.is_high();
            self.changed = true;
        }
    }
}

impl Wrapper {
    pub fn poll_keyboard(&mut self) -> Option<u8> {
        let mut data = [0u8; 1];
        let kb_res = self.i2c.read(LILYGO_KB_I2C_ADDRESS, &mut data);
        match kb_res {
            Ok(_) => {
                if data[0] != 0x00 {
                    Some(data[0])
                } else {
                    None
                }
            }
            Err(_e) => None,
        }
    }

    pub fn read_battery_level(&mut self) -> u16 {
        let pin_value: u16 = self.adc.read_blocking(&mut self.battery_pin);
        info!("bat adc is {pin_value} ");
        pin_value
    }

    pub fn poll_trackball(&mut self) {
        self.left.poll();
        self.right.poll();
        self.up.poll();
        self.down.poll();
        self.click.poll();
    }

    pub fn poll_touchscreen(&mut self) -> Result<Vec<TouchPoint, 5>, Gt911Error<Error>> {
        self.touch.get_multi_touch(&mut self.i2c)
    }
}

static SPI_BUS: StaticCell<RefCell<Spi<Blocking>>> = StaticCell::new();

// pub struct DummyTimesource();

// impl TimeSource for DummyTimesource {
//     // In theory you could use the RTC of the rp2040 here, if you had
//     // any external time synchronizing device.
//     fn get_timestamp(&self) -> Timestamp {
//         Timestamp {
//             year_since_1970: 0,
//             zero_indexed_month: 0,
//             zero_indexed_day: 0,
//             hours: 0,
//             minutes: 0,
//             seconds: 0,
//         }
//     }
// }

impl Wrapper {
    pub fn init(peripherals: Peripherals) -> Wrapper {
        let mut delay = Delay::new();

        // have to turn on the board and wait 500ms before using the keyboard
        let mut board_power = Output::new(peripherals.GPIO10, High, OutputConfig::default());
        board_power.set_high();
        delay.delay_millis(1000);

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
            SpiConfig::default().with_frequency(Rate::from_mhz(40)), // .with_mode(Mode::_0)
        )
        .unwrap()
        .with_sck(tft_sck)
        .with_miso(tft_miso)
        .with_mosi(tft_mosi);

        info!("setting up the display");
        let spi_delay = Delay::new();
        // let spi_device = ExclusiveDevice::new(spi, tft_cs, spi_delay).unwrap();
        let shared_spi_bus = RefCell::new(spi);
        let shared_spi_bus = SPI_BUS.init(shared_spi_bus);

        let tft_device = RefCellDevice::new(shared_spi_bus, tft_cs, spi_delay)
            .expect("failed to create spi device");
        // let mut buffer = [0u8; 512];
        static DISPLAY_BUF: StaticCell<[u8; 512]> = StaticCell::new();
        let buffer = DISPLAY_BUF.init([0u8; 512]);
        let di = SpiInterface::new(tft_device, tft_dc, buffer);
        info!("building");
        let display = Builder::new(ST7789, di)
            .display_size(240, 320)
            .invert_colors(ColorInversion::Inverted)
            .color_order(ColorOrder::Rgb)
            .orientation(Orientation::new().rotate(Rotation::Deg90))
            .init(&mut delay)
            .unwrap();

        info!("initialized display");

        // let BOARD_SDCARD_CS = peripherals.GPIO39;
        // let sdmmc_cs = Output::new(BOARD_SDCARD_CS, High, OutputConfig::default());
        // let sdcard_device = RefCellDevice::new(shared_spi_bus, sdmmc_cs, spi_delay).expect("failed to create spi device");
        // let sdcard = SdCard::new(sdcard_device, delay);
        // let mut volume_mgr = VolumeManager::new(sdcard, DummyTimesource {});

        // initialize keyboard
        let mut i2c = I2c::new(
            peripherals.I2C0,
            Config::default()
                .with_frequency(Rate::from_khz(100))
                .with_timeout(BusTimeout::Disabled),
        )
        .unwrap()
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO8);

        // initialize battery monitor
        let analog_pin = peripherals.GPIO4;
        let mut adc_config: AdcConfig<ADC1> = AdcConfig::new();
        let pin: AdcPin<GPIO4, ADC1> = adc_config.enable_pin(analog_pin, Attenuation::_11dB);

        let touch = Gt911Blocking::default();
        touch.init(&mut i2c).unwrap();

        info!("returning finished wrapper");
        // set up the trackball button pins

        let timer = TimerGroup::new(peripherals.TIMG1).timer0;
        esp_hal_embassy::init(timer);

        Wrapper {
            display,
            i2c,
            delay,
            touch,
            wifi: Some(peripherals.WIFI),
            timg0: Some(peripherals.TIMG0),
            rng: Some(peripherals.RNG),
            adc: Adc::new(peripherals.ADC1, adc_config),
            battery_pin: pin,
            left: TrackballPin {
                changed: false,
                prev: false,
                pin: Input::new(
                    peripherals.GPIO1,
                    InputConfig::default().with_pull(Pull::Up),
                ),
            },
            right: TrackballPin {
                changed: false,
                prev: false,
                pin: Input::new(
                    peripherals.GPIO2,
                    InputConfig::default().with_pull(Pull::Up),
                ),
            },
            up: TrackballPin {
                changed: false,
                prev: false,
                pin: Input::new(
                    peripherals.GPIO3,
                    InputConfig::default().with_pull(Pull::Up),
                ),
            },
            down: TrackballPin {
                changed: false,
                prev: false,
                pin: Input::new(
                    peripherals.GPIO15,
                    InputConfig::default().with_pull(Pull::Up),
                ),
            },
            click: TrackballPin {
                changed: false,
                prev: false,
                pin: Input::new(
                    peripherals.GPIO0,
                    InputConfig::default().with_pull(Pull::Up),
                ),
            },
        }
    }

    pub async fn start_wifi(&mut self, spawner: &Spawner) {
        let Some(rngg) = self.rng.take() else { return ; };
        let Some(timg0) = self.timg0.take() else { return; };
        let Some(wifi) = self.wifi.take() else { return; };
        let mut rng = Rng::new(rngg);
        let timer_g0 = TimerGroup::new(timg0);

        info!("made timer");
        let esp_wifi_ctrl = &*mk_static!(
            EspWifiController<'static>,
            init(timer_g0.timer0, rng.clone()).unwrap()
        );
        info!("making controller");
        let (wifi_controller, interfaces) =
            esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();
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

        // spawner.spawn(page_downloader(network_stack, tls_seed)).ok();
        info!("we are connected. on to the HTTP request");

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
        for ap in result.iter() {
            info!("found AP: {:?}", ap);
        }
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
