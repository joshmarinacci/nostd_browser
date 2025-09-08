// pub struct MenuView {
//     pub items: Vec<String>,
//     pub position: Point,
//     pub highlighted_index: usize,
//     pub visible: bool,
//     pub dirty: bool,
// }
// 
// const PAD: u32 = 5;
// impl MenuView {
//     pub(crate) fn nav_prev(&mut self) {
//         self.highlighted_index = (self.highlighted_index + 1) % self.items.len();
//         self.dirty = true;
//     }
//     pub(crate) fn nav_next(&mut self) {
//         self.highlighted_index = (self.highlighted_index + self.items.len() - 1) % self.items.len();
//         self.dirty = true;
//     }
//     pub fn new(items: Vec<&str>, p1: Point) -> Box<MenuView> {
//         Box::new(MenuView {
//             items: items.iter().map(|s| s.to_string()).collect(),
//             position: p1,
//             highlighted_index: 0,
//             visible: true,
//             dirty: true,
//         })
//     }
//     pub fn start_hidden(items: Vec<&str>, p1: Point) -> Box<MenuView> {
//         Box::new(MenuView {
//             items: items.iter().map(|s| s.to_string()).collect(),
//             position: p1,
//             highlighted_index: 0,
//             visible: false,
//             dirty: true,
//         })
//     }
//     fn size(&self) -> Size {
//         let font = FONT_9X15;
//         let line_height = (font.character_size.height + 2) as u32;
//         let len = self.items.len() as u32;
//         Size::new(100 + PAD * 2, len * line_height + PAD * 2)
//     }
// }
// 
// impl View for MenuView {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// 
//     fn visible(&self) -> bool {
//         self.visible
//     }
// 
//     fn set_visible(&mut self, visible: bool) {
//         self.visible = visible;
//     }
// 
//     fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
//     }
// 
//     fn bounds(&self) -> Rectangle {
//         Rectangle {
//             top_left: self.position,
//             size: self.size().add(Size::new(10, 10)),
//         }
//     }
//     fn draw(&mut self, context: &mut DrawContext) {
//         if !self.visible {
//             return;
//         }
//         let line_height = (context.theme.font.character_size.height + 2) as i32;
//         let pad = PAD as i32;
// 
//         let xoff: i32 = 2;
//         let yoff: i32 = 0;
//         let menu_size = self.size();
//         // menu background
//         if context.theme.shadow {
//             let shadow_style = PrimitiveStyle::with_fill(Rgb565::CSS_LIGHT_GRAY);
//             context.display.rect(&Rectangle::new(self.position.add(Point::new(5, 5)), menu_size), shadow_style);
//         }
// 
//         let panel_style = PrimitiveStyleBuilder::new()
//             .stroke_width(1)
//             .stroke_alignment(Inside)
//             .stroke_color(context.theme.base_bd)
//             .fill_color(context.theme.base_bg)
//             .build();
//         context.display.rect(&Rectangle::new(self.position,menu_size),panel_style);
//         for (i, item) in self.items.iter().enumerate() {
//             let line_y = (i as i32) * line_height + pad;
// 
//             if i == self.highlighted_index {
//                 context.display.rect(&Rectangle::new(
//                     Point::new(pad, line_y + yoff).add(self.position),
//                     Size::new(100, line_height as u32),
//                 ),PrimitiveStyle::with_fill(context.theme.base_bd));
//             };
//             let fg = if i == self.highlighted_index {
//                 context.theme.base_bg
//             } else {
//                 context.theme.base_fg
//             };
//             let text_style = MonoTextStyle::new(&context.theme.font, fg);
//             let pos = Point::new(pad + xoff, line_y + line_height - 3 + yoff).add(self.position);
//             context.display.text(item,&pos,text_style);
//         }
//     }
//     fn handle_input(&mut self, event: GuiEvent) {
//         match event {
//             GuiEvent::KeyEvent(key) => match key {
//                 b'j' => self.nav_prev(),
//                 b'k' => self.nav_next(),
//                 _ => {}
//             },
//             GuiEvent::ScrollEvent(_, delta) => {
//                 if delta.y < 0 {
//                     self.nav_next();
//                 }
//                 if delta.y > 0 {
//                     self.nav_prev();
//                 }
//             }
//             GuiEvent::TouchEvent(pt) => {
//                 let pos = pt.sub(self.position);
//                 let line_height = (BASE_FONT.character_size.height + 2) as i32;
//                 let index:usize = (pos.y / line_height) as usize;
//                 if  index < self.items.len() {
//                     self.highlighted_index = index;
//                     self.dirty = true;
//                 }
//             }
//             _ => {
//                 warn!("unhandled event: {:?}", event);
//             }
//         }
//     }
// }
// 
// pub struct OverlayLabel {
//     text: String,
//     bounds: Rectangle,
//     visible: bool,
// }
// impl OverlayLabel {
//     pub fn new(text: &str, bounds: Rectangle) -> Box<OverlayLabel> {
//         Box::new(OverlayLabel {
//             text: text.to_string(),
//             bounds,
//             visible: true,
//         })
//     }
//     pub fn set_text(&mut self, text: &str) {
//         self.text = text.to_string();
//     }
// }
// impl View for OverlayLabel {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
// 
//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// 
//     fn visible(&self) -> bool {
//         self.visible
//     }
// 
//     fn set_visible(&mut self, visible: bool) {
//         self.visible = visible;
//     }
// 
//     fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
//     }
// 
//     fn bounds(&self) -> Rectangle {
//         self.bounds
//     }
// 
//     fn draw(&mut self, context: &mut DrawContext) {
//         let style = PrimitiveStyleBuilder::new()
//             .fill_color(context.theme.base_fg)
//             .build();
// 
//         context.display.rect(&self.bounds,style);
//         let style = MonoTextStyle::new(&context.theme.font, context.theme.base_bg);
//         context.display.text(&self.text,&self.bounds.center(),style);
//     }
// 
//     fn handle_input(&mut self, event: GuiEvent) {
//         info!("button got input: {:?}", event);
//     }
// }
// 
// 
// pub struct TextInput {
//     pub text: String,
//     pub bounds: Rectangle,
//     pub visible: bool,
// }
// 
// impl TextInput {
//     pub fn new(text: &str, bounds: Rectangle) -> Box<TextInput> {
//         Box::new(TextInput {
//             text:String::from(text),
//             bounds,
//             visible: true,
//         })
//     }
// }
// impl View for TextInput {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
// 
//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// 
//     fn visible(&self) -> bool {
//         self.visible
//     }
// 
//     fn set_visible(&mut self, visible: bool) {
//         self.visible = visible;
//     }
// 
//     fn layout(&mut self, _display: &mut dyn ViewTarget, _theme: &Theme) {
//     }
// 
//     fn bounds(&self) -> Rectangle {
//         self.bounds
//     }
// 
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