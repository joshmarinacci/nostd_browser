use embedded_graphics::geometry::{Dimensions, Point, Size};
use alloc::boxed::Box;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use log::info;
use core::any::Any;
use alloc::string::{String, ToString};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor, WebColors};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::Drawable;
use alloc::vec::Vec;
use embedded_graphics::mono_font::ascii::FONT_9X15;
use core::ops::Add;
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

pub struct MenuView {
    pub items: Vec<String>,
    pub position: Point,
    pub highlighted_index: usize,
    pub visible: bool,
    pub dirty: bool,
}

impl MenuView {
    pub(crate) fn nav_prev(&mut self) {
        self.highlighted_index = (self.highlighted_index + 1) % self.items.len();
        self.dirty = true;
    }
    pub(crate) fn nav_next(&mut self) {
        self.highlighted_index = (self.highlighted_index + self.items.len() - 1) % self.items.len();
        self.dirty = true;
    }
    pub fn new(items: Vec<&str>, p1: Point) -> Box<MenuView> {
        Box::new(MenuView {
            items: items.iter().map(|s| s.to_string()).collect(),
            position: p1,
            highlighted_index: 0,
            visible: true,
            dirty: true,
        })
    }
    pub fn start_hidden(items: Vec<&str>, p1: Point) -> Box<MenuView> {
        Box::new(MenuView {
            items: items.iter().map(|s| s.to_string()).collect(),
            position: p1,
            highlighted_index: 0,
            visible: false,
            dirty: true,
        })
    }
    fn size(&self) -> Size {
        let font = FONT_9X15;
        let line_height = (font.character_size.height + 2) as i32;
        return Size::new(
            100 + 2 * 2,
            (self.items.len() as i32 * line_height + 2 * 2) as u32,
        );
    }
}

impl View for MenuView {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn bounds(&self) -> Rectangle {
        Rectangle {
            top_left: self.position,
            size: self.size().add(Size::new(10, 10)),
        }
    }
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle) {
        if !self.visible {
            return;
        }
        let line_height = (base_font.character_size.height + 2) as i32;
        let pad = 5;

        let xoff: i32 = 2;
        let yoff: i32 = 2;
        let menu_size = self.size();
        // menu background
        let shadow =
            Rectangle::new(self.position.add(Point::new(10, 10)), menu_size).intersection(clip);
        shadow
            .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_LIGHT_GRAY))
            .draw(display)
            .unwrap();
        let background =
            Rectangle::new(self.position.add(Point::new(5, 5)), menu_size).intersection(clip);
        background
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)
            .unwrap();
        for (i, item) in self.items.iter().enumerate() {
            let bg = if i == self.highlighted_index {
                Rgb565::BLACK
            } else {
                Rgb565::WHITE
            };
            let fg = if i == self.highlighted_index {
                Rgb565::WHITE
            } else {
                base_text_color
            };
            let line_y = (i as i32) * line_height + pad;
            Rectangle::new(
                Point::new(pad + xoff, line_y + yoff).add(self.position),
                Size::new(100, line_height as u32),
            )
            .intersection(clip)
            .into_styled(PrimitiveStyle::with_fill(bg))
            .draw(display)
            .unwrap();
            let text_style = MonoTextStyle::new(&base_font, fg);
            let pos = Point::new(pad + xoff, line_y + line_height - 2 + yoff).add(self.position);
            let text_bounds = Text::new(item, pos, text_style).bounding_box();
            if !text_bounds.intersection(clip).is_zero_sized() {
                Text::new(&item, pos, text_style).draw(display).unwrap();
            }
        }
    }
    fn handle_input(&mut self, event: GuiEvent) {
        // info!("Handling key event: {:?}", event);
        match event {
            GuiEvent::KeyEvent(key) => match key {
                b'j' => self.nav_prev(),
                b'k' => self.nav_next(),
                _ => {}
            },
            GuiEvent::PointerEvent(pt,delta) => {
                // info!("menu got {pt} {delta}");
                if delta.y < 0 {
                    self.nav_next();
                }
                if delta.y > 0 {
                    self.nav_prev();
                }
            }
        }
    }
}