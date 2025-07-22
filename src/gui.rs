use crate::common::TDeckDisplay;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Add;
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, Primitive, RgbColor, Size, WebColors};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use log::info;

pub struct MenuView<'a> {
    pub id: &'a str,
    pub items: Vec<&'a str>,
    pub position: Point,
    pub highlighted_index: usize,
    pub visible: bool,
    pub dirty: bool,
    pub callback: Option<Box<dyn FnMut(&mut MenuView, &str) + 'a>>,
}

impl<'a> MenuView<'a> {
    pub(crate) fn handle_key_event(&mut self, key: u8) {
        info!("Handling key event: {}", key);
        match key {
            b'j' => self.nav_prev(),
            b'k' => self.nav_next(),
            _ => {}
        }
    }
    pub(crate) fn is_visible(&self) -> bool {
        self.visible
    }
    pub(crate) fn show(&mut self) {
        self.visible = true;
        self.dirty = true;
        self.highlighted_index = 0;
    }
    pub(crate) fn hide(&mut self) {
        self.visible = false;
        self.dirty = true;
    }
    pub(crate) fn nav_prev(&mut self) {
        self.highlighted_index = (self.highlighted_index + 1) % self.items.len();
        self.dirty = true;
    }
    pub(crate) fn nav_next(&mut self) {
        self.highlighted_index = (self.highlighted_index + self.items.len() - 1) % self.items.len();
        self.dirty = true;
    }
    fn new(id: &'a str, items: &Vec<&'a str>, p1: Point) -> MenuView<'a> {
        MenuView {
            id: id,
            items: items.to_vec(),
            position: p1,
            highlighted_index: 0,
            visible: false,
            dirty: true,
            callback: None,
        }
    }
    fn draw(&mut self, display: &mut TDeckDisplay) {
        if !self.visible {
            return;
        }
        if !self.dirty {
            return;
        }
        let font = FONT_9X15;
        let lh = font.character_size.height as i32;
        let pad = 5;
        let rect = Rectangle::new(
            self.position,
            Size::new(100, (self.items.len() as i32 * lh + pad * 2) as u32),
        );
        rect.into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_LIGHT_GRAY))
            .draw(display)
            .unwrap();
        // info!("Highlighted index {}", self.highlighted_index);
        for (i, item) in self.items.iter().enumerate() {
            let bg = if i == self.highlighted_index {
                Rgb565::RED
            } else {
                Rgb565::WHITE
            };
            let fg = if i == self.highlighted_index {
                Rgb565::WHITE
            } else {
                Rgb565::RED
            };
            let ly = (i as i32) * lh + pad;
            Rectangle::new(
                Point::new(pad, ly).add(self.position),
                Size::new(100, lh as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(bg))
            .draw(display)
            .unwrap();
            let text_style = MonoTextStyle::new(&font, fg);
            Text::new(
                &item,
                Point::new(pad, ly + lh - 2).add(self.position),
                text_style,
            )
            .draw(display)
            .unwrap();
        }
        // self.dirty = false;
    }
}

pub struct CompoundMenu<'a> {
    pub menus: Vec<MenuView<'a>>,
    pub focused: &'a str,
    pub callback: Option<Box<dyn FnMut(&mut CompoundMenu, &str, &str) + 'a>>,
    pub dirty: bool,
}

impl<'a> CompoundMenu<'a> {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn hide_menu(&mut self, id: &str) {
        let menu = self.menus.iter_mut().find(|m| m.id == id);
        if let Some(menu) = menu {
            menu.hide();
            self.focused = "main";
        }
        self.dirty = true;
    }
    pub fn open_menu(&mut self, id: &str) {
        let menu = self.menus.iter_mut().find(|m| m.id == id);
        if let Some(menu) = menu {
            menu.show();
            self.focused = menu.id;
        }
        self.dirty = true;
    }
    pub fn is_menu_visible(&self, id: &str) -> bool {
        let menu = self.menus.iter().find(|m| m.id == id);
        if let Some(menu) = menu {
            return menu.is_visible();
        }
        false
    }
    pub fn hide(&mut self) {
        for menu in &mut self.menus {
            menu.hide();
        }
        self.dirty = true;
    }
    pub fn add_menu(&mut self, menu: MenuView<'a>) {
        self.menus.push(menu);
        self.dirty = true;
    }
    pub fn handle_key_event(&mut self, key: u8) {
        info!("compound handling key event {}", key);
        if key == b'\r' {
            let menu = self.menus.iter().find(|m| m.id == self.focused);
            if let Some(menu) = menu {
                let cmd = menu.items[menu.highlighted_index];
                info!("triggering action for {}", cmd);
                let mut callback = self.callback.take().unwrap();
                callback(self, menu.id, cmd);
                self.callback = Some(callback);
            }
        } else {
            let menu = self.menus.iter_mut().find(|m| m.id == self.focused);
            if let Some(menu) = menu {
                menu.handle_key_event(key);
            }
        }
        self.dirty = true;
    }
    pub fn draw(&mut self, display: &mut TDeckDisplay) {
        self.dirty = false;
        for menu in &mut self.menus {
            menu.draw(display);
        }
    }
}











pub trait View {
    fn draw(&mut self, display: &mut TDeckDisplay);
    fn handle_input(&mut self, event:GuiEvent);
}

pub struct Scene {
    pub views:Vec<Box<dyn View>>,
    pub focused:Option<usize>,
    dirty: bool
}

impl Scene {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            dirty: true,
            views: vec![],
            focused: None,
        }
    }
}

impl Scene {
    pub fn handle_event(&mut self, evt: GuiEvent) {
        if let Some(index) = self.focused {
            if index < self.views.len() {
                let view = &mut self.views[index];
                view.handle_input(evt);
            }
        }
        self.dirty = true;
    }
    pub fn set_focused(&mut self, index:usize) {
        self.focused = Some(index);
    }
}

impl Scene {
    pub fn draw(&mut self, display: &mut TDeckDisplay) {
        self.views.iter_mut().for_each(|v| v.draw(display));
        self.dirty = false;
    }
}

pub struct VButton {
    text:String,
    visible: bool,
}

impl VButton {
    pub fn new(label: &str) -> Box<dyn View> {
        Box::new(VButton{
            visible:true,
            text: label.to_string(),
        })
    }
}

impl View for VButton {
    fn draw(&mut self, display: &mut TDeckDisplay) {
        if !self.visible { return; }
        info!("vbutton draw {}", self.text);
        let style = MonoTextStyle::new(&FONT_9X15, Rgb565::CSS_BLACK);
        Text::new(&self.text, Point::new(20,20), style).draw(display).unwrap();
    }

    fn handle_input(&mut self, event:GuiEvent) {
        match event {
            GuiEvent::KeyEvent(key) => {
                info!("vbutton key event: {:?}", key);
                self.visible = true;
            }
        }
    }
}
pub struct VLabel {
    text:String,
    visible: bool,
}

impl VLabel {
    pub fn new(label: &str) -> Box<dyn View> {
        Box::new(VLabel {
            visible:true,
            text: label.to_string(),
        })
    }
}

impl View for VLabel {
    fn draw(&mut self, display: &mut TDeckDisplay) {
        if !self.visible { return; }
        info!("vlabel draw {}", self.text);
        let style = MonoTextStyle::new(&FONT_9X15, Rgb565::CSS_BLACK);
        Text::new(&self.text, Point::new(20,50), style).draw(display).unwrap();
    }
    fn handle_input(&mut self, event:GuiEvent) {
        info!("vlabel handle_input {:?}",event);
        self.visible = true;
    }
}

#[derive(Debug)]
pub enum GuiEvent {
    KeyEvent(u8),
}

