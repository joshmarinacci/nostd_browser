use crate::common::TDeckDisplay;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::{vec};
use alloc::vec::Vec;
use core::any::{Any, TypeId};
use core::fmt::{Debug, Formatter};
use core::ops::Add;
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, Primitive, RgbColor, Size, WebColors};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use hashbrown::HashMap;
use log::{info, warn};
use crate::textview::TextView;

pub trait View {
    fn draw(&mut self, display: &mut TDeckDisplay);
    fn handle_input(&mut self, event:GuiEvent);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl Debug for Box<dyn View> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let val = self as &dyn Any;
        info!("is menu {}",TypeId::of::<MenuView>() == val.type_id());
        info!("is box menuview {}",TypeId::of::<Box<MenuView>>() == val.type_id());
        info!("is box view {}",TypeId::of::<Box<dyn View>>() == val.type_id());
        match val.downcast_ref::<Box<MenuView>>() {
            Some(menu) => {
                write!(f, "is a menu view")
            }
            None => {
                write!(f, "some other object")
            }
        }
    }
}

pub struct MenuView {
    pub id: String,
    pub items: Vec<String>,
    pub position: Point,
    pub highlighted_index: usize,
    pub visible: bool,
    pub dirty: bool,
    // pub callback: Option<Box<dyn FnMut(&mut MenuView, &str) + 'a>>,
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
    pub fn new(id: &str, items: Vec<&str>, p1: Point) -> Box<MenuView> {
        Box::new(MenuView {
            id: id.into(),
            items: items.iter().map(|s| s.to_string()).collect(),
            position: p1,
            highlighted_index: 0,
            visible: true,
            dirty: true,
        })
    }
    pub fn start_hidden(id: &str, items: Vec<&str>, p1: Point) -> Box<MenuView> {
        Box::new(MenuView {
            id: id.into(),
            items: items.iter().map(|s| s.to_string()).collect(),
            position: p1,
            highlighted_index: 0,
            visible: false,
            dirty: true,
        })
    }
    // fn draw(&mut self, display: &mut TDeckDisplay) {
    //     if !self.visible {
    //         return;
    //     }
    //     if !self.dirty {
    //         return;
    //     }
    //     let font = FONT_9X15;
    //     let lh = font.character_size.height as i32;
    //     let pad = 5;
    //     let rect = Rectangle::new(
    //         self.position,
    //         Size::new(100, (self.items.len() as i32 * lh + pad * 2) as u32),
    //     );
    //     rect.into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_LIGHT_GRAY))
    //         .draw(display)
    //         .unwrap();
    //     // info!("Highlighted index {}", self.highlighted_index);
    //     for (i, item) in self.items.iter().enumerate() {
    //         let bg = if i == self.highlighted_index {
    //             Rgb565::RED
    //         } else {
    //             Rgb565::WHITE
    //         };
    //         let fg = if i == self.highlighted_index {
    //             Rgb565::WHITE
    //         } else {
    //             Rgb565::RED
    //         };
    //         let ly = (i as i32) * lh + pad;
    //         Rectangle::new(
    //             Point::new(pad, ly).add(self.position),
    //             Size::new(100, lh as u32),
    //         )
    //         .into_styled(PrimitiveStyle::with_fill(bg))
    //         .draw(display)
    //         .unwrap();
    //         let text_style = MonoTextStyle::new(&font, fg);
    //         Text::new(
    //             &item,
    //             Point::new(pad, ly + lh - 2).add(self.position),
    //             text_style,
    //         )
    //         .draw(display)
    //         .unwrap();
    //     }
    //     // self.dirty = false;
    // }
}
impl View for MenuView {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(&mut self, display: &mut TDeckDisplay) {
        if !self.visible { return; }
        // info!("MenuView draw");
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
    }

    fn handle_input(&mut self, event: GuiEvent) {
        // info!("Handling key event: {:?}", event);
        match event {
            GuiEvent::KeyEvent(key) => {
                match key {
                    b'j' => self.nav_prev(),
                    b'k' => self.nav_next(),
                    _ => {}
                }
            }
        }
    }
}

pub struct Scene {
    pub views:Vec<Box<dyn View>>,
    pub focused:Option<i32>,
    pub keys:HashMap<String,i32>,
    dirty: bool
}

impl Scene {
    pub fn add(&mut self, name: &str, main_menu: Box<MenuView>) {
        self.views.push(main_menu);
        self.keys.insert(name.to_string(), (self.views.len() as i32)-1);
    }
}

impl Scene {
    pub fn is_menu_selected(&self, index: i32, hi: usize) -> bool {
        if let Some(menu) = self.views[index as usize].as_any().downcast_ref::<MenuView>() {
            return menu.highlighted_index == hi;
        }
        false
    }
    pub fn is_menu_selected_by_name(&self, name: &str, hi:usize) -> bool {
        if let Some(index) = self.keys.get(name) {
            return self.is_menu_selected(*index, hi)
        }
        return false
    }
    pub fn hide_menu(&mut self, index: i32) {
        if let Some(menu) = self.views[index as usize].as_any_mut().downcast_mut::<MenuView>() {
            menu.visible = false;
            self.dirty = true;
            self.set_focused(0);
        }
    }
    pub fn hide_menu_by_name(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            self.hide_menu(*index);
        }
    }
    pub fn show_menu(&mut self, index: i32) {
        if let Some(menu) = self.views[index as usize].as_any_mut().downcast_mut::<MenuView>() {
            menu.visible = true;
            self.dirty = true;
            self.set_focused(index);
        }
    }
    pub fn show_menu_by_name(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            self.show_menu(*index);
        } else {
            warn!("Missing menu by name '{}'", name);
        }
    }
    pub fn get_menu_at(&self, index: i32) -> Option<&MenuView> {
        self.views[index as usize].as_any().downcast_ref::<MenuView>()
    }
    pub fn get_menu_by_name(&self, name: &str) -> Option<&MenuView> {
        if let Some(index) = self.keys.get(name) {
            self.get_menu_at(*index)
        } else {
            warn!("Missing menu by name '{}'", name);
            None
        }
    }
    pub fn get_textview_at(&self, index: i32) -> Option<&TextView> {
        self.views[index as usize].as_any().downcast_ref::<TextView>()
    }
    pub fn get_textview_at_mut(&mut self, index: i32) -> Option<&mut TextView>  {
        self.views[index as usize].as_any_mut().downcast_mut::<TextView>()
    }
    pub fn get_textview_at_mut_by_name(&mut self, name:&str) -> Option<&mut TextView>  {
        if let Some(index) = self.keys.get(name) {
            self.views[*index as usize].as_any_mut().downcast_mut::<TextView>()
        } else {
            None
        }
    }
    pub fn is_focused(&self, p0: i32) -> bool {
        if let Some(f) = self.focused {
            if f == p0 {
                return true
            }
        }
        false
    }
    pub fn is_focused_by_name(&self, name: &str) -> bool {
        if let Some(f) = self.keys.get(name) {
            return self.is_focused(*f)
        }
        false
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn mark_dirty(&mut self) {
        self.dirty = true
    }
    pub fn new() -> Scene {
        Scene {
            dirty: true,
            views: vec![],
            focused: None,
            keys: HashMap::new(),
        }
    }
    pub fn handle_event(&mut self, evt: GuiEvent) {
        if let Some(index) = self.focused {
            if index < self.views.len() as i32 {
                let view = &mut self.views[index as usize];
                view.handle_input(evt);
            }
        }
        self.dirty = true;
    }
    pub fn set_focused(&mut self, index:i32) {
        self.focused = Some(index);
    }
    pub fn set_focused_by_name(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            self.set_focused(*index);
        }
    }

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
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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

#[derive(Debug, Copy, Clone)]
pub enum GuiEvent {
    KeyEvent(u8),
}

