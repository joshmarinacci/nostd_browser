use alloc::vec;
use gui2::geom::Bounds;
use gui2::{HAlign, View};

pub fn make_overlay_label<C,F>(name:&str, title:&str) -> View<C,F> {
    View {
        name:name.into(),
        title: title.into(),
        bounds: Bounds::new(0,0,100,20),
        visible:true,
        state: None,
        layout: None,
        input: None,
        draw: Some(|v,ctx,theme|{
            ctx.fill_rect(&v.bounds,&theme.fg);
            ctx.fill_text(&v.bounds, &v.title, &theme.bg, &HAlign::Right);
        }),
        draw2: None,
    }
}




//     fn draw(&mut self, context: &mut DrawContext) {
//         let stroke_width = if context.is_focused { 3 } else { 1 };
//         let bounds_style = PrimitiveStyleBuilder::new()
//             .fill_color(Rgb565::WHITE)
//             .stroke_color(Rgb565::BLACK)
//             .stroke_width(stroke_width)
//             .stroke_alignment(StrokeAlignment::Inside)
//             .build();
//         context.display.rect(&self.bounds, bounds_style);
//
//         let text_style = MonoTextStyle::new(&context.theme.font, context.theme.base_fg);
//         context.display.text(&self.text,&self.bounds.top_left.add(Point::new(5,20)),text_style)
//     }
//
//     fn handle_input(&mut self, event: GuiEvent) {
//         match event {
//             GuiEvent::KeyEvent(key) => {
//                 match key {
//                     30..126 => {
//                         info!("printable key: {:?}", key);
//                         self.text.push_str(&String::from_utf8_lossy(&[key]))
//                     },
//                     13 => {
//                         info!("text input return key")
//                     }
//                     8 => {
//                         info!("backspace");
//                         self.text.pop();
//                     }
//                     0_u8..=29_u8 | 126_u8..=u8::MAX => {
//                         info!("unprintable key: {:?}", key);
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
// }