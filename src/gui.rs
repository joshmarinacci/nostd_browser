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
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, Point, Primitive, RgbColor, Size, WebColors};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use hashbrown::HashMap;
use log::{info, warn};
use crate::comps::MenuView;

pub(crate) const base_background_color: Rgb565 = Rgb565::CSS_LIGHT_GRAY;
pub(crate) const base_font:MonoFont = FONT_9X15;
pub(crate) const base_text_color: Rgb565 = Rgb565::BLACK;
pub(crate) const base_button_background_color: Rgb565 = Rgb565::GREEN;

pub trait View {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn bounds(&self) -> Rectangle;
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle);
    fn handle_input(&mut self, event: GuiEvent);
}
impl Debug for Box<dyn View> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let val = self as &dyn Any;
        // info!("is menu {}", TypeId::of::<MenuView>() == val.type_id());
        // info!(
        //     "is box menuview {}",
        //     TypeId::of::<Box<MenuView>>() == val.type_id()
        // );
        // info!(
        //     "is box view {}",
        //     TypeId::of::<Box<dyn View>>() == val.type_id()
        // );
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
pub struct Scene {
    pub views: Vec<Box<dyn View>>,
    pub focused: Option<String>,
    pub keys: HashMap<String, i32>,
    dirty: bool,
    pub clip: Rectangle,
}

impl Scene {
    pub fn info(&self) {
        info!("Scene info:");
        info!("focused: {:?}", self.focused);
        info!("dirty: {:?}", self.dirty);
        info!("clip: {:?}", self.clip);
        info!("keys: {:?}", self.keys);
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            dirty: true,
            views: vec![],
            focused: None,
            keys: HashMap::new(),
            clip: Rectangle::zero(),
        }
    }
    pub fn add(&mut self, name: &str, main_menu: Box<dyn View>) {
        let bounds = main_menu.bounds();
        self.views.push(main_menu);
        self.keys
            .insert(name.to_string(), (self.views.len() as i32) - 1);
        self.mark_dirty(bounds);
    }
    pub fn remove(&mut self, name: &str) {
        if let Some(index) = self.keys.get(name) {
            let view = self.views[*index as usize].as_mut();
            info!("pretending to delete the view {name}");
        } else {
            warn!("no view found for the name: {name}");
        }
    }
}

impl Scene {
    pub fn menu_equals(&self, name: &str, value:&str) -> bool {
        if let Some(index) = self.keys.get(name) {
            if let Some(menu) = self.views[*index as usize]
                .as_any()
                .downcast_ref::<MenuView>()
            {
                let item = &menu.items[menu.highlighted_index as usize];
                if item == value {
                    return true;
                } else {
                    return false;
                }
            }
        }
        warn!("menu_equals: no view found for the name: {name}");
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
                // self.set_focused(0);
                return
            }
        }
        warn!("hide_menu: no view found for the name: {name}");
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
                self.set_focused(name);
                // self.set_focused(*index);
                self.mark_dirty(bounds);
                return
            }
        }
        warn!("show_menu: no menu found for the name: {name}");
    }
    pub fn get_menu(&self, name: &str) -> Option<&MenuView> {
        if let Some(index) = self.keys.get(name) {
            self.views[*index as usize]
                .as_any()
                .downcast_ref::<MenuView>()
        } else {
            warn!("get_menu: Missing menu by name '{}'", name);
            None
        }
    }
    pub fn get_textview_mut(&mut self, name: &str) -> Option<&mut TextView> {
        if let Some(index) = self.keys.get(name) {
            self.views[*index as usize]
                .as_any_mut()
                .downcast_mut::<TextView>()
        } else {
            warn!("Missing textview by name '{}'", name);
            None
        }
    }
    pub fn is_focused(&self, name: &str) -> bool {
        if let Some(f2) = &self.focused {
            if f2.eq_ignore_ascii_case(name) {
                return true;
            }
        }
        warn!("is_focused: Missing view by name '{}'", name);
        false
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn mark_dirty(&mut self, bounds: Rectangle) {
        self.dirty = true;
        self.clip = union(&self.clip, &bounds);
    }
    pub fn handle_event(&mut self, evt: GuiEvent) {
        // info!("=== handle_event: {evt:?}");
        // info!("focused is {:?}", self.focused);
        if let Some(view) = self.get_focused_view_as_mut() {
            view.handle_input(evt);
            let bounds = view.bounds();
            self.mark_dirty(bounds);
        } else {
            warn!("no focused view found for event '{evt:?}'");
        }
    }
    pub fn set_focused(&mut self, name: &str) {
        if let Some(view) = self.get_view(name) {
            let bounds = view.bounds();
            self.mark_dirty(bounds);
            self.focused = Some(name.to_string());
        }
        warn!("Missing view by name '{}'", name);
    }

    pub fn get_view(&self, name: &str) -> Option<&Box<dyn View>> {
        if let Some(index) = self.keys.get(name) {
            Some(&self.views[*index as usize])
        } else {
            None
        }
    }
    pub fn get_focused_view(&self) -> Option<&Box<dyn View>> {
        if let Some(name) = &self.focused {
            return if let Some(index) = self.keys.get(name) {
                Some(&self.views[*index as usize])
            } else {
                None
            }
        }
        None
    }
    pub fn get_focused_view_as_mut(&mut self) -> Option<&mut (dyn View)> {
        if let Some(name) = &self.focused {
            return if let Some(index) = self.keys.get(name) {
                Some(self.views[*index as usize].as_mut())
            } else {
                None
            }
        }
        None
    }
}

impl Scene {
    pub fn draw(&mut self, display: &mut TDeckDisplay) {
        if !self.is_dirty() {
            return;
        }
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
    let x = a.top_left.x.min(b.top_left.x);
    let y = a.top_left.y.min(b.top_left.y);
    let x2 = (a.top_left.x + a.size.width as i32).max(b.top_left.x + b.size.width as i32);
    let y2 = (a.top_left.y + a.size.height as i32).max(b.top_left.y + b.size.height as i32);
    Rectangle::with_corners(Point::new(x, y), Point::new(x2, y2))
}

#[derive(Debug, Copy, Clone)]
pub enum GuiEvent {
    KeyEvent(u8),
    PointerEvent(Point, Point),
}

