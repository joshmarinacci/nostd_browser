use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::ops::{Add, Sub};
use embedded_graphics::geometry::{AnchorPoint, Dimensions, Point, Size};
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::mono_font::{MonoTextStyle};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor, WebColors};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::StrokeAlignment::Inside;
use embedded_graphics::primitives::{
    PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment,
};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;
use log::{info, warn};
use crate::{GuiEvent, Theme, View, ViewTarget, BASE_FONT};

pub struct Panel {
    pub bounds: Rectangle,
    visible: bool,
    children: Vec<String>,
}

impl Panel {
    pub fn new(bounds: Rectangle) -> Box<Panel> {
        Box::new(Panel {
            bounds,
            visible: true,
            children: Vec::new(),
        })
    }
    pub fn add_child(&mut self, child: String) {
        self.children.push(child);
    }
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

    fn draw(&mut self, display: &mut dyn ViewTarget, clip: &Rectangle, theme: &Theme) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(theme.base_bd)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .fill_color(theme.base_bg)
            .build();
        display.rect(&self.bounds,style);
    }

    fn handle_input(&mut self, _event: GuiEvent) {}

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
        info!("panel laying out children {:?}", self.children);
        // for child in &self.children {
        // if let Some(view) = scene.get_view_mut() {
        //
        // }
        // }
    }
}


pub struct Label {
    text: String,
    position: Point,
    visible: bool,
}

impl View for Label {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {

    }

    fn bounds(&self) -> Rectangle {
        Rectangle {
            top_left: self.position,
            size: Size::new(50, 20),
        }
    }

    fn draw(&mut self, display: &mut dyn ViewTarget, clip: &Rectangle, theme: &Theme) {
        let style = MonoTextStyle::new(&theme.font, theme.base_fg);
        display.text(&self.text,&self.position,style);
    }

    fn handle_input(&mut self, _: GuiEvent) {}
}

impl Label {
    pub fn new(text: &str, p1: Point) -> Box<Label> {
        Box::new(Label {
            text: text.to_string(),
            position: p1,
            visible: true,
        })
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

pub struct Button {
    text: String,
    position: Point,
    bounds: Rectangle,
    visible: bool,
}

impl Button {
    pub fn new(text: &str, position: Point) -> Box<Button> {
        Box::new(Button {
            text: text.to_string(),
            position,
            bounds: Rectangle::new(Point::new(20, 20), Size::new(20, 20)),
            visible: true,
        })
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl View for Button {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
        let style = MonoTextStyle::new(&BASE_FONT, Rgb565::BLACK);
        let bounds = Text::new(&self.text, self.position, style).bounding_box();
        let bigger = bounds.size.add(Size::new(20, 20));
        self.bounds = bounds.resized(bigger, AnchorPoint::Center)
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn draw(&mut self, display: &mut dyn ViewTarget, clip: &Rectangle, theme: &Theme) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(theme.base_bd)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .fill_color(theme.base_bg)
            .build();

        display.rect(&self.bounds,style);
        let style = MonoTextStyle::new(&theme.font, theme.base_fg);
        display.text(&self.text,&self.position,style);
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

const PAD: u32 = 5;
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
        let line_height = (font.character_size.height + 2) as u32;
        let len = self.items.len() as u32;
        Size::new(100 + PAD * 2, len * line_height + PAD * 2)
    }
}

impl View for MenuView {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
    }

    fn bounds(&self) -> Rectangle {
        Rectangle {
            top_left: self.position,
            size: self.size().add(Size::new(10, 10)),
        }
    }
    fn draw(&mut self, display: &mut dyn ViewTarget, clip: &Rectangle, theme: &Theme) {
        if !self.visible {
            return;
        }
        let line_height = (theme.font.character_size.height + 2) as i32;
        let pad = PAD as i32;

        let xoff: i32 = 2;
        let yoff: i32 = 0;
        let menu_size = self.size();
        // menu background
        if theme.shadow {
            let shadow_style = PrimitiveStyle::with_fill(Rgb565::CSS_LIGHT_GRAY);
            display.rect(&Rectangle::new(self.position.add(Point::new(5, 5)), menu_size), shadow_style);
        }

        let panel_style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_alignment(Inside)
            .stroke_color(theme.base_bd)
            .fill_color(theme.base_bg)
            .build();
        display.rect(&Rectangle::new(self.position,menu_size),panel_style);
        for (i, item) in self.items.iter().enumerate() {
            let line_y = (i as i32) * line_height + pad;

            if i == self.highlighted_index {
                display.rect(&Rectangle::new(
                    Point::new(pad, line_y + yoff).add(self.position),
                    Size::new(100, line_height as u32),
                ),PrimitiveStyle::with_fill(theme.base_bd));
            };
            let fg = if i == self.highlighted_index {
                theme.base_bg
            } else {
                theme.base_fg
            };
            let text_style = MonoTextStyle::new(&theme.font, fg);
            let pos = Point::new(pad + xoff, line_y + line_height - 3 + yoff).add(self.position);
            let text_bounds = Text::new(item, pos, text_style).bounding_box();
            display.text(item,&pos,text_style);
        }
    }
    fn handle_input(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::KeyEvent(key) => match key {
                b'j' => self.nav_prev(),
                b'k' => self.nav_next(),
                _ => {}
            },
            GuiEvent::ScrollEvent(_, delta) => {
                if delta.y < 0 {
                    self.nav_next();
                }
                if delta.y > 0 {
                    self.nav_prev();
                }
            }
            GuiEvent::TouchEvent(pt) => {
                let pos = pt.sub(self.position);
                let line_height = (BASE_FONT.character_size.height + 2) as i32;
                let index:usize = (pos.y / line_height) as usize;
                if  index < self.items.len() {
                    self.highlighted_index = index;
                    self.dirty = true;
                }
            }
            _ => {
                warn!("unhandled event: {:?}", event);
            }
        }
    }
}

pub struct OverlayLabel {
    text: String,
    bounds: Rectangle,
    visible: bool,
}
impl OverlayLabel {
    pub fn new(text: &str, bounds: Rectangle) -> Box<OverlayLabel> {
        Box::new(OverlayLabel {
            text: text.to_string(),
            bounds,
            visible: true,
        })
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}
impl View for OverlayLabel {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn draw(&mut self, display: &mut dyn ViewTarget, clip: &Rectangle, theme: &Theme) {
        let style = PrimitiveStyleBuilder::new()
            .fill_color(theme.base_fg)
            .build();

        display.rect(&self.bounds,style);
        let style = MonoTextStyle::new(&theme.font, theme.base_bg);
        display.text(&self.text,&self.bounds.center(),style);
    }

    fn handle_input(&mut self, event: GuiEvent) {
        info!("button got input: {:?}", event);
    }
}


pub struct TextInput {
    pub text: String,
    pub bounds: Rectangle,
    pub visible: bool,
}

impl TextInput {
    pub fn new(text: &str, bounds: Rectangle) -> Box<TextInput> {
        Box::new(TextInput {
            text:String::from(text),
            bounds,
            visible: true,
        })
    }
}
impl View for TextInput {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn draw(&mut self, display: &mut dyn ViewTarget, _clip: &Rectangle, theme: &Theme) {
        let bounds_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::WHITE)
            .stroke_color(Rgb565::BLACK)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();
        display.rect(&self.bounds, bounds_style);

        let text_style = MonoTextStyle::new(&theme.font, theme.base_fg);
        display.text(&self.text,&self.bounds.top_left.add(Point::new(5,20)),text_style)
    }

    fn handle_input(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::KeyEvent(key) => {
                match key {
                    30..126 => {
                        info!("printable key: {:?}", key);
                        self.text.push_str(&String::from_utf8_lossy(&[key]))
                    },
                    13 => {
                        info!("text input return key")
                    }
                    8 => {
                        info!("backspace");
                        self.text.pop();
                    }
                    0_u8..=29_u8 | 126_u8..=u8::MAX => {
                        info!("unprintable key: {:?}", key);
                    }
                }
            }
            _ => {}
        }
    }
}