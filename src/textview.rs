use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, View};
use crate::textview::LineStyle::{Header, Plain};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use core::any::Any;
use core::cmp::max;
use embedded_graphics::geometry::{OriginDimensions, Point};
use embedded_graphics::mono_font::ascii::{FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use log::{info, warn};
use nostd_html_parser::blocks::{Block, BlockType};

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum LineStyle {
    Header,
    Plain,
    Link,
}

pub struct TextRun {
    pub style: LineStyle,
    pub text: String,
}

impl TextRun {
    fn plain(p0: &str) -> TextRun {
        TextRun {
            style: Plain,
            text: p0.to_string(),
        }
    }
}

pub struct TextLine {
    pub block_type: BlockType,
    pub runs: Vec<TextRun>,
}

impl TextLine {
    pub fn with(runs: Vec<TextRun>) -> TextLine {
        TextLine {
            block_type: BlockType::Plain,
            runs: Vec::from(runs),
        }
    }
    pub fn new(p0: &str) -> TextLine {
        TextLine {
            block_type: BlockType::Plain,
            runs: Vec::from([
                TextRun::plain(p0),
            ])
        }
    }
}

pub fn break_lines(block:&Block, width: u32) -> Vec<TextLine> {
    let style = match block.block_type {
        BlockType::Plain => Plain,
        BlockType::Header => Header,
        BlockType::ListItem => Plain,
    };
    let text = &block.text;

    let mut lines: Vec<TextLine> = vec![];
    let mut tl:TextLine = TextLine {
        block_type: block.block_type.clone(),
        runs: vec![],
    };
    let mut bucket = String::new();
    for (i,word) in text.split(' ').enumerate() {
        let word = word.trim();
        if word == "" {
            continue;
        }
        if bucket.len() + word.len() < width as usize {
            bucket.push_str(word);
            bucket.push_str(" ");
        } else {
            tl.runs.push(TextRun{
                style: style.clone(),
                text: bucket.clone(),
            });
            lines.push(tl);
            tl = TextLine {
                block_type: block.block_type.clone(),
                runs: vec![],
            };
            bucket.clear();
            bucket.push_str(word);
            bucket.push_str(" ");
        }
    }
    tl.runs.push(TextRun{
        style:style.clone(),
        text: bucket.clone(),
    });
    lines.push(tl);
    return lines;
}


pub struct TextView {
    pub dirty: bool,
    pub lines: Vec<TextLine>,
    pub visible: bool,
    pub scroll_index: i32,
}
impl View for TextView {
    fn draw(&mut self, display: &mut TDeckDisplay) {
        if !self.visible {
            return;
        }
        self.dirty = false;
        let font = FONT_9X15;
        let viewport_height:i32 = (display.size().height / font.character_size.height) as i32;
        let line_height = font.character_size.height as i32;
        let char_width = font.character_size.width as i32;

        // select the lines in the current viewport
        let mut end:usize = (self.scroll_index as i32 + viewport_height) as usize;
        if end >= self.lines.len() {
            end = self.lines.len();
        }
        let start = max(self.scroll_index,0) as usize;
        let viewport_lines = &self.lines[start .. end];

        // draw the lines
        for (j, line) in viewport_lines.iter().enumerate() {
            let mut inset_chars: usize = 0;
            let y = j as i32 * line_height + 10;
            let style = match line.block_type {
                BlockType::Plain => MonoTextStyle::new(&FONT_9X15, Rgb565::BLACK),
                BlockType::ListItem => MonoTextStyle::new(&FONT_9X15, Rgb565::GREEN),
                BlockType::Header => MonoTextStyle::new(&FONT_9X15_BOLD, Rgb565::BLACK),
            };
            for (i, run) in line.runs.iter().enumerate() {
                let pos = Point::new(inset_chars as i32 * char_width, y);
                Text::new(&run.text, pos, style).draw(display).unwrap();
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
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

}