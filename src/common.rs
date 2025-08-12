use crate::page::Page;
use alloc::string::String;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::{Display, NoResetPin};
use crate::gui::ViewTarget;

pub type TDeckDisplay = Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    NoResetPin,
>;

impl ViewTarget for TDeckDisplay {
    fn size(&self) -> Size {
        self.bounding_box().size
    }

    fn text(&mut self, txt: &str, pos: &Point, style: MonoTextStyle<Rgb565>) {
        Text::new(&txt, pos.clone(), style).draw(self).unwrap();
    }

    fn rect(&mut self, rectangle: &Rectangle, style: PrimitiveStyle<Rgb565>) {
        rectangle.into_styled(style).draw(self).unwrap();
    }
}

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
