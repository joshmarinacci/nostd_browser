#![no_std]
extern crate alloc;
use core::fmt::{Debug, Formatter};

// pub struct DrawContext<'a> {
//     pub display: &'a mut dyn ViewTarget,
//     clip: &'a Rectangle,
//     pub theme: &'a Theme,
//     is_focused: bool
// }
//

// pub struct Scene {
//     draw_order: Vec<String>,
//     focused: Option<String>,
//     keys: HashMap<String, Box<dyn View>>,
//     dirty: bool,
//     layout_dirty: bool,
//     clip: Rectangle,
//     theme: Theme,
//     auto_redraw: bool,
// }
//
// impl Scene {
//     pub fn set_font(&mut self, font: MonoFont<'static>, bold: MonoFont<'static>) {
//         self.theme.font = font;
//         self.theme.bold = bold;
//         self.mark_dirty(Rectangle::new(Point::new(0, 0), Size::new(320, 240)));
//         self.mark_layout_dirty();
//     }
// }
//
// impl Scene {
//     pub fn set_theme(&mut self, theme: Theme) {
//         self.theme = theme;
//         self.mark_dirty(Rectangle::new(Point::new(0, 0), Size::new(320, 240)))
//     }
//     pub fn set_auto_redraw(&mut self, auto_redraw: bool) {
//         self.auto_redraw = auto_redraw;
//         self.mark_dirty(Rectangle::new(Point::new(0, 0), Size::new(320, 240)))
//     }
// }
//
// impl Scene {
//     pub fn info(&self) {
//         info!("Scene info:");
//         info!("focused: {:?}", self.focused);
//         info!("dirty: {:?}", self.dirty);
//         info!("clip: {:?}", self.clip);
//         info!("keys: {:?}", self.keys);
//     }
// }
//
// impl Scene {
//     pub fn add(&mut self, name: &str, view: Box<dyn View>) {
//         let bounds = view.bounds();
//         self.keys.insert(name.to_string(), view);
//         self.draw_order.push(name.to_string());
//         self.mark_dirty(bounds);
//         self.mark_layout_dirty()
//     }
//     pub fn remove(&mut self, name: &str) {
//         if self.keys.contains_key(name) {
//             self.keys.remove(name);
//             if let Some(index) = self.draw_order.iter().position(|x| x == name) {
//                 self.draw_order.remove(index);
//             }
//             info!("deleting the view {name}");
//             self.mark_layout_dirty()
//         } else {
//             warn!("remove: no view found for the name: {name}");
//         }
//     }
// }
//
// impl Scene {
//     pub fn mutate_view<F: Fn(&mut Box<dyn View>)>(&mut self, name: &str, callback: F) {
//         if let Some(view) = self.keys.get_mut(name) {
//             callback(view);
//             let bounds = view.bounds();
//             self.mark_dirty(bounds);
//         } else {
//             warn!("mutate_view: Missing view with name '{}'", name);
//         }
//     }
//     fn mark_layout_dirty(&mut self) {
//         self.layout_dirty = true
//     }
//     pub fn handle_input(&mut self, evt: GuiEvent) {
//         // info!("=== handle_event: {evt:?}");
//         // info!("focused is {:?}", self.focused);
//         if let Some(view) = self.get_focused_view_as_mut() {
//             view.handle_input(evt);
//             let bounds = view.bounds();
//             self.mark_dirty(bounds);
//         } else {
//             warn!("no focused view found for event '{evt:?}'");
//         }
//     }
// }
//
// impl Scene {
//     pub fn draw(&mut self, display: &mut dyn ViewTarget) {
//         if self.layout_dirty {
//             self.do_layout(display);
//         }
//         if !self.dirty {
//             return;
//         }
//         let mut context:DrawContext = DrawContext {
//             display: display,
//             clip: &self.clip,
//             theme: &self.theme,
//             is_focused: false
//         };
//         for name in &self.draw_order {
//             context.is_focused = self.is_focused(name);
//             if let Some(view) = self.keys.get_mut(name) {
//                 view.draw(&mut context);
//             }
//         }
//         if self.auto_redraw {
//             self.dirty = true;
//             self.clip = Rectangle::new(
//                 Point::new(0, 0),
//                 display.size(),
//             );
//         } else {
//             self.dirty = false;
//             self.clip = Rectangle::zero();
//         }
//     }
//
//     fn do_layout(&mut self, display: &mut dyn ViewTarget) {
//         self.layout_dirty = false;
//         for name in &self.draw_order {
//             if let Some(view) = self.keys.get_mut(name) {
//                 view.layout(display, &self.theme);
//             }
//         }
//     }
// }
//
// fn union(a: &Rectangle, b: &Rectangle) -> Rectangle {
//     if a.is_zero_sized() {
//         return b.clone();
//     }
//     if b.is_zero_sized() {
//         return a.clone();
//     }
//     let x = a.top_left.x.min(b.top_left.x);
//     let y = a.top_left.y.min(b.top_left.y);
//     let x2 = (a.top_left.x + a.size.width as i32).max(b.top_left.x + b.size.width as i32);
//     let y2 = (a.top_left.y + a.size.height as i32).max(b.top_left.y + b.size.height as i32);
//     Rectangle::with_corners(Point::new(x, y), Point::new(x2, y2))
// }
