use embedded_graphics::geometry::{Dimensions, Point, Size};
use alloc::boxed::Box;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use log::info;
use core::any::Any;
use alloc::string::{String, ToString};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::Drawable;
use crate::common::TDeckDisplay;
use crate::gui::{base_background_color, base_button_background_color, base_font, base_text_color, GuiEvent, View};

pub struct Panel {
    pub bounds: Rectangle,
}

impl View for Panel {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle) {
        self.bounds.intersection(clip)
            .into_styled(PrimitiveStyle::with_fill(base_background_color))
            .draw(display)
            .unwrap();
    }

    fn handle_input(&mut self, event: GuiEvent) {
    }
}

impl Panel {
    pub fn new(bounds: Rectangle) -> Box<dyn View> {
        Box::new(Panel {
            bounds,
        })
    }
}

pub struct Label {
    text:String,
    position:Point,
}

impl View for Label {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rectangle {
        Rectangle {
            top_left:self.position,
            size: Size::new(50,20),
        }
    }

    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle) {
        let style = MonoTextStyle::new(&base_font, base_text_color);
        let text = Text::new(&self.text, self.position, style);
        if !text.bounding_box().intersection(clip).is_zero_sized() {
            text.draw(display).unwrap();
        }
    }

    fn handle_input(&mut self, event: GuiEvent) {
    }
}

impl Label {
    pub fn new(text: &str, p1: Point) -> Box<Label> {
        Box::new(Label {
            text:text.to_string(),
            position: p1,
        })
    }
}

pub struct Button {
    text:String,
    position:Point,
}

impl Button {
    pub fn new(text: &str, position: Point) -> Box<Button> {
        Box::new(Button {
            text: text.to_string(),
            position,
        })
    }
}

impl View for Button {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bounds(&self) -> Rectangle {
        let style = MonoTextStyle::new(&base_font, base_text_color);
        let bounds = Text::new(&self.text, self.position, style).bounding_box();
        bounds
    }

    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle) {
        self.bounds().intersection(clip)
            .into_styled(PrimitiveStyle::with_fill(base_button_background_color))
            .draw(display).unwrap();
        let style = MonoTextStyle::new(&base_font, base_text_color);
        let text = Text::new(&self.text, self.position, style);
        if !text.bounding_box().intersection(clip).is_zero_sized() {
            text.draw(display).unwrap();
        }
    }

    fn handle_input(&mut self, event: GuiEvent) {
        info!("button got input: {:?}", event);
    }
}