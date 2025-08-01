use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, Theme, View};
use alloc::boxed::Box;
use core::any::Any;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Primitive, RgbColor};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use log::info;

pub struct GameView {
    pub bounds: Rectangle,
    pub paddle_pos: Point,
    pub paddle_size: i32,
    pub visible: bool,
}

impl GameView {
    pub fn new() -> Box<Self> {
        Box::new(GameView {
            bounds: Rectangle::new(Point::new(0, 0), Size::new(200, 200)),
            paddle_pos: Point::new(100,100),
            paddle_size: 30,
            visible: true,
        })
    }
}

impl View for GameView {
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle, theme: &Theme) {
        // info!("drawing the game view");
        let screen = Rectangle::new(Point::new(0,0), Size::new(200, 200));
        screen.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
        let rect = Rectangle::with_center(self.paddle_pos, Size::new(self.paddle_size as u32, 20));
        rect.into_styled(PrimitiveStyle::with_fill(Rgb565::RED)).draw(display).unwrap();
    }

    fn handle_input(&mut self, event: GuiEvent) {
        // info!("received event: {:?}", event);
        match event {
            GuiEvent::PointerEvent(pt,delta) => {
                // info!("GAME: pointer event {pt} {delta}");
                self.paddle_pos.x += delta.x*5;
                // self.paddle_pos.x = self.paddle_pos.x.max(self.paddle_size);
                // self.paddle_pos.x = self.paddle_pos.x.min(200-self.paddle_size);
            }
            GuiEvent::KeyEvent(key) => {
                info!("GAME: key event: {:?}", key);
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }
}
