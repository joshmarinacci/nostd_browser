use crate::common::TDeckDisplay;
use crate::textview::TextView;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::{Any, TypeId};
use core::fmt::{Debug, Formatter};
use core::ops::Add;
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, Point, Primitive, RgbColor, Size, WebColors};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use hashbrown::HashMap;
use log::{info, warn};

pub trait View {
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle);
    fn handle_input(&mut self, event: GuiEvent);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn bounds(&self) -> Rectangle;
}
impl Debug for Box<dyn View> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let val = self as &dyn Any;
        info!("is menu {}", TypeId::of::<MenuView>() == val.type_id());
        info!(
            "is box menuview {}",
            TypeId::of::<Box<MenuView>>() == val.type_id()
        );
        info!(
            "is box view {}",
            TypeId::of::<Box<dyn View>>() == val.type_id()
        );
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
        let font = FONT_9X15;
        let line_height = (font.character_size.height + 2) as i32;
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
            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
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
                Rgb565::BLACK
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
            let text_style = MonoTextStyle::new(&font, fg);
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
                info!("menu got {pt} {delta}");
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

pub struct Scene {
    pub views: Vec<Box<dyn View>>,
    pub focused: Option<i32>,
    pub keys: HashMap<String, i32>,
    dirty: bool,
    pub clip: Rectangle,
}

impl Scene {
    pub fn add(&mut self, name: &str, main_menu: Box<dyn View>) {
        let bounds = main_menu.bounds();
        self.views.push(main_menu);
        self.keys
            .insert(name.to_string(), (self.views.len() as i32) - 1);
        self.mark_dirty(bounds);
    }
}

impl Scene {
    pub fn is_menu_selected(&self, index: i32, hi: usize) -> bool {
        if let Some(menu) = self.views[index as usize]
            .as_any()
            .downcast_ref::<MenuView>()
        {
            return menu.highlighted_index == hi;
        }
        false
    }
    pub fn is_menu_selected_by_name(&self, name: &str, hi: usize) -> bool {
        if let Some(index) = self.keys.get(name) {
            return self.is_menu_selected(*index, hi);
        }
        false
    }
    pub fn menu_equals(&self, name: &str, value:&str) -> bool {
        if let Some(index) = self.keys.get(name) {
            if let Some(menu) = self.views[*index as usize]
                .as_any()
                .downcast_ref::<MenuView>()
            {
                let item = &menu.items[menu.highlighted_index as usize];
                if item == value {
                    return true;
                }
            }
        }
        false
    }
    pub fn hide_menu(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            if let Some(menu) = self.views[*index as usize]
                .as_any_mut()
                .downcast_mut::<MenuView>()
            {
                menu.visible = false;
                self.dirty = true;
                self.set_focused(0);
            }
        }
    }
    pub fn show_menu(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            if let Some(menu) = self.views[*index as usize]
                .as_any_mut()
                .downcast_mut::<MenuView>()
            {
                let bounds = menu.bounds();
                menu.visible = true;
                self.dirty = true;
                self.set_focused(*index);
                self.mark_dirty(bounds);
            }
        } else {
            warn!("Missing menu by name '{}'", name);
        }
    }
    pub fn get_menu_at(&self, index: i32) -> Option<&MenuView> {
        self.views[index as usize]
            .as_any()
            .downcast_ref::<MenuView>()
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
        self.views[index as usize]
            .as_any()
            .downcast_ref::<TextView>()
    }
    pub fn get_textview_at_mut(&mut self, index: i32) -> Option<&mut TextView> {
        self.views[index as usize]
            .as_any_mut()
            .downcast_mut::<TextView>()
    }
    pub fn get_textview_at_mut_by_name(&mut self, name: &str) -> Option<&mut TextView> {
        if let Some(index) = self.keys.get(name) {
            self.views[*index as usize]
                .as_any_mut()
                .downcast_mut::<TextView>()
        } else {
            None
        }
    }
    pub fn is_focused(&self, p0: i32) -> bool {
        if let Some(f) = self.focused {
            if f == p0 {
                return true;
            }
        }
        false
    }
    pub fn is_focused_by_name(&self, name: &str) -> bool {
        if let Some(f) = self.keys.get(name) {
            return self.is_focused(*f);
        }
        false
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn mark_dirty(&mut self, bounds: Rectangle) {
        self.dirty = true;
        self.clip = union(&self.clip, &bounds);
    }
    pub fn new() -> Scene {
        Scene {
            dirty: true,
            views: vec![],
            focused: None,
            keys: HashMap::new(),
            clip: Rectangle::zero(),
        }
    }
    pub fn handle_event(&mut self, evt: GuiEvent) {
        if let Some(index) = self.focused {
            if index < self.views.len() as i32 {
                let view = &mut self.views[index as usize];
                view.handle_input(evt);
                let bounds = view.bounds();
                self.mark_dirty(bounds);
            }
        }
    }
    pub fn set_focused(&mut self, index: i32) {
        let view = &mut self.views[index as usize];
        let bounds = view.bounds();
        self.mark_dirty(bounds);
        self.focused = Some(index);
    }
    pub fn set_focused_by_name(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            self.set_focused(*index);
        }
    }
}

impl Scene {
    pub fn draw(&mut self, display: &mut TDeckDisplay) {
        self.views
            .iter_mut()
            .for_each(|v| v.draw(display, &self.clip));
        self.dirty = false;
        self.clip = Rectangle::zero();
    }
}

fn union(a: &Rectangle, b: &Rectangle) -> Rectangle {
    if a.is_zero_sized() {
        return b.clone();
    }
    if b.is_zero_sized() {
        return a.clone();
    }
    let x = a.top_left.x.max(b.top_left.x);
    let y = a.top_left.y.max(b.top_left.y);
    let x2 = (a.top_left.x + a.size.width as i32).max(b.top_left.x + b.size.width as i32);
    let y2 = (a.top_left.y + a.size.height as i32).max(b.top_left.y + b.size.height as i32);
    Rectangle::with_corners(Point::new(x, y), Point::new(x2, y2))
}

#[derive(Debug, Copy, Clone)]
pub enum GuiEvent {
    KeyEvent(u8),
    PointerEvent(Point, Point),
}
