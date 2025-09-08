use crate::common::{NetCommand, TDeckDisplay, NET_COMMANDS};
use crate::menuview::{make_menuview, MenuState};
use crate::page::Page;
use crate::pageview::PageView;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{format, vec};
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::ascii::{
    FONT_6X13, FONT_6X13_BOLD, FONT_8X13, FONT_8X13_BOLD, FONT_9X15, FONT_9X15_BOLD,
};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{RgbColor, WebColors};
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::geom::Bounds;
use gui2::{connect_parent_child, EventType, GuiEvent, Scene};
use log::info;
use nostd_html_parser::blocks::{Block, BlockType};

const MAIN_MENU: &'static str = "main";
const FONT_MENU: &'static str = "font";
const THEME_MENU: &'static str = "theme";
pub const PAGE_VIEW: &'static str = "page";

pub const BASE_FONT: MonoFont = FONT_9X15;
pub const BOLD_FONT: MonoFont = FONT_9X15_BOLD;
pub struct AppTheme {
    pub base_bg: Rgb565,
    pub base_bd: Rgb565,
    pub base_fg: Rgb565,
    pub accent_fg: Rgb565,
    pub highlight_fg: Rgb565,
    pub shadow: bool,
    pub font: MonoFont<'static>,
    pub bold: MonoFont<'static>,
}
pub const LIGHT_THEME: AppTheme = AppTheme {
    base_bg: Rgb565::WHITE,
    base_bd: Rgb565::BLACK,
    base_fg: Rgb565::BLACK,
    accent_fg: Rgb565::BLUE,
    highlight_fg: Rgb565::CSS_ORANGE_RED,
    shadow: false,
    font: BASE_FONT,
    bold: BOLD_FONT,
};
pub const DARK_THEME: AppTheme = AppTheme {
    base_bg: Rgb565::BLACK,
    base_bd: Rgb565::WHITE,
    base_fg: Rgb565::WHITE,
    accent_fg: Rgb565::CSS_DARK_BLUE,
    highlight_fg: Rgb565::CSS_DARK_ORANGE,
    font: BASE_FONT,
    bold: BOLD_FONT,
    shadow: false,
};

pub const THEME: Option<Box<&AppTheme>> = None;

pub fn handle_action<C, F>(event: &mut GuiEvent<C, F>) {
    let panel_bounds = Bounds::new(20, 20, 320 - 40, 240 - 40);
    if event.target == MAIN_MENU {
        info!("clicked on the main menu");
        if menu_item_selected(event, MAIN_MENU, "Theme") {
            show_and_focus(event, THEME_MENU);
            return;
        }
        if menu_item_selected(event, MAIN_MENU, "Font") {
            show_and_focus(event, FONT_MENU);
            return;
        }
        if menu_item_selected(event, MAIN_MENU, "Browser") {
            show_and_focus(event, "browser");
            return;
        }
        if menu_item_selected(event, MAIN_MENU, "Wifi") {
            let panel = make_panel("panel1", panel_bounds.clone());
            let mut label1a = make_label("wifi-label1a", "SSID").position_at(60, 80);
            // let label1b = Label::new(SSID.unwrap_or("----"), Point::new(150, 80));
            let label2a = make_label("wifi-label2a", "PASSWORD").position_at(60, 100);
            // let label2b = Label::new(PASSWORD.unwrap_or("----"), Point::new(150, 100));
            let button = make_button("wifi-button", "done").position_at(160 - 20, 200 - 20);

            connect_parent_child(event.scene, &panel.name, &label1a.name);
            connect_parent_child(event.scene, &panel.name, &label2a.name);
            connect_parent_child(event.scene, &panel.name, &button.name);
            event.scene.add_view_to_root(panel);
            event.scene.add_view(label1a);
            event.scene.add_view(label2a);
            event.scene.add_view(button);
            // scene.add("wifi-label1b", label1b);
            // scene.add("wifi-label2b", label2b);
            event.scene.hide_view(MAIN_MENU);
            event.scene.set_focused("wifi-button");
            return;
        }
        if menu_item_selected(event, MAIN_MENU, "Info") {
            info!("showing the info panel");
            let panel = make_panel("panel1", panel_bounds.clone());

            let free = esp_alloc::HEAP.free();
            let used = esp_alloc::HEAP.used();

            let label1a = make_label("info-label1", "Heap").position_at(120, 50);
            connect_parent_child(event.scene, &panel.name, &label1a.name);
            event.scene.add_view(label1a);

            let label2a = make_label("info-label2a", "Free memory").position_at(60, 80);
            connect_parent_child(event.scene, &panel.name, &label2a.name);
            event.scene.add_view(label2a);

            let label2b = make_label("info-label2b", &format!("{:?}", free)).position_at(200,80);
            connect_parent_child(event.scene, &panel.name, &label2b.name);
            event.scene.add_view(label2b);

            let label3a = make_label("info-label3a", "Used memory").position_at(60,100);
            connect_parent_child(event.scene, &panel.name, &label3a.name);
            event.scene.add_view(label3a);

            let label3b = make_label("info-label3b", &format!("{:?}", used)).position_at(200,100);
            connect_parent_child(event.scene, &panel.name, &label3b.name);
            event.scene.add_view(label3b);

            let label4a = make_label("info-label4a", "Total memory").position_at(60,120);
            connect_parent_child(event.scene, &panel.name, &label4a.name);
            event.scene.add_view(label4a);

            let label4b = make_label("info-label4b", &format!("{:?}", free + used)).position_at(200,120);
            connect_parent_child(event.scene, &panel.name, &label4b.name);
            event.scene.add_view(label4b);

            let button = make_button("info-button", "done").position_at(160 - 20, 200 - 20);
            connect_parent_child(event.scene, &panel.name, &button.name);
            event.scene.add_view(button);

            event.scene.add_view_to_root(panel);

            event.scene.hide_view(MAIN_MENU);
            event.scene.set_focused("info-button");
            return;
        }
        if menu_item_selected(event, MAIN_MENU, "close") {
            event.scene.hide_view(MAIN_MENU);
            event.scene.set_focused(PAGE_VIEW);
            return;
        }
    }
    if event.scene.is_focused("wifi-button") {
        info!("clicked the button");
        event.scene.remove_view("wifi-panel");
        event.scene.remove_view("wifi-label1a");
        event.scene.remove_view("wifi-label1b");
        event.scene.remove_view("wifi-label2a");
        event.scene.remove_view("wifi-label2b");
        event.scene.remove_view("wifi-button");
        return;
    }
    if event.scene.is_focused("info-button") {
        info!("clicked the info button");
        event.scene.remove_view("info-panel");
        event.scene.remove_view("info-label1");
        event.scene.remove_view("info-label2a");
        event.scene.remove_view("info-label2b");
        event.scene.remove_view("info-label3a");
        event.scene.remove_view("info-label3b");
        event.scene.remove_view("info-label4a");
        event.scene.remove_view("info-label4b");
        event.scene.remove_view("info-button");
        return;
    }
    if event.scene.is_focused(THEME_MENU) {
        if menu_item_selected(event, THEME_MENU, "Dark") {
            // THEME.insert(DARK_THEME);
            event.scene.mark_dirty();
            return;
        }
        if menu_item_selected(event, THEME_MENU, "Light") {
            // THEME.insert(LIGHT_THEME);
            // scene.set_theme(LIGHT_THEME);
            return;
        }
        if menu_item_selected(event, THEME_MENU, "close") {
            event.scene.hide_view(THEME_MENU);
            event.scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if event.scene.is_focused(FONT_MENU) {
        if menu_item_selected(event, FONT_MENU, "Small") {
            set_font(FONT_6X13, FONT_6X13_BOLD);
        }
        if menu_item_selected(event, FONT_MENU, "Medium") {
            set_font(FONT_8X13, FONT_8X13_BOLD);
        }
        if menu_item_selected(event, FONT_MENU, "Large") {
            set_font(FONT_9X15, FONT_9X15_BOLD);
        }
        if menu_item_selected(event, FONT_MENU, "close") {
            event.scene.hide_view(FONT_MENU);
            event.scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if event.scene.is_focused("wifi") {
        if menu_item_selected(event, "wifi", "close") {
            event.scene.hide_view("wifi");
            event.scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if event.scene.is_focused("browser") {
        if menu_item_selected(event, "browser", "Bookmarks") {
            //         // show the bookmarks
            //         NET_COMMANDS
            //             .send(NetCommand::Load("bookmarks.html".to_string()))
            //             .await;
            event.scene.hide_view(MAIN_MENU);
            event.scene.hide_view("browser");
            event.scene.set_focused(PAGE_VIEW);
            return;
        }
        if menu_item_selected(event, "browser", "Open URL") {
            let panel = make_panel("url-panel", panel_bounds);
            let label = make_label("url-label", "URL").position_at(40, 40);
            let input = make_text_input("url-input", "https://apps.josh.earth").position_at(40, 70);
            let button = make_button("url-button", "load").position_at(160 - 20, 200 - 20);
            connect_parent_child(event.scene, &panel.name, &label.name);
            connect_parent_child(event.scene, &panel.name, &input.name);
            connect_parent_child(event.scene, &panel.name, &button.name);
            event.scene.add_view(label);
            event.scene.add_view(input);
            event.scene.add_view(button);
            event.scene.add_view_to_root(panel);
            event.scene.hide_view("browser");
            event.scene.set_focused("url-input");
            return;
        }
        if menu_item_selected(event, "browser", "Back") {
            event.scene.hide_view(MAIN_MENU);
            event.scene.hide_view("browser");
            if let Some(view) = event.scene.get_view_mut(PAGE_VIEW) {
                if let Some(view) = &mut view.state {
                    if let Some(page_view) = view.downcast_mut::<PageView>() {
                        page_view.prev_page();
                    }
                }
            }
            event.scene.set_focused(PAGE_VIEW);
        }
        if menu_item_selected(event, "browser", "Forward") {
            event.scene.hide_view(MAIN_MENU);
            event.scene.hide_view("browser");
            if let Some(view) = event.scene.get_view_mut(PAGE_VIEW) {
                if let Some(view) = &mut view.state {
                    if let Some(page_view) = view.downcast_mut::<PageView>() {
                        page_view.next_page();
                    }
                }
            }
            event.scene.set_focused(PAGE_VIEW);
        }
        if menu_item_selected(event, "browser", "close") {
            event.scene.hide_view("browser");
            event.scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if event.scene.is_focused("url-input") {
        event.scene.remove_view("url-panel");
        event.scene.remove_view("url-label");
        event.scene.remove_view("url-button");
        event.scene.hide_view(MAIN_MENU);
        event.scene.hide_view("browser");
        event.scene.set_focused(PAGE_VIEW);
        if let Some(view) = event.scene.get_view_mut("url-input") {
            info!("got the text {:?}", view.title);
            // let href = &input.text;
            // NET_COMMANDS
            //     .send(NetCommand::Load(href.to_string()))
            //     .await;
        }
        event.scene.remove_view("url-input");
        return;
    }
    if event.scene.is_focused("url-button") {
        info!("clicked the button");
        return;
    }
}

fn set_font(plain: MonoFont, bold: MonoFont) {
    todo!()
}

fn menu_item_selected<C, F>(event: &mut GuiEvent<C, F>, name: &str, text: &str) -> bool {
    if let Some(view) = event.scene.get_view(name) {
        if let Some(state) = &view.state {
            if let Some(state) = state.downcast_ref::<MenuState>() {
                info!("menu_item_selected: state={:?}", state.selected);
                let selected_text = &state.data[state.selected as usize];
                if selected_text == text {
                    return true;
                }
            }
        }
    }
    return false;
}

pub fn make_gui_scene() -> Scene<Rgb565, MonoFont<'static>> {
    let mut scene = Scene::new();

    let panel = make_panel("panel", Bounds::new(20, 20, 260, 200));
    scene.add_view_to_root(panel);

    let full_screen_bounds = Bounds::new(0, 0, 320, 240);
    let textview = PageView::new(full_screen_bounds, Page::new());
    scene.add_view_to_root(textview);
    let mut menuview = make_menuview(
        "main",
        vec![
            "Browser".into(),
            "Network".into(),
            "Settings".into(),
            "Info".into(),
            "close".into(),
        ],
    );
    menuview.bounds.x = 0;
    menuview.bounds.y = 0;
    menuview.visible = false;
    scene.add_view_to_root(menuview);
    scene.set_focused("menu");

    scene.add_view_to_root(
        make_menuview(THEME_MENU, vec!["Light", "Dark", "close"])
            .position_at(20, 20)
            .hide(),
    );

    scene.add_view_to_root(
        make_menuview(FONT_MENU, vec!["Small", "Medium", "Large", "close"])
            .position_at(20, 20)
            .hide(),
    );

    scene.add_view_to_root(
        make_menuview("wifi", vec!["status", "scan", "close"])
            .position_at(20, 20)
            .hide(),
    );

    let browser_menu = make_menuview(
        "browser",
        vec![
            "Bookmarks",
            "SDCard",
            "Open URL",
            "Back",
            "Forward",
            "close",
        ],
    )
    .position_at(20, 20)
    .hide();
    scene.add_view_to_root(browser_menu);

    // set up a fake page
    if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
        if let Some(state) = &mut view.state {
            if let Some(tv) = state.downcast_mut::<PageView>() {
                let mut blocks = vec![];
                blocks.push(Block::new_of_type(BlockType::Header, "Header Text"));
                for i in [0..10] {
                    blocks.push(Block::new_of_type(
                        BlockType::ListItem,
                        &format!("list item {i:?}"),
                    ));
                }
                blocks.push(Block::new_of_type(
                    BlockType::Paragraph,
                    "This is some long body text that needs to be broken into multiple lines",
                ));
                let page = Page {
                    selection: 0,
                    blocks,
                    links: vec![],
                    url: "".to_string(),
                };
                tv.load_page(page);
            }
        }
    }

    scene.set_focused(PAGE_VIEW);

    // let info_panel_bounds = Rectangle::new(Point::new(120, 210), Size::new(200, 30));
    // scene.add("status", OverlayLabel::new("some info", info_panel_bounds));
    scene
}

pub fn update_view_from_input<C, F>(event: &mut GuiEvent<C, F>) {
    info!(
        "update view from input {:?} {:?}",
        event.target, event.event_type
    );
    match event.event_type {
        EventType::Keyboard(key) => {
            if key == b' ' && is_visible(event, MAIN_MENU) == false {
                event.scene.show_view(MAIN_MENU);
                event.scene.set_focused(MAIN_MENU);
                return;
            }
        }
        _ => {
            // scene.handle_input(event);
        }
    }
    handle_action(event);
}

fn show_and_focus<C, F>(event: &mut GuiEvent<C, F>, name: &str) {
    if let Some(menu) = event.scene.get_view_mut(name) {
        menu.visible = true;
    }
    event.scene.set_focused(name);
    event.scene.mark_dirty();
}
fn is_visible<C, F>(event: &GuiEvent<C, F>, name: &str) -> bool {
    if let Some(menu) = event.scene.get_view(name) {
        menu.visible
    } else {
        false
    }
}
