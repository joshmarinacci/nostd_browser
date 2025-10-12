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