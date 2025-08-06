use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, Theme, View};
use alloc::boxed::Box;
use core::any::Any;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Primitive, RgbColor, Transform};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use log::info;

pub struct GameView {
    pub bounds: Rectangle,
    pub paddle: Rectangle,
    pub old_paddle: Rectangle,
    pub visible: bool,
    pub count: i32,
    pub ball_bounds: Rectangle,
    pub ball_velocity: Point,
}

impl GameView {
    pub fn new() -> Box<Self> {
        Box::new(GameView {
            bounds: Rectangle::new(Point::new(0, 0), Size::new(200, 200)),
            paddle: Rectangle::new(Point::new(100, 100), Size::new(50, 10)),
            old_paddle: Rectangle::new(Point::new(100, 100), Size::new(50, 10)),
            visible: true,
            count: 0,
            ball_bounds: Rectangle::new(Point::new(58, 90), Size::new(10, 10)),
            ball_velocity: Point::new(2,1),
        })
    }
}

impl View for GameView {
    fn draw(&mut self, display: &mut TDeckDisplay, _clip: &Rectangle, _theme: &Theme) {
        self.count = self.count + 1;

        let old_ball_bounds = self.ball_bounds;
        // self.ball_bounds = Rectangle::new(old_ball_bounds.top_left + self.ball_velocity, old_ball_bounds.size);
        self.ball_bounds = self.ball_bounds.translate(self.ball_velocity);

        // if off right edge
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

        let inter = self.ball_bounds.intersection(&self.paddle);
        if !inter.is_zero_sized() {
            self.ball_velocity = Point::new(self.ball_velocity.x, -self.ball_velocity.y);
        }

        // draw background
        if self.count < 10 {
            let screen = Rectangle::new(Point::new(0,0), Size::new(320, 240));
            screen.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
        }

        // draw the ball
        old_ball_bounds.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
        self.ball_bounds.into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA)).draw(display).unwrap();

        // draw the paddle
        self.old_paddle.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
        self.paddle.into_styled(PrimitiveStyle::with_fill(Rgb565::RED)).draw(display).unwrap();
    }

    fn handle_input(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::ScrollEvent(_pt, delta) => {
                self.old_paddle = self.paddle;
                self.paddle = self.paddle.translate(Point::new(delta.x*20, 0));
                if self.paddle.top_left.x < 0 {
                    self.paddle.top_left.x = 0;
                }
                if self.paddle.top_left.x + (self.paddle.size.width as i32) > 320 {
                    self.paddle.top_left.x = 320 - self.paddle.size.width as i32;
                }
                self.paddle.top_left.y = 200;
            }
            GuiEvent::KeyEvent(key) => {
                info!("GAME: key event: {:?}", key);
            }
            _ => {}
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

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
