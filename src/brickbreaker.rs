use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, Theme, View};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Primitive, RgbColor, Transform, WebColors};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;
use log::info;

pub struct Brick {
    pub bounds: Rectangle,
    pub color: Rgb565,
    pub active: bool,
}
pub struct GameView {
    pub bounds: Rectangle,
    pub paddle: Rectangle,
    pub old_paddle: Rectangle,
    pub visible: bool,
    pub count: i32,
    pub ball_bounds: Rectangle,
    pub ball_velocity: Point,
    pub bricks: Vec<Brick>,
}
impl GameView {
    pub fn new() -> Box<Self> {
        let colors = [Rgb565::GREEN, Rgb565::YELLOW, Rgb565::CSS_ORANGE, Rgb565::RED];
        let mut bricks:Vec<Brick> = Vec::new();
        for i in 0..6 {
            for j in 0..4 {
                bricks.push(Brick {
                    bounds: Rectangle::new(Point::new(40 + i * 40 as i32, 20+ j * 20 as i32), Size::new(35, 15)),
                    color: colors[(j as usize) % colors.len()],
                    active: true,
                })
            }
        }
        Box::new(GameView {
            bounds: Rectangle::new(Point::new(0, 0), Size::new(200, 200)),
            paddle: Rectangle::new(Point::new(100, 100), Size::new(50, 10)),
            old_paddle: Rectangle::new(Point::new(100, 100), Size::new(50, 10)),
            visible: true,
            count: 0,
            ball_bounds: Rectangle::new(Point::new(58, 90), Size::new(10, 10)),
            ball_velocity: Point::new(2, 1),
            bricks,
        })
    }
}

impl GameView {
    pub(crate) fn handle_collisions(&mut self) {
        self.ball_bounds = self.ball_bounds.translate(self.ball_velocity);

        // collide with bricks
        for brick in &mut self.bricks {
            if !self.ball_bounds.intersection(&brick.bounds).is_zero_sized() {
                brick.active = false;
            }
        }

        // collide with the screen edges
        if self.ball_bounds.top_left.y >= 240 - 20 {
            self.ball_velocity = Point::new(self.ball_velocity.x, -self.ball_velocity.y);
        }
        if self.ball_bounds.top_left.y <= 0 {
            self.ball_velocity = Point::new(self.ball_velocity.x, -self.ball_velocity.y);
        }
        if self.ball_bounds.top_left.x >= 320 - 20 {
            self.ball_velocity = Point::new(-self.ball_velocity.x, self.ball_velocity.y);
        }
        if self.ball_bounds.top_left.x <= 0 {
            self.ball_velocity = Point::new(-self.ball_velocity.x, self.ball_velocity.y);
        }

        // collide with the paddle
        let inter = self.ball_bounds.intersection(&self.paddle);
        if !inter.is_zero_sized() {
            self.ball_velocity = Point::new(self.ball_velocity.x, -self.ball_velocity.y);
        }

    }
}


impl View for GameView {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }
    fn draw(&mut self, display: &mut TDeckDisplay, _clip: &Rectangle, _theme: &Theme) {
        self.count = self.count + 1;

        let old_ball_bounds = self.ball_bounds;
        self.handle_collisions();

        // draw background
        if self.count < 10 {
            let screen = Rectangle::new(Point::new(0, 0), Size::new(320, 240));
            screen
                .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
                .draw(display)
                .unwrap();
        }

        for brick in &self.bricks {
            if brick.active {
                brick.bounds.into_styled(PrimitiveStyle::with_fill(brick.color)).draw(display).unwrap();
            } else {
                brick.bounds.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();
            }
        }


        // draw the ball
        old_ball_bounds
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)
            .unwrap();
        self.ball_bounds
            .into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA))
            .draw(display)
            .unwrap();

        // draw the paddle
        self.old_paddle
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)
            .unwrap();
        self.paddle
            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
            .draw(display)
            .unwrap();
    }

    fn handle_input(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::ScrollEvent(_pt, delta) => {
                self.old_paddle = self.paddle;
                self.paddle = self.paddle.translate(Point::new(delta.x * 20, 0));
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

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
