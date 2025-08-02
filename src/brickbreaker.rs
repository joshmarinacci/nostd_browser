use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, Theme, View};
use alloc::boxed::Box;
use core::any::Any;
use core::ops::Add;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Primitive, RgbColor};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use log::info;

pub struct GameView {
    pub bounds: Rectangle,
    pub paddle_pos: Point,
    pub old_paddle_pos: Point,
    pub paddle_size: i32,
    pub visible: bool,
    pub count: i32,
    pub ball_bounds: Rectangle,
    pub ball_velocity: Point,
}

impl GameView {
    pub fn new() -> Box<Self> {
        Box::new(GameView {
            bounds: Rectangle::new(Point::new(0, 0), Size::new(200, 200)),
            paddle_pos: Point::new(100,100),
            old_paddle_pos: Point::new(100,0),
            paddle_size: 30,
            visible: true,
            count: 0,
            ball_bounds: Rectangle::new(Point::new(58, 90), Size::new(20, 20)),
            ball_velocity: Point::new(3,2),
        })
    }
}

impl View for GameView {
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle, theme: &Theme) {
        self.count = self.count + 1;
        if self.count % 10 == 0 {
            let screen = Rectangle::new(Point::new(0,0), Size::new(320, 240));
            screen.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
        }

        let old_ball_bounds = self.ball_bounds;
        self.ball_bounds = Rectangle::new(old_ball_bounds.top_left + self.ball_velocity, old_ball_bounds.size);

        if self.ball_bounds.top_left.y >= 240-20 {
            self.ball_velocity = Point::new(self.ball_velocity.x, -self.ball_velocity.y);
        }
        if self.ball_bounds.top_left.y <= 0 {
            self.ball_velocity = Point::new(self.ball_velocity.x, -self.ball_velocity.y);
        }
        if self.ball_bounds.top_left.x >= 320-20 {
            self.ball_velocity = Point::new(-self.ball_velocity.x, self.ball_velocity.y);
        }
        if self.ball_bounds.top_left.x <= 0 {
            self.ball_velocity = Point::new(-self.ball_velocity.x, self.ball_velocity.y);
        }


        old_ball_bounds.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
        self.ball_bounds.into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA)).draw(display).unwrap();

        // info!("drawing the game view");
        let old = Rectangle::with_center(self.old_paddle_pos, Size::new(self.paddle_size as u32, 20));
        old.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();

        let rect = Rectangle::with_center(self.paddle_pos, Size::new(self.paddle_size as u32, 20));
        rect.into_styled(PrimitiveStyle::with_fill(Rgb565::RED)).draw(display).unwrap();
    }

    fn handle_input(&mut self, event: GuiEvent) {
        // info!("received event: {:?}", event);
        match event {
            GuiEvent::PointerEvent(pt,delta) => {
                // info!("GAME: pointer event {pt} {delta}");
                self.old_paddle_pos.x = self.paddle_pos.x;
                self.old_paddle_pos.y = self.paddle_pos.y;
                let mut nx = self.paddle_pos.x + delta.x * 20;
                if nx < self.paddle_size as i32 {
                    nx = self.paddle_size
                }
                if nx > 320-self.paddle_size as i32 {
                    nx = 320-self.paddle_size
                }
                self.paddle_pos.x = nx;
                self.paddle_pos.y = 200;
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
