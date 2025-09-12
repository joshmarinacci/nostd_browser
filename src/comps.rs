use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use gui2::geom::Bounds;
use gui2::{
    find_children, Action, DrawEvent, DrawingContext, GuiEvent, HAlign, Scene, Theme, View,
};
use hashbrown::HashMap;
use log::info;

pub fn make_overlay_label<C, F>(name: &str, title: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds::new(0, 0, 100, 20),
        visible: true,
        state: None,
        layout: None,
        input: None,
        draw: Some(|v, ctx, theme| {
            ctx.fill_rect(&v.bounds, &theme.fg);
            ctx.fill_text(&v.bounds, &v.title, &theme.bg, &HAlign::Right);
        }),
        draw2: None,
    }
}

pub fn make_rect_view<C, F>(name: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, 20, 20),
        visible: true,
        draw: None,
        draw2: Some(|e| info!("drawing rect")),
        layout: None,
        state: None,
        input: None,
    }
}

//                     30..126 => {
//                         info!("printable key: {:?}", key);
//                         self.text.push_str(&String::from_utf8_lossy(&[key]))
//                     13 => {
//                         info!("text input return key")
//                     8 => {
//                         info!("backspace");
//                         self.text.pop();
//                     0_u8..=29_u8 | 126_u8..=u8::MAX => {
//                         info!("unprintable key: {:?}", key);
