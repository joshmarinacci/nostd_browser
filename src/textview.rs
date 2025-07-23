use alloc::string::{String, ToString};
use alloc::{format, vec};
use alloc::vec::Vec;
use core::any::Any;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{OriginDimensions, Point};
use embedded_graphics::mono_font::ascii::FONT_9X15;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::text::Text;
use log::info;
use crate::common::TDeckDisplay;
use crate::gui::{GuiEvent, View};
use crate::textview::LineStyle::{Header, Plain};

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
    fn header(p0: &str) -> TextRun {
        TextRun {
            style: Header,
            text: p0.to_string(),
        }
    }
    fn plain(p0: &str) -> TextRun {
        TextRun {
            style: Plain,
            text: p0.to_string(),
        }
    }
}

pub struct TextLine {
    pub runs: Vec<TextRun>,
}

impl TextLine {
    pub fn with(runs: Vec<TextRun>) -> TextLine {
        TextLine {
            runs: Vec::from(runs),
        }
    }
    pub fn new(p0: &str) -> TextLine {
        TextLine {
            runs: Vec::from([
                TextRun::plain(p0),
            ])
        }
    }
}

pub fn break_lines(text: &str, width: u32, style: LineStyle) -> Vec<TextLine> {
    let mut lines: Vec<TextLine> = vec![];
    let mut tl:TextLine = TextLine {
        runs: vec![],
    };
    let mut bucket = String::new();
    for (i,word) in text.split(' ').enumerate() {
        let word = word.trim();
        // info!("word = {:?}", word);
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
    pub scroll_index: usize,
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
        let debug =  MonoTextStyle::new(&FONT_9X15, Rgb565::CSS_ORANGE);
        let x_inset = 5;

        // info!("drawing lines at scroll {}", scroll_offset);
        // select the lines in the current viewport
        // draw the lines
        let mut end:usize = (self.scroll_index as i32 + viewport_height) as usize;
        if end >= self.lines.len() {
            end = self.lines.len();
        }
        let viewport_lines = &self.lines[(self.scroll_index) .. end];
        for (j, line) in viewport_lines.iter().enumerate() {
            let mut inset_chars: usize = 3;
            let y = j as i32 * line_height + 10;
            Text::new(&format!("{}", j), Point::new(x_inset, y), debug).draw(display).unwrap();
            for (i, run) in line.runs.iter().enumerate() {
                let pos = Point::new(inset_chars as i32 * char_width, y);
                let style = MonoTextStyle::new(&FONT_9X15, Rgb565::CSS_RED);
                // let style = match run.style {
                //     Plain => theme.plain,
                //     Header => theme.header,
                //     Link => theme.link,
                // };
                Text::new(&run.text, pos, style).draw(display).unwrap();
                inset_chars += run.text.len();
            }
        }

    }
    fn handle_input(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::KeyEvent(key) => {
                match key {
                    b'j' => self.scroll_index = (self.scroll_index + 1) % self.lines.len(),
                    b'k' => self.scroll_index = (self.scroll_index - 1) % self.lines.len(),
                    _ => {}
                }
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