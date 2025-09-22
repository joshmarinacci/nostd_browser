// use crate::common::{NetCommand, NET_COMMANDS};
use crate::page::Page;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::cmp::max;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_9X15_BOLD;
use log::{info, warn};
use nostd_html_parser::blocks::BlockType;
use nostd_html_parser::lines::{break_lines, RunStyle, TextLine};
use rust_embedded_gui::geom::Bounds;
use rust_embedded_gui::gfx::{HAlign, TextStyle};
use rust_embedded_gui::view::View;
use rust_embedded_gui::{Action, DrawEvent, EventType, GuiEvent};

pub struct RenderedPage {
    pub link_count: i32,
    pub lines: Vec<TextLine>,
    pub page: Page,
    pub scroll_index: i32,
}
impl RenderedPage {
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
}
pub struct PageView {
    pub dirty: bool,
    pub history: Vec<RenderedPage>,
    pub history_index: usize,
    pub visible: bool,
    pub bounds: Bounds,
    pub columns: u32,
}

impl PageView {
    pub fn new(bounds: Bounds, page: Page) -> View {
        let pv = PageView {
            dirty: true,
            visible: true,
            columns: 20,
            history: vec![RenderedPage {
                lines: vec![],
                scroll_index: 0,
                page,
                link_count: 0,
            }],
            history_index: 0,
            bounds,
        };
        View {
            name: "page".into(),
            title: "page".into(),
            bounds,
            visible: true,
            state: Some(Box::new(pv)),
            input: Some(handle_input),
            layout: None,
            draw: Some(draw),
        }
    }
    pub fn load_page(&mut self, page: Page) {
        let mut lines: Vec<TextLine> = vec![];
        let mut link_count = 0;
        for block in &page.blocks {
            let mut some_lines = break_lines(&block, self.columns);
            for line in &some_lines {
                for run in &line.runs {
                    // info!("Run: {:?}", run);
                    match &run.style {
                        RunStyle::Link(href) => {
                            // info!("found a link: {:?}", href);
                            link_count += 1;
                        }
                        _ => {}
                    }
                }
            }
            lines.append(&mut some_lines);
        }
        let pg: RenderedPage = RenderedPage {
            link_count,
            lines,
            page,
            scroll_index: 0,
        };
        self.history.push(pg);
        self.history_index = self.history.len() - 1;
    }
    pub(crate) fn prev_page(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
        }
    }
    pub(crate) fn next_page(&mut self) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
        }
    }
    pub fn prev_link(&mut self) {
        let rp = self.get_current_rendered_page();
        rp.page.selection -= 1;
        if rp.page.selection < 0 {
            rp.page.selection = rp.link_count - 1;
        }
    }
    pub fn next_link(&mut self) {
        let rp = self.get_current_rendered_page();
        rp.page.selection += 1;
        if rp.page.selection >= rp.link_count {
            rp.page.selection = 0;
        }
    }
    pub(crate) fn nav_current_link(&mut self) {
        let rp = self.get_current_rendered_page();
        if let Some(href) = rp.find_href_by_index(rp.page.selection) {
            info!("loading the href {}", href);
            let mut href = href.to_string();
            if !href.starts_with("http") {
                info!("doing a relative link. base is {}", rp.page.url);
                href = format!("{}{}", rp.page.url, href);
                info!("final url is {}", href);
            }
            // NET_COMMANDS
            //     .try_send(NetCommand::Load(href.to_string()))
            //     .unwrap()
        }
    }
    fn get_current_rendered_page(&mut self) -> &mut RenderedPage {
        &mut self.history[self.history_index]
    }
    fn get_imutable_page(&self) -> &RenderedPage {
        &self.history[self.history_index]
    }
}
// fn layout(&mut self, display: &mut dyn ViewTarget, theme: &Theme) {
//     self.bounds = Rectangle::new(Point::new(0, 0),  Size::new(display.size().width, display.size().height));
//     self.columns = display.size().width/theme.font.character_size.width
// }
//
fn draw(e: &mut DrawEvent) {
    if !e.view.visible {
        return;
    }
    // let font = context.theme.font;
    let font = FONT_9X15_BOLD;
    let line_height = font.character_size.height + 2;
    // let viewport_height: i32 = (context.display.size().height / line_height) as i32;
    let viewport_height: i32 = 240 / line_height as i32;
    let char_width = font.character_size.width as i32;

    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);

    // select the lines in the current viewport
    if let Some(state) = &e.view.state {
        if let Some(state) = state.downcast_ref::<PageView>() {
            let rpage = state.get_imutable_page();
            let mut end: usize = (rpage.scroll_index + viewport_height) as usize;
            if end >= rpage.lines.len() {
                end = rpage.lines.len();
            }
            let start = max(rpage.scroll_index, 0) as usize;
            let viewport_lines = &rpage.lines[start..end];

            let x_inset = 8;
            let y_inset = 5;

            let mut link_count = -1;
            // draw the lines
            for (j, line) in viewport_lines.iter().enumerate() {
                let mut inset_chars: usize = 0;
                let y = j as i32 * (line_height as i32) + 10;
                // let style = match line.block_type {
                //     BlockType::Paragraph => MonoTextStyle::new(&font, &theme.fg),
                //     BlockType::ListItem => MonoTextStyle::new(&font, &theme.fg),
                //     BlockType::Header => MonoTextStyle::new(&font, &theme.fg),
                // };
                // draw a bullet
                if line.block_type == BlockType::ListItem {
                    e.ctx.fill_rect(&Bounds::new(2, y, 4, 3), &e.theme.fg);
                }
                for run in &line.runs {
                    let pos = Point::new(inset_chars as i32 * char_width + x_inset, y + y_inset);
                    let plain_style =
                        TextStyle::new(&e.theme.font, &e.theme.fg).with_halign(HAlign::Left);
                    let text_style = match &run.style {
                        RunStyle::Link(href) => {
                            // info!("found a link: {:?}", href);
                            link_count += 1;
                            if rpage.page.selection == link_count {
                                plain_style.with_underline(true)
                            } else {
                                plain_style
                            }
                        }
                        RunStyle::Plain => plain_style,
                        RunStyle::Bold => plain_style,
                    };
                    e.ctx
                        .fill_text(&Bounds::new(pos.x, pos.y, 100, 10), &run.text, &text_style);
                    inset_chars += run.text.len();
                }
            }
        }
    }
}

fn handle_input(event: &mut GuiEvent) -> Option<Action> {
    event.scene.mark_dirty_view(event.target);
    if let Some(state) = event.scene.get_view_state::<PageView>(event.target) {
        match event.event_type {
            EventType::Keyboard(key) => {
                match key {
                    b'j' => {
                        let page = state.get_current_rendered_page();
                        page.scroll_index = (page.scroll_index + 10) % (page.lines.len() as i32)
                    }
                    b'k' => {
                        let page = state.get_current_rendered_page();
                        page.scroll_index = max(page.scroll_index - 10, 0)
                    }
                    b'a' => state.prev_link(),
                    b's' => state.next_link(),
                    13 => state.nav_current_link(),
                    _ => {
                        warn!("Unhandled key {:?}", key);
                    }
                }
                state.dirty = true
            }
            EventType::Scroll(dx, dy) => {
                if (dx < 0) || (dy < 0) {
                    state.prev_link();
                };
                if (dx > 0) || (dy > 0) {
                    state.next_link();
                };
            }
            _ => {}
        }
    }
    None
}
