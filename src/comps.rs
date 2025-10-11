use log::info;
use iris_ui::geom::Bounds;
use iris_ui::view::{Align, View, ViewId};
use iris_ui::DrawEvent;
use iris_ui::gfx::TextStyle;

pub fn make_overlay_label(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        bounds: Bounds::new(0, 0, 100, 20),
        draw: Some(|e: &mut DrawEvent| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.text);
            let style = TextStyle::new(&e.theme.font, &e.theme.standard.fill).with_halign(Align::End);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        .. Default::default()
    }
}

pub fn make_rect_view(name: &'static str) -> View {
    View {
        name: ViewId::new(name),
        title: name.into(),
        bounds: Bounds::new(0, 0, 20, 20),
        visible: true,
        draw: Some(|e| {
            info!("bounds: {:?}", e.view.bounds);
            e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
        }),
        .. Default::default()
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
