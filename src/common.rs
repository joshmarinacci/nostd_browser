use crate::page::Page;
use alloc::string::String;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::{Display, NoResetPin};

pub type TDeckDisplay = Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    NoResetPin,
>;
// pub type TDeckDisplay = dyn DrawTarget<Color=Rgb565, Error=core::convert::Infallible>;

pub static PAGE_CHANNEL: Channel<CriticalSectionRawMutex, Page, 2> = Channel::new();

#[derive(Debug)]
pub enum NetCommand {
    Load(String),
}

pub static NET_COMMANDS: Channel<CriticalSectionRawMutex, NetCommand, 2> = Channel::new();

#[derive(Debug)]
pub enum NetStatus {
    Offline(),
    InitializingStack(),
    Scanning(),
    Connecting(),
    Connected(),
    LoadingPage(),
    PageLoaded(),
    Error(String),
    Info(String),
}

pub static NET_STATUS: Channel<CriticalSectionRawMutex, NetStatus, 2> = Channel::new();
