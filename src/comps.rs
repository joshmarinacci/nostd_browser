use log::info;
use rust_embedded_gui::geom::Bounds;
use rust_embedded_gui::gfx::{HAlign, TextStyle};
use rust_embedded_gui::view::View;
use rust_embedded_gui::DrawEvent;

pub fn make_overlay_label(name: &str, title: &str) -> View {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds::new(0, 0, 100, 20),
        visible: true,
        state: None,
        layout: None,
        input: None,
        draw: Some(|e: &mut DrawEvent| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.fg);
            let style = TextStyle::new(&e.theme.font, &e.theme.bg).with_halign(HAlign::Right);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
    }
}

pub fn make_rect_view(name: &str) -> View {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, 20, 20),
        visible: true,
        draw: Some(|e| {
            info!("bounds: {:?}", e.view.bounds);
            e.ctx.fill_rect(&e.view.bounds, &e.theme.fg);
        }),
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
