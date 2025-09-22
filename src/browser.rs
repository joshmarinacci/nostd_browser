// use crate::common::{NetCommand, NET_COMMANDS};
use crate::comps::make_overlay_label;
use crate::menuview::make_menuview;
use crate::page::Page;
use crate::pageview::PageView;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{format, vec};
use embedded_graphics::mono_font::ascii::{
    FONT_6X13, FONT_6X13_BOLD, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD,
};
use embedded_graphics::mono_font::iso_8859_10::FONT_7X13;
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{RgbColor, WebColors};
use log::info;
use nostd_html_parser::blocks::{Block, BlockType};
use rust_embedded_gui::button::make_button;
use rust_embedded_gui::geom::Bounds;
use rust_embedded_gui::label::make_label;
use rust_embedded_gui::panel::make_panel;
use rust_embedded_gui::scene::Scene;
use rust_embedded_gui::text_input::make_text_input;
use rust_embedded_gui::toggle_group::make_toggle_group;
use rust_embedded_gui::{Action, EventType, GuiEvent};

const MAIN_MENU: &'static str = "main";
const BROWSER_MENU: &'static str = "browser";

const SETTINGS_PANEL: &'static str = "settings";
const WIFI_PANEL: &'static str = "wifi-panel";
const WIFI_MENU: &'static str = "wifi-menu";
const WIFI_BUTTON: &'static str = "wifi-button";
pub const PAGE_VIEW: &'static str = "page";

const INFO_PANEL: &'static str = "info-panel";
const INFO_BUTTON: &'static str = "info-button";

const URL_PANEL: &'static str = "url-panel";

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

pub const ACTIVE_THEME: Option<Box<&AppTheme>> = None;

pub struct AppState {
    pub theme: &'static AppTheme,
    pub font: &'static MonoFont<'static>,
    pub bold_font: &'static MonoFont<'static>,
}
pub fn handle_action2(target: &str, action: &Action, scene: &mut Scene, app: &mut AppState) {
    info!("handling action2 {:?} from {:?}", action, target);
    match action {
        Action::Command(cmd) => {
            if target == MAIN_MENU {
                match cmd.as_str() {
                    "Browser" => {
                        scene.show_view(BROWSER_MENU);
                        scene.set_focused(BROWSER_MENU);
                    }
                    "Network" => show_wifi_panel(scene),
                    "Settings" => {
                        scene.hide_view(MAIN_MENU);
                        show_settings_panel(scene);
                    }
                    "Info" => show_info_panel(scene),
                    "close" => {
                        scene.hide_view(MAIN_MENU);
                        scene.set_focused(PAGE_VIEW);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if target == BROWSER_MENU {
                match cmd.as_str() {
                    "Open URL" => {
                        show_url_panel(scene);
                    }
                    "Bookmarks" => {
                        // show the bookmarks
                        // NET_COMMANDS.send(NetCommand::Load("bookmarks.html".to_string()));
                        scene.hide_view(MAIN_MENU);
                        scene.hide_view(BROWSER_MENU);
                        scene.set_focused(PAGE_VIEW);
                    }
                    "Back" => {
                        scene.hide_view(MAIN_MENU);
                        scene.hide_view(BROWSER_MENU);
                        if let Some(state) = scene.get_view_state::<PageView>(PAGE_VIEW) {
                            state.prev_page();
                        }
                        scene.set_focused(PAGE_VIEW);
                    }
                    "Forward" => {
                        scene.hide_view(MAIN_MENU);
                        scene.hide_view(BROWSER_MENU);
                        if let Some(page_view) = scene.get_view_state::<PageView>(PAGE_VIEW) {
                            page_view.next_page();
                        }
                        scene.set_focused(PAGE_VIEW);
                    }
                    "close" => {
                        scene.hide_view(BROWSER_MENU);
                        scene.set_focused(MAIN_MENU);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if target == WIFI_MENU {
                match cmd.as_str() {
                    "status" => info!("status"),
                    "scan" => info!("scan"),
                    "close" => {
                        scene.hide_view("wifi");
                        scene.set_focused(MAIN_MENU);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if target == "url-input" {
                info!("url input {}", cmd);
                if let Some(view) = scene.get_view_mut("url-input") {
                    info!("got the text {:?}", view.title);
                    // NET_COMMANDS
                    //     .send(NetCommand::Load(view.title.to_string()))
                    // .await;
                }
                scene.remove_parent_and_children(URL_PANEL);
                scene.hide_view(MAIN_MENU);
                scene.hide_view(BROWSER_MENU);
                scene.set_focused(PAGE_VIEW);
                return;
            }
            if target == "settings-theme" {
                if cmd == "Dark" {
                    app.theme = &DARK_THEME;
                    scene.mark_dirty_all();
                }
                if cmd == "Light" {
                    app.theme = &LIGHT_THEME;
                    scene.mark_dirty_all();
                }
            }
            if target == "font-menu" {
                match cmd.as_str() {
                    "Small" => {
                        app.font = &FONT_6X13;
                        app.font = &FONT_6X13_BOLD;
                        scene.mark_dirty_all();
                        scene.hide_view("font-menu");
                    }
                    "Medium" => {
                        app.font = &FONT_7X13;
                        app.bold_font = &FONT_7X13_BOLD;
                        scene.mark_dirty_all();
                        scene.hide_view("font-menu");
                    }
                    "Large" => {
                        app.font = &FONT_9X15;
                        app.bold_font = &FONT_9X15_BOLD;
                        scene.mark_dirty_all();
                        scene.hide_view("font-menu");
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
        }
        Action::Generic => {
            info!("handling generic");
            if target == INFO_BUTTON {
                scene.remove_parent_and_children(INFO_PANEL);
                scene.set_focused(PAGE_VIEW);
                return;
            }
            if target == "settings-close-button" {
                scene.remove_parent_and_children(SETTINGS_PANEL);
                scene.set_focused(PAGE_VIEW);
            }
            if target == "url-cancel-button" {
                scene.remove_parent_and_children(URL_PANEL);
                scene.set_focused(PAGE_VIEW);
            }
            if target == "url-load-button" {
                scene.remove_parent_and_children(URL_PANEL);
                scene.set_focused(PAGE_VIEW);
            }
            if target == "settings-font-button" {
                let font_menu = make_menuview("font-menu", vec!["Small", "Medium", "Large"])
                    .position_at(150, 70);
                scene.add_view_to_root(font_menu);
            }
            if target == WIFI_BUTTON {
                scene.remove_parent_and_children(WIFI_PANEL);
                scene.set_focused(PAGE_VIEW);
            }
        }
    }
}
fn show_url_panel(scene: &mut Scene) {
    let panel = make_panel(URL_PANEL, Bounds::new(20, 20, 320 - 40, 240 - 40));
    scene.add_view_to_parent(
        make_label("url-label", "URL").position_at(40, 40),
        &panel.name,
    );
    let mut input = make_text_input("url-input", "https://apps.josh.earth").position_at(40, 70);
    input.bounds.w = 200;
    scene.add_view_to_parent(input, &panel.name);
    scene.add_view_to_parent(
        make_button("url-cancel-button", "cancel").position_at(60, 160),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_button("url-load-button", "load").position_at(160, 160),
        &panel.name,
    );
    scene.add_view_to_root(panel);
    scene.hide_view(MAIN_MENU);
    scene.hide_view(BROWSER_MENU);
    scene.set_focused("url-input");
}
fn show_info_panel(scene: &mut Scene) {
    info!("showing the info panel");
    let panel_bounds = Bounds::new(20, 20, 320 - 40, 240 - 40);
    let panel = make_panel(INFO_PANEL, panel_bounds.clone());

    // let free = esp_alloc::HEAP.free();
    // let used = esp_alloc::HEAP.used();
    let free = 0;
    let used = 0;

    let label1a = make_label("info-label1", "Heap").position_at(100, 30);
    scene.add_child(&panel.name, &label1a.name);
    scene.add_view(label1a);

    let label2a = make_label("info-label2a", "Free memory").position_at(40, 60);
    scene.add_child(&panel.name, &label2a.name);
    scene.add_view(label2a);

    let label2b = make_label("info-label2b", &format!("{:?}", free)).position_at(180, 60);
    scene.add_child(&panel.name, &label2b.name);
    scene.add_view(label2b);

    let label3a = make_label("info-label3a", "Used memory").position_at(40, 80);
    scene.add_child(&panel.name, &label3a.name);
    scene.add_view(label3a);

    let label3b = make_label("info-label3b", &format!("{:?}", used)).position_at(180, 80);
    scene.add_child(&panel.name, &label3b.name);
    scene.add_view(label3b);

    let label4a = make_label("info-label4a", "Total memory").position_at(40, 100);
    scene.add_child(&panel.name, &label4a.name);
    scene.add_view(label4a);

    let label4b = make_label("info-label4b", &format!("{:?}", free + used)).position_at(180, 100);
    scene.add_child(&panel.name, &label4b.name);
    scene.add_view(label4b);

    let button = make_button(INFO_BUTTON, "done").position_at(160 - 20-20, 200 - 20-20);
    scene.add_child(&panel.name, &button.name);
    scene.add_view(button);

    scene.add_view_to_root(panel);

    scene.hide_view(MAIN_MENU);
    scene.set_focused(INFO_BUTTON);
}
fn show_wifi_panel(scene: &mut Scene) {
    let panel = make_panel(WIFI_PANEL, Bounds::new(20, 20, 320 - 40, 240 - 40));
    let label1a = make_label("wifi-label1a", "SSID").position_at(40, 40);
    let label2a = make_label("wifi-label2a", "PASSWORD").position_at(40, 60);
    let button = make_button(WIFI_BUTTON, "done").position_at(160 - 20, 120);

    scene.add_view_to_root(panel);
    scene.add_view_to_parent(label1a, WIFI_PANEL);
    scene.add_view_to_parent(label2a, WIFI_PANEL);
    scene.add_view_to_parent(button, WIFI_PANEL);
    scene.hide_view(MAIN_MENU);
    scene.set_focused(WIFI_BUTTON);
}
fn show_settings_panel(scene: &mut Scene) {
    info!("showing settings panel");
    let panel = make_panel(SETTINGS_PANEL, Bounds::new(20, 20, 320 - 40, 240 - 40));
    scene.add_view_to_parent(
        make_label("settings-theme-label", "Theme").position_at(20, 20),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_toggle_group("settings-theme", vec!["Light", "Dark"], 0).position_at(80, 20),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_label("settings-font-label", "Font").position_at(40, 60),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_button("settings-font-button", "Small").position_at(80, 60),
        &panel.name,
    );

    scene.add_view_to_parent(
        make_button("settings-close-button", "Close").position_at(110, 120),
        &panel.name,
    );
    scene.add_view_to_root(panel);
}
pub fn make_gui_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let panel = make_panel("panel", Bounds::new(20, 20, 260, 200));
    scene.add_view_to_root(panel);

    let full_screen_bounds = Bounds::new(0, 0, 320, 240);
    let page_view = PageView::new(full_screen_bounds, Page::new());
    scene.add_view_to_root(page_view);
    let main_menu = make_menuview(
        MAIN_MENU,
        vec![
            "Browser".into(),
            "Network".into(),
            "Settings".into(),
            "Info".into(),
            "close".into(),
        ],
    )
    .position_at(0, 0);

    scene.add_view_to_root(main_menu);

    scene.add_view_to_root(
        make_menuview(WIFI_MENU, vec!["status", "scan", "close"])
            .position_at(20, 20)
            .hide(),
    );

    let browser_menu = make_menuview(
        BROWSER_MENU,
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
    if let Some(page_view) = scene.get_view_state::<PageView>(PAGE_VIEW) {
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
        page_view.load_page(page);
    }

    scene.set_focused(PAGE_VIEW);

    scene.add_view_to_root(make_overlay_label("overlay-status", "some info").position_at(200, 200));
    // scene.add_view_to_root(make_rect_view("touch-overlay"));
    scene
}

pub fn update_view_from_keyboard_input(scene: &mut Scene, key: u8) {
    if key == b' ' {
        if scene.is_visible(MAIN_MENU) == false && scene.is_focused(PAGE_VIEW) {
            scene.show_view(MAIN_MENU);
            scene.set_focused(MAIN_MENU);
            return;
        }
    }
}
pub fn update_view_from_input(event: &mut GuiEvent, app: &mut AppState) {
    match &event.event_type {
        EventType::Keyboard(key) => {
            if *key == b' ' {
                if event.scene.is_visible(MAIN_MENU) == false && event.scene.is_focused(PAGE_VIEW) {
                    event.scene.show_view(MAIN_MENU);
                    event.scene.set_focused(MAIN_MENU);
                    return;
                } else {
                    info!("trigger an action");
                    // handle_action(event);
                }
            }
        }
        EventType::Tap(pt) => {
            info!("tapped on point {pt:?}");
        }
        _ => {}
    }
}
