use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, View};
use alloc::string::{ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::max;
use embedded_graphics::geometry::{OriginDimensions, Point};
use embedded_graphics::mono_font::ascii::{FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, Primitive, RgbColor};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use log::{info, warn};
use nostd_html_parser::blocks::BlockType;
use nostd_html_parser::lines::TextLine;

pub struct TextView {
    pub dirty: bool,
    pub lines: Vec<TextLine>,
    pub visible: bool,
    pub scroll_index: i32,
    pub bounds: Rectangle,
}
impl View for TextView {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }
    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle) {
        if !self.visible {
            return;
        }
        self.dirty = false;
        let font = FONT_9X15;
        let line_height = font.character_size.height + 2;
        let viewport_height: i32 = (display.size().height / line_height) as i32;
        let char_width = font.character_size.width as i32;

        self.bounds
            .intersection(clip)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(display)
            .unwrap();

        // select the lines in the current viewport
        let mut end: usize = (self.scroll_index as i32 + viewport_height) as usize;
        if end >= self.lines.len() {
            end = self.lines.len();
        }
        let start = max(self.scroll_index, 0) as usize;
        let viewport_lines = &self.lines[start..end];

        let x_inset = 5;
        let y_inset = 5;

        // draw the lines
        for (j, line) in viewport_lines.iter().enumerate() {
            let mut inset_chars: usize = 0;
            let y = j as i32 * (line_height as i32) + 10;
            let style = match line.block_type {
                BlockType::Paragraph => MonoTextStyle::new(&FONT_9X15, Rgb565::BLACK),
                BlockType::ListItem => MonoTextStyle::new(&FONT_9X15, Rgb565::RED),
                BlockType::Header => MonoTextStyle::new(&FONT_9X15_BOLD, Rgb565::BLACK),
            };
            for (i, run) in line.runs.iter().enumerate() {
                let pos = Point::new(inset_chars as i32 * char_width + x_inset, y + y_inset);
                let text = Text::new(&run.text, pos, style);
                if !text.bounding_box().intersection(clip).is_zero_sized() {
                    text.draw(display).unwrap();
                }
                inset_chars += run.text.len();
            }
        }
    }
    fn handle_input(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::KeyEvent(key) => {
                match key {
                    b'j' => self.scroll_index = (self.scroll_index + 1) % (self.lines.len() as i32),
                    b'k' => self.scroll_index = max(self.scroll_index - 1, 0),
                    _ => {
                        warn!("Unhandled key {:?}", key);
                    }
                }
                info!("now scroll index {}", self.scroll_index);
                self.dirty = true
            }
            _ => {

            }
        }
    }
}
