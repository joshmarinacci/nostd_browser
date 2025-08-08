use crate::common::TDeckDisplay;
use comps::MenuView;
use crate::pageview::PageView;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::fmt::{Debug, Formatter};
use embedded_graphics::mono_font::ascii::{FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, Point, RgbColor, Size};
use embedded_graphics::primitives::Rectangle;
use hashbrown::HashMap;
use log::{info, warn};

pub mod comps;

pub const BASE_FONT: MonoFont = FONT_9X15;
pub const BOLD_FONT: MonoFont = FONT_9X15_BOLD;
pub struct Theme {
    pub base_bg: Rgb565,
    pub base_bd: Rgb565,
    pub base_fg: Rgb565,
    pub shadow: bool,
    pub font: MonoFont<'static>,
    pub bold: MonoFont<'static>,
}
pub const LIGHT_THEME: Theme = Theme {
    base_bg: Rgb565::WHITE,
    base_bd: Rgb565::BLACK,
    base_fg: Rgb565::BLACK,
    shadow: false,
    font: BASE_FONT,
    bold: BOLD_FONT,
};
pub const DARK_THEME: Theme = Theme {
    base_bg: Rgb565::BLACK,
    base_bd: Rgb565::WHITE,
    base_fg: Rgb565::WHITE,
    font: BASE_FONT,
    bold: BOLD_FONT,
    shadow: false,
};

pub trait View {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn layout(&mut self, display: &mut TDeckDisplay, theme: &Theme);
    fn bounds(&self) -> Rectangle;
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle, theme: &Theme);
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
            Some(_menu) => {
                write!(f, "is a menu view")
            }
            None => {
                write!(f, "some other object")
            }
        }
    }
}
pub struct Scene {
    draw_order: Vec<String>,
    focused: Option<String>,
    keys: HashMap<String, Box<dyn View>>,
    dirty: bool,
    layout_dirty: bool,
    clip: Rectangle,
    theme: Theme,
    auto_redraw: bool,
}

impl Scene {
    pub fn set_font(&mut self, font: MonoFont<'static>) {
        self.theme.font = font;
        self.mark_dirty(Rectangle::new(Point::new(0, 0), Size::new(320, 240)))
        self.mark_layout_dirty();
    }
}

impl Scene {
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.mark_dirty(Rectangle::new(Point::new(0, 0), Size::new(320, 240)))
    }
    pub fn set_auto_redraw(&mut self, auto_redraw: bool) {
        self.auto_redraw = auto_redraw;
        self.mark_dirty(Rectangle::new(Point::new(0, 0), Size::new(320, 240)))
    }
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
            layout_dirty: true,
            draw_order: Vec::new(),
            focused: None,
            keys: HashMap::new(),
            clip: Rectangle::zero(),
            theme: LIGHT_THEME,
            auto_redraw: false,
        }
    }
    pub fn add(&mut self, name: &str, view: Box<dyn View>) {
        let bounds = view.bounds();
        self.keys.insert(name.to_string(), view);
        self.draw_order.push(name.to_string());
        self.mark_dirty(bounds);
        self.mark_layout_dirty()
    }
    pub fn remove(&mut self, name: &str) {
        if self.keys.contains_key(name) {
            self.keys.remove(name);
            if let Some(index) = self.draw_order.iter().position(|x| x == name) {
                self.draw_order.remove(index);
            }
            info!("deleting the view {name}");
            self.mark_layout_dirty()
        } else {
            warn!("remove: no view found for the name: {name}");
        }
    }
}

impl Scene {
    pub fn get_view(&self, name: &str) -> Option<&Box<dyn View>> {
        if let Some(view) = self.keys.get(name) {
            Some(view)
        } else {
            None
        }
    }
    pub fn get_view_mut(&mut self, name: &str) -> Option<&mut Box<dyn View>> {
        if let Some(view) = self.keys.get_mut(name) {
            Some(view)
        } else {
            None
        }
    }
    pub fn mutate_view<F: Fn(&mut Box<dyn View>)>(&mut self, name: &str, callback: F) {
        if let Some(view) = self.keys.get_mut(name) {
            callback(view);
            let bounds = view.bounds();
            self.mark_dirty(bounds);
        } else {
            warn!("mutate_view: Missing view with name '{}'", name);
        }
    }

    pub fn get_menu(&self, name: &str) -> Option<&MenuView> {
        if let Some(view) = self.keys.get(name) {
            if let Some(menu) = view.as_any().downcast_ref::<MenuView>() {
                Some(menu)
            } else {
                None
            }
        } else {
            warn!("get_menu: Missing menu by name '{}'", name);
            None
        }
    }
    pub fn get_menu_mut(&mut self, name: &str) -> Option<&mut MenuView> {
        if let Some(view) = self.keys.get_mut(name) {
            if let Some(menu) = view.as_any_mut().downcast_mut::<MenuView>() {
                Some(menu)
            } else {
                None
            }
        } else {
            warn!("get_menu: Missing menu by name '{}'", name);
            None
        }
    }

    pub fn menu_equals(&self, name: &str, value: &str) -> bool {
        if let Some(menu) = self.get_menu(name) {
            let item = &menu.items[menu.highlighted_index];
            return if item == value { true } else { false };
        }
        warn!("menu_equals: no view found for the name: {name}");
        false
    }
    pub fn hide(&mut self, name: &str) {
        if let Some(menu) = self.get_view_mut(name) {
            menu.set_visible(false);
            let bounds = menu.bounds();
            self.mark_dirty(bounds);
        } else {
            warn!("hide_menu: no view found for the name: {name}");
        }
    }
    pub fn show_menu(&mut self, name: &str) {
        if let Some(menu) = self.get_view_mut(name) {
            let bounds = menu.bounds();
            menu.set_visible(true);
            self.set_focused(name);
            self.mark_dirty(bounds);
        } else {
            warn!("show_menu: no menu found for the name: {name}");
        }
    }
    pub fn get_textview_mut(&mut self, name: &str) -> Option<&mut PageView> {
        if let Some(view) = self.keys.get_mut(name) {
            if let Some(menu) = view.as_any_mut().downcast_mut::<PageView>() {
                Some(menu)
            } else {
                None
            }
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
    pub fn mark_dirty(&mut self, bounds: Rectangle) {
        self.dirty = true;
        self.clip = union(&self.clip, &bounds);
    }
    fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true
    }
    pub fn handle_input(&mut self, evt: GuiEvent) {
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
        } else {
            warn!("Missing view by name '{}'", name);
        }
    }
    pub fn get_focused_view(&self) -> Option<&Box<dyn View>> {
        if let Some(name) = &self.focused {
            self.get_view(name)
        } else {
            None
        }
    }
    pub fn get_focused_view_as_mut(&mut self) -> Option<&mut Box<dyn View>> {
        if let Some(name) = &self.focused {
            return self.get_view_mut(&name.clone());
        }
        None
    }
}

impl Scene {
    pub fn draw(&mut self, display: &mut TDeckDisplay) {
        if self.layout_dirty {
            self.do_layout(display);
        }
        if !self.dirty {
            return;
        }
        for name in &self.draw_order {
            if let Some(view) = self.keys.get_mut(name) {
                view.draw(display, &self.clip, &self.theme);
            }
        }
        if self.auto_redraw {
            self.dirty = true;
            self.clip = Rectangle::new(
                Point::new(0, 0),
                Size::new(
                    display.bounding_box().size.width,
                    display.bounding_box().size.height,
                ),
            );
        } else {
            self.dirty = false;
            self.clip = Rectangle::zero();
        }
    }

    fn do_layout(&mut self, display: &mut TDeckDisplay) {
        self.layout_dirty = false;
        for name in &self.draw_order {
            if let Some(view) = self.keys.get_mut(name) {
                view.layout(display, &self.theme);
            }
        }
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
    ScrollEvent(Point, Point),
    ClickEvent(),
}
