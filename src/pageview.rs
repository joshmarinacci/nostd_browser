use crate::common::{NetCommand, TDeckDisplay, NET_COMMANDS};
use crate::gui::{GuiEvent, Theme, View};
use crate::page::Page;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::any::Any;
use core::cmp::max;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_6X13, FONT_8X13, FONT_8X13_BOLD, FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, Primitive, RgbColor};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::iso_8859_16::FONT_6X13_BOLD;
use log::{info, warn};
use nostd_html_parser::blocks::BlockType;
use nostd_html_parser::lines::{break_lines, RunStyle, TextLine};

pub struct PageView {
    pub dirty: bool,
    pub lines: Vec<TextLine>,
    pub link_count: i32,
    pub visible: bool,
    pub scroll_index: i32,
    pub bounds: Rectangle,
    pub page: Page,
}

impl PageView {
    pub fn new(bounds: Rectangle, page: Page) -> PageView {
        PageView {
            dirty: true,
            visible: true,
            lines: vec![],
            scroll_index: 0,
            link_count: 0,
            page,
            bounds,
        }
    }
    pub fn load_page(&mut self, page: Page, columns: u32) {
        let mut lines: Vec<TextLine> = vec![];
        let mut link_count = 0;
        for block in &page.blocks {
            let mut some_lines = break_lines(&block, columns);
            for line in &some_lines {
                for run in &line.runs {
                    // info!("Run: {:?}", run);
                    match &run.style {
                        RunStyle::Link(href) => {
                            info!("found a link: {:?}", href);
                            link_count += 1;
                        }
                        _ => {}
                    }
                }
            }
            lines.append(&mut some_lines);
        }
        self.link_count = link_count;
        self.lines = lines;
        self.page = page;
    }

    pub fn prev_link(&mut self) {
        self.page.selection -= 1;
        if self.page.selection < 0 {
            self.page.selection = self.link_count - 1;
        }
    }
    pub fn next_link(&mut self) {
        self.page.selection += 1;
        if self.page.selection >= self.link_count {
            self.page.selection = 0;
        }
        info!(
            "selected link: {:?}",
            self.find_href_by_index(self.page.selection)
        );
    }
    pub fn find_href_by_index(&self, index: i32) -> Option<&str> {
        // info!("find_href_by_index: {}", index);
        // // blocks of spans -> text lines of text runs
        let mut count = 0;
        for line in &self.lines {
            for run in &line.runs {
                match &run.style {
                    RunStyle::Link(href) => {
                        // info!("link is {}", href);
                        if count == index {
                            // info!("found the right link");
                            return Some(&href);
                        }
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
        None
    }
    pub(crate) fn nav_current_link(&self) {
        if let Some(href) = self.find_href_by_index(self.page.selection) {
            info!("loading the href {}", href);
            let mut href = href.to_string();
            if !href.starts_with("http") {
                info!("doing a relative link. base is {}", self.page.url);
                href = format!("{}{}", self.page.url, href);
                info!("final url is {}", href);
            }
            NET_COMMANDS
                .try_send(NetCommand::Load(href.to_string()))
                .unwrap()
        }
    }
}
impl View for PageView {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn visible(&self) -> bool {
        self.visible
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    fn layout(&mut self, display: &mut TDeckDisplay, theme: &Theme) {
        self.bounds = Rectangle::new(Point::new(0, 0),  Size::new(display.size().width, display.size().height));
    }

    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }

    fn draw(&mut self, display: &mut TDeckDisplay, clip: &Rectangle, theme: &Theme) {
        if !self.visible {
            return;
        }
        self.dirty = false;
        let font = theme.font;
        let line_height = font.character_size.height + 2;
        let viewport_height: i32 = (display.size().height / line_height) as i32;
        let char_width = font.character_size.width as i32;

        self.bounds
            .intersection(clip)
            .into_styled(PrimitiveStyle::with_fill(theme.base_bg))
            .draw(display)
            .unwrap();

        // select the lines in the current viewport
        let mut end: usize = (self.scroll_index as i32 + viewport_height) as usize;
        if end >= self.lines.len() {
            end = self.lines.len();
        }
        let start = max(self.scroll_index, 0) as usize;
        let viewport_lines = &self.lines[start..end];

        let x_inset = 8;
        let y_inset = 5;

        let mut link_count = -1;
        // draw the lines
        for (j, line) in viewport_lines.iter().enumerate() {
            let mut inset_chars: usize = 0;
            let y = j as i32 * (line_height as i32) + 10;
            let style = match line.block_type {
                BlockType::Paragraph => MonoTextStyle::new(&theme.font, theme.base_fg),
                BlockType::ListItem => MonoTextStyle::new(&theme.font, theme.base_fg),
                BlockType::Header => MonoTextStyle::new(calc_bold(theme.font), theme.base_fg),
            };
            // draw a bullet
            if line.block_type == BlockType::ListItem {
                Rectangle::new(Point::new(2, y), Size::new(4, 3))
                    .into_styled(PrimitiveStyle::with_fill(theme.base_fg))
                    .draw(display)
                    .unwrap();
            }
            for run in &line.runs {
                let pos = Point::new(inset_chars as i32 * char_width + x_inset, y + y_inset);
                let text_style = match &run.style {
                    RunStyle::Link(_) => {
                        // info!("found a link: {:?}", href);
                        link_count += 1;
                        let builder = MonoTextStyleBuilder::new()
                            .font(&theme.font)
                            .underline();
                        if self.page.selection == link_count {
                            builder
                                .text_color(Rgb565::WHITE)
                                .background_color(Rgb565::BLUE)
                                .build()
                        } else {
                            builder
                                .text_color(Rgb565::BLUE)
                                .background_color(Rgb565::WHITE)
                                .build()
                        }
                    }
                    RunStyle::Plain => style,
                    RunStyle::Bold => style,
                };
                let text = Text::new(&run.text, pos, text_style);
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
                    b'j' => {
                        self.scroll_index = (self.scroll_index + 10) % (self.lines.len() as i32)
                    }
                    b'k' => self.scroll_index = max(self.scroll_index - 10, 0),
                    b'a' => self.prev_link(),
                    b's' => self.next_link(),
                    13 => self.nav_current_link(),
                    _ => {
                        warn!("Unhandled key {:?}", key);
                    }
                }
                info!("now scroll index {}", self.scroll_index);
                self.dirty = true
            }
            GuiEvent::ScrollEvent(pt, delta) => {
                if (delta.x < 0) || (delta.y < 0) {
                    self.prev_link();
                };
                if (delta.x > 0) || (delta.y > 0) {
                    self.next_link();
                };
            }
            _ => {}
        }
    }
}

fn calc_bold(font: MonoFont<'static>) -> &'static MonoFont<'static> {
    if font == FONT_6X13 {
        return &FONT_6X13_BOLD
    }
    if font == FONT_8X13 {
        return &FONT_8X13_BOLD
    }
    if font == FONT_9X15 {
        return &FONT_9X15_BOLD
    }
    return &FONT_9X15_BOLD
}
