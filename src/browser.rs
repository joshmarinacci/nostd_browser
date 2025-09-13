use crate::comps::{make_overlay_label, make_rect_view};
use crate::menuview::make_menuview;
use crate::page::Page;
use crate::pageview::PageView;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{format, vec};
use embedded_graphics::mono_font::ascii::{FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{RgbColor, WebColors};
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::geom::Bounds;
use gui2::toggle_group::{make_toggle_group, SelectOneOfState};
use gui2::{connect_parent_child, Action, EventType, GuiEvent, Scene};
use log::info;
use nostd_html_parser::blocks::{Block, BlockType};

const MAIN_MENU: &'static str = "main";
const BROWSER_MENU: &'static str = "browser";

const SETTINGS_PANEL: &'static str = "settings";
const WIFI_PANEL: &'static str = "wifi-panel";
const WIFI_MENU: &'static str = "wifi-menu";
const WIFI_BUTTON: &'static str = "wifi-button";
const FONT_MENU: &'static str = "font";
const THEME_MENU: &'static str = "theme";
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
    pub theme: AppTheme,
}
pub fn handle_action2<C, F>(event: &mut GuiEvent<C, F>) {
    info!(
        "handling action2 {:?} from {:?}",
        event.action, event.target
    );
    let act = event.action.clone();
    match act {
        Some(Action::Command(cmd)) => {
            if event.target == MAIN_MENU {
                match cmd.as_str() {
                    "Browser" => show_and_focus(event, BROWSER_MENU),
                    "Network" => show_wifi_panel(event),
                    "Settings" => {
                        event.scene.hide_view(MAIN_MENU);
                        show_settings_panel(event);
                    }
                    "Info" => show_info_panel(event),
                    "close" => {
                        event.scene.hide_view(MAIN_MENU);
                        event.scene.set_focused(PAGE_VIEW);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if event.target == BROWSER_MENU {
                match cmd.as_str() {
                    "Open URL" => {
                        show_url_panel(event);
                    }
                    "Bookmarks" => {
                        //         // show the bookmarks
                        //         NET_COMMANDS
                        //             .send(NetCommand::Load("bookmarks.html".to_string()))
                        //             .await;
                        event.scene.hide_view(MAIN_MENU);
                        event.scene.hide_view(BROWSER_MENU);
                        event.scene.set_focused(PAGE_VIEW);
                    }
                    "Back" => {
                        event.scene.hide_view(MAIN_MENU);
                        event.scene.hide_view(BROWSER_MENU);
                        if let Some(state) = event.scene.get_view_state::<PageView>(PAGE_VIEW) {
                            state.prev_page();
                        }
                        event.scene.set_focused(PAGE_VIEW);
                    }
                    "Forward" => {
                        event.scene.hide_view(MAIN_MENU);
                        event.scene.hide_view(BROWSER_MENU);
                        if let Some(page_view) = event.scene.get_view_state::<PageView>(PAGE_VIEW) {
                            page_view.next_page();
                        }
                        event.scene.set_focused(PAGE_VIEW);
                    }
                    "close" => {
                        event.scene.hide_view(BROWSER_MENU);
                        event.scene.set_focused(MAIN_MENU);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if event.target == WIFI_MENU {
                match cmd.as_str() {
                    "status" => info!("status"),
                    "scan" => info!("scan"),
                    "close" => {
                        event.scene.hide_view("wifi");
                        event.scene.set_focused(MAIN_MENU);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if event.target == "url-input" {
                info!("url input {}", cmd);
                if let Some(view) = event.scene.get_view_mut("url-input") {
                    info!("got the text {:?}", view.title);
                    // NET_COMMANDS
                    //     .send(NetCommand::Load(view.title.to_string()))
                    // .await;
                }
                event.scene.remove_parent_and_children(URL_PANEL);
                event.scene.hide_view(MAIN_MENU);
                event.scene.hide_view(BROWSER_MENU);
                event.scene.set_focused(PAGE_VIEW);
                return;
            }
        }
        Some(Action::Generic) => {
            info!("handling generic");
            if event.target == INFO_BUTTON {
                event.scene.remove_parent_and_children(INFO_PANEL);
                event.scene.set_focused(PAGE_VIEW);
                return;
            }
            if event.target == "settings-close-button" {
                event.scene.remove_parent_and_children(SETTINGS_PANEL);
                event.scene.set_focused(PAGE_VIEW);
            }
            if event.target == "url-cancel-button" {
                event.scene.remove_parent_and_children(URL_PANEL);
                event.scene.set_focused(PAGE_VIEW);
            }
            if event.target == "url-load-button" {
                event.scene.remove_parent_and_children(URL_PANEL);
                event.scene.set_focused(PAGE_VIEW);
            }
            if event.target == WIFI_BUTTON {
                event.scene.remove_parent_and_children(WIFI_PANEL);
                event.scene.set_focused(PAGE_VIEW);
            }
            if event.target == "settings-theme" {
                info!("switching theme");
                if let Some(state) = event.scene.get_view_state::<SelectOneOfState>(event.target) {
                    if state.selected == 0 {
                        ACTIVE_THEME.insert(Box::new(&LIGHT_THEME));
                    }
                    if state.selected == 1 {
                        ACTIVE_THEME.insert(Box::new(&DARK_THEME));
                    }
                }
            }
        }
        None => {
            info!("no action")
        }
    }
}
fn show_url_panel<C, F>(event: &mut GuiEvent<C, F>) {
    let panel = make_panel(URL_PANEL, Bounds::new(20, 20, 320 - 40, 240 - 40));
    event.scene.add_view_to_parent(
        make_label("url-label", "URL").position_at(40, 40),
        &panel.name,
    );
    let mut input = make_text_input("url-input", "https://apps.josh.earth").position_at(40, 70);
    input.bounds.w = 200;
    event.scene.add_view_to_parent(input, &panel.name);
    event.scene.add_view_to_parent(
        make_button("url-cancel-button", "cancel").position_at(60, 160),
        &panel.name,
    );
    event.scene.add_view_to_parent(
        make_button("url-load-button", "load").position_at(160, 160),
        &panel.name,
    );
    event.scene.add_view_to_root(panel);
    event.scene.hide_view(MAIN_MENU);
    event.scene.hide_view(BROWSER_MENU);
    event.scene.set_focused("url-input");
}
fn show_info_panel<C, F>(event: &mut GuiEvent<C, F>) {
    info!("showing the info panel");
    let panel_bounds = Bounds::new(20, 20, 320 - 40, 240 - 40);
    let panel = make_panel(INFO_PANEL, panel_bounds.clone());

    let free = esp_alloc::HEAP.free();
    let used = esp_alloc::HEAP.used();

    let label1a = make_label("info-label1", "Heap").position_at(120, 50);
    event.scene.add_child(&panel.name, &label1a.name);
    event.scene.add_view(label1a);

    let label2a = make_label("info-label2a", "Free memory").position_at(60, 80);
    event.scene.add_child(&panel.name, &label2a.name);
    event.scene.add_view(label2a);

    let label2b = make_label("info-label2b", &format!("{:?}", free)).position_at(200, 80);
    event.scene.add_child(&panel.name, &label2b.name);
    event.scene.add_view(label2b);

    let label3a = make_label("info-label3a", "Used memory").position_at(60, 100);
    event.scene.add_child(&panel.name, &label3a.name);
    event.scene.add_view(label3a);

    let label3b = make_label("info-label3b", &format!("{:?}", used)).position_at(200, 100);
    event.scene.add_child(&panel.name, &label3b.name);
    event.scene.add_view(label3b);

    let label4a = make_label("info-label4a", "Total memory").position_at(60, 120);
    event.scene.add_child(&panel.name, &label4a.name);
    event.scene.add_view(label4a);

    let label4b = make_label("info-label4b", &format!("{:?}", free + used)).position_at(200, 120);
    event.scene.add_child(&panel.name, &label4b.name);
    event.scene.add_view(label4b);

    let button = make_button(INFO_BUTTON, "done").position_at(160 - 20, 200 - 20);
    event.scene.add_child(&panel.name, &button.name);
    event.scene.add_view(button);

    event.scene.add_view_to_root(panel);

    event.scene.hide_view(MAIN_MENU);
    event.scene.set_focused(INFO_BUTTON);
}
fn show_wifi_panel<C, F>(event: &mut GuiEvent<C, F>) {
    let panel = make_panel(WIFI_PANEL, Bounds::new(20, 20, 320 - 40, 240 - 40));
    let label1a = make_label("wifi-label1a", "SSID").position_at(60, 80);
    // let label1b = Label::new(SSID.unwrap_or("----"), Point::new(150, 80));
    let label2a = make_label("wifi-label2a", "PASSWORD").position_at(60, 100);
    // let label2b = Label::new(PASSWORD.unwrap_or("----"), Point::new(150, 100));
    let button = make_button(WIFI_BUTTON, "done").position_at(160 - 20, 200 - 20);

    event.scene.add_view_to_root(panel);
    connect_parent_child(event.scene, WIFI_PANEL, &label1a.name);
    connect_parent_child(event.scene, WIFI_PANEL, &label2a.name);
    connect_parent_child(event.scene, WIFI_PANEL, &button.name);
    event.scene.add_view(label1a);
    event.scene.add_view(label2a);
    event.scene.add_view(button);
    // scene.add("wifi-label1b", label1b);
    // scene.add("wifi-label2b", label2b);
    event.scene.hide_view(MAIN_MENU);
    event.scene.set_focused(WIFI_BUTTON);
}
fn show_settings_panel<C, F>(event: &mut GuiEvent<C, F>) {
    info!("showing settings panel");
    let panel = make_panel(SETTINGS_PANEL, Bounds::new(20, 20, 320 - 40, 240 - 40));
    event.scene.add_view_to_parent(
        make_label("settings-theme-label", "Theme").position_at(60, 40),
        &panel.name,
    );
    event.scene.add_view_to_parent(
        make_toggle_group("settings-theme", vec!["Light", "Dark"], 0).position_at(100, 40),
        &panel.name,
    );
    event.scene.add_view_to_parent(
        make_label("settings-font-label", "Font").position_at(60, 80),
        &panel.name,
    );
    event.scene.add_view_to_parent(
        make_button("settings-font-button", "Small").position_at(100, 80),
        &panel.name,
    );

    event.scene.add_view_to_parent(
        make_button("settings-close-button", "Close").position_at(130, 140),
        &panel.name,
    );
    event.scene.add_view_to_root(panel);
}

pub fn make_gui_scene() -> Scene<Rgb565, MonoFont<'static>> {
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
    .position_at(0, 0)
    .hide();
    scene.add_view_to_root(main_menu);
    // scene.set_focused("menu");

    // scene.add_view_to_root(
    //     make_menuview(THEME_MENU, vec!["Light", "Dark", "close"])
    //         .position_at(20, 20)
    //         .hide(),
    // );
    //
    // scene.add_view_to_root(
    //     make_menuview(FONT_MENU, vec!["Small", "Medium", "Large", "close"])
    //         .position_at(20, 20)
    //         .hide(),
    // );

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
    scene.add_view_to_root(make_rect_view("touch-overlay"));
    scene
}

pub fn update_view_from_input<C, F>(event: &mut GuiEvent<C, F>) {
    match &event.event_type {
        EventType::Keyboard(key) => {
            if *key == b' ' {
                if is_visible(event, MAIN_MENU) == false && event.scene.is_focused(PAGE_VIEW) {
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
    if let Some(_action) = &event.action {
        handle_action2(event);
    }
}

fn show_and_focus<C, F>(event: &mut GuiEvent<C, F>, name: &str) {
    event.scene.show_view(name);
    event.scene.set_focused(name);
}
fn is_visible<C, F>(event: &GuiEvent<C, F>, name: &str) -> bool {
    if let Some(menu) = event.scene.get_view(name) {
        menu.visible
    } else {
        false
    }
}
