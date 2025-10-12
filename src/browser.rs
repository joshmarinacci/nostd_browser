// use crate::common::{NetCommand, NET_COMMANDS};
use crate::comps::make_overlay_label;
use crate::page::Page;
use crate::pageview::PageView;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
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
use iris_ui::button::{make_button, make_full_button};
use iris_ui::geom::Bounds;
use iris_ui::grid::{make_grid_panel, GridLayoutState};
use iris_ui::GuiEvent;
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::label::make_label;
use iris_ui::layouts::layout_vbox;
use iris_ui::list_view::make_list_view;
use iris_ui::panel::make_panel;
use iris_ui::scene::Scene;
use iris_ui::text_input::make_text_input;
use iris_ui::toggle_group::make_toggle_group;
use iris_ui::view::Flex::{Intrinsic, Resize};
use iris_ui::view::{View, ViewId};

const MAIN_MENU: &'static ViewId = &ViewId::new("main");
const BROWSER_MENU: &'static ViewId = &ViewId::new("browser");

const SETTINGS_PANEL: &'static ViewId = &ViewId::new("settings");
const WIFI_PANEL: &'static ViewId = &ViewId::new("wifi-panel");
const WIFI_MENU: &'static ViewId = &ViewId::new("wifi-menu");
const WIFI_BUTTON: &'static ViewId = &ViewId::new("wifi-button");
pub const PAGE_VIEW: &'static ViewId = &ViewId::new("page-view");

const INFO_PANEL: &'static ViewId = &ViewId::new("info-panel");
const INFO_BUTTON: &'static ViewId = &ViewId::new("info-button");

const URL_PANEL: &'static ViewId = &ViewId::new("url-panel");

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

#[derive(Debug)]
pub enum NetCommand {
    Load(String),
}

#[derive(Debug)]
pub enum GuiResponse {
    Net(NetCommand),
}

const cancel_url_command:&'static str = "cancel-url";
const load_url_command:&'static str = "load-url";

pub struct AppState {
    pub theme: &'static AppTheme,
    pub font: &'static MonoFont<'static>,
    pub bold_font: &'static MonoFont<'static>,
}
pub fn handle_action(
    result:&InputResult,
    scene: &mut Scene,
    app: &mut AppState,
) -> Option<GuiResponse> {
    info!("handling action2 {:?} from {:?}", result.action, result.source);
    match &result.action {
        Some(OutputAction::Command(cmd)) => {
            let url_input = ViewId::new("url-input");
            if cmd == cancel_url_command {
                scene.hide_view(URL_PANEL);
                scene.remove_parent_and_children(URL_PANEL);
                scene.set_focused(PAGE_VIEW);
            }
            if cmd == load_url_command {
                let result = if let Some(view) = scene.get_view(&url_input) {
                    Some(GuiResponse::Net(NetCommand::Load(view.title.to_string())))
                } else {
                    None
                };
                scene.remove_parent_and_children(URL_PANEL);
                scene.set_focused(PAGE_VIEW);
                return result;
            }
            if result.source == *MAIN_MENU {
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
            if result.source == *BROWSER_MENU {
                match cmd.as_str() {
                    "Open URL" => {
                        show_url_panel(scene);
                    }
                    "Bookmarks" => {
                        // show the bookmarks
                        scene.hide_view(MAIN_MENU);
                        scene.hide_view(BROWSER_MENU);
                        scene.set_focused(PAGE_VIEW);
                        return Some(GuiResponse::Net(NetCommand::Load(
                            "https://joshondesign.com/".to_string(),
                        )));
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
            if result.source == *WIFI_MENU {
                match cmd.as_str() {
                    "status" => info!("status"),
                    "scan" => info!("scan"),
                    "close" => {
                        scene.hide_view(&ViewId::new("wifi"));
                        scene.set_focused(MAIN_MENU);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if result.source == url_input {
                scene.remove_parent_and_children(URL_PANEL);
                scene.hide_view(MAIN_MENU);
                scene.hide_view(BROWSER_MENU);
                scene.set_focused(PAGE_VIEW);
                if let Some(view) = scene.get_view_mut(&url_input) {
                    return Some(GuiResponse::Net(NetCommand::Load(view.title.to_string())));
                }
            }
            if result.source == ViewId::new("settings-theme") {
                if cmd == "Dark" {
                    app.theme = &DARK_THEME;
                    scene.mark_dirty_all();
                }
                if cmd == "Light" {
                    app.theme = &LIGHT_THEME;
                    scene.mark_dirty_all();
                }
            }
            let font_menu = ViewId::new("font-menu");
            if result.source == ViewId::new("font-menu") {
                match cmd.as_str() {
                    "Small" => {
                        app.font = &FONT_6X13;
                        app.font = &FONT_6X13_BOLD;
                        scene.mark_dirty_all();
                        scene.hide_view(&font_menu);
                    }
                    "Medium" => {
                        app.font = &FONT_7X13;
                        app.bold_font = &FONT_7X13_BOLD;
                        scene.mark_dirty_all();
                        scene.hide_view(&font_menu);
                    }
                    "Large" => {
                        app.font = &FONT_9X15;
                        app.bold_font = &FONT_9X15_BOLD;
                        scene.mark_dirty_all();
                        scene.hide_view(&font_menu);
                    }
                    _ => {
                        info!("unknown menu item");
                    }
                }
            }
            if result.source == *PAGE_VIEW {
                return Some(GuiResponse::Net(NetCommand::Load(cmd.to_string())));
            }
        }
        _ => {

        }
    }
    info!("handling generic");
    if result.source == *INFO_BUTTON {
        scene.remove_parent_and_children(INFO_PANEL);
        scene.set_focused(PAGE_VIEW);
    }
    if result.source == ViewId::new("settings-close-button") {
        scene.remove_parent_and_children(SETTINGS_PANEL);
        scene.set_focused(PAGE_VIEW);
    }
    if result.source == ViewId::new("settings-font-button") {
        let font_menu_id = ViewId::new("font-menu");
        let font_menu = make_list_view(&font_menu_id, vec!["Small", "Medium", "Large"], 0)
            .position_at(150, 70);
        scene.add_view_to_root(font_menu);
        scene.set_focused(&font_menu_id);
    }
    if result.source == *WIFI_BUTTON {
        scene.remove_parent_and_children(WIFI_PANEL);
        scene.set_focused(PAGE_VIEW);
    }

    None
}

fn add_command_button_to(scene: &mut Scene, title:&str, command:&str, parent:&ViewId) {
    let btn = make_full_button(&scene.next_view_id(), title, command, false);
    scene.add_view_to_parent(btn,parent);
}
fn show_url_panel(scene: &mut Scene) {
    let panel = make_panel(URL_PANEL)
        .with_layout(Some(layout_vbox))
        .with_flex(Intrinsic, Intrinsic)
        .with_bounds(Bounds::new(20, 20, 320 - 40, 240 - 40));
    scene.add_view_to_parent(make_label("url-label", "URL"),&panel.name);
    let mut input = make_text_input("url-input", "https://apps.josh.earth")
        .with_flex(Resize,Intrinsic);
    scene.add_view_to_parent(input, &panel.name);
    add_command_button_to(scene, "Cancel", cancel_url_command, &panel.name);
    add_command_button_to(scene, "Load", load_url_command, &panel.name);
    scene.add_view_to_root(panel);
    scene.hide_view(MAIN_MENU);
    scene.hide_view(BROWSER_MENU);
    scene.set_focused(&ViewId::new("url-input"));
}
fn show_info_panel(scene: &mut Scene) {
    info!("showing the info panel");
    let panel_bounds = Bounds::new(20, 20, 320 - 40, 240 - 40);
    let mut panel = make_grid_panel(INFO_PANEL)
        .with_bounds(panel_bounds.clone())
        .with_flex(Intrinsic, Intrinsic)
        ;


    // let free = esp_alloc::HEAP.free();
    // let used = esp_alloc::HEAP.used();
    let free = 0;
    let used = 0;

    let mut layout = GridLayoutState::new_row_column(4, 30, 2, 100);

    let label1a = make_label("info-label1", "Heap");
    layout.place_at_row_column(&label1a.name, 0, 0);
    scene.add_view_to_parent(label1a,&panel.name);

    let label2a = make_label("info-label2a", "Free memory").position_at(40, 60);
    layout.place_at_row_column(&label2a.name, 1, 0);
    scene.add_view_to_parent(label2a,&panel.name);

    let label2b = make_label("info-label2b", &format!("{:?}", free)).position_at(180, 60);
    layout.place_at_row_column(&label2b.name, 1, 1);
    scene.add_view_to_parent(label2b, &panel.name);

    let label3a = make_label("info-label3a", "Used memory").position_at(40, 80);
    layout.place_at_row_column(&label3a.name, 2, 0);
    scene.add_view_to_parent(label3a, &panel.name);

    let label3b = make_label("info-label3b", &format!("{:?}", used)).position_at(180, 80);
    layout.place_at_row_column(&label3b.name, 2, 1);
    scene.add_view_to_parent(label3b,&panel.name);

    let label4a = make_label("info-label4a", "Total memory").position_at(40, 100);
    layout.place_at_row_column(&label4a.name, 3, 0);
    scene.add_view_to_parent(label4a,&panel.name);

    let label4b = make_label("info-label4b", &format!("{:?}", free + used)).position_at(180, 100);
    layout.place_at_row_column(&label4b.name, 3, 1);
    scene.add_view_to_parent(label4b,&panel.name);

    let button = make_button(INFO_BUTTON, "done");
    layout.place_at_row_column(&button.name, 4, 1);
    scene.add_view_to_parent(button,&panel.name);

    panel.state = Some(Box::new(layout));

    scene.add_view_to_root(panel);

    scene.hide_view(MAIN_MENU);
    scene.set_focused(INFO_BUTTON);
}
fn show_wifi_panel(scene: &mut Scene) {
    let panel = make_panel(WIFI_PANEL).with_bounds(Bounds::new(20, 20, 320 - 40, 240 - 40));
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
    let mut panel = make_panel(SETTINGS_PANEL)
        .with_bounds(Bounds::new(20, 20, 320 - 60, 240 - 40-40))
        .with_layout(Some(layout_vbox));
    panel.h_flex = Intrinsic;
    panel.v_flex = Intrinsic;
    scene.add_view_to_parent(
        make_label("settings-theme-label", "Theme"),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_toggle_group(&ViewId::new("settings-theme"), vec!["Light", "Dark"], 0)
            .with_flex(Resize, Intrinsic),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_label("settings-font-label", "Font"),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_button(&ViewId::new("settings-font-button"), "Small"),
        &panel.name,
    );

    scene.add_view_to_parent(
        make_button(&ViewId::new("settings-close-button"), "Close"),
        &panel.name,
    );
    scene.add_view_to_root(panel);
}
pub fn make_gui_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let panel = make_panel(&ViewId::new("panel")).with_bounds(Bounds::new(20, 20, 260, 200));
    scene.add_view_to_root(panel);

    let full_screen_bounds = Bounds::new(0, 0, 320, 240);
    let page_view = PageView::new(full_screen_bounds, Page::new());
    scene.add_view_to_root(page_view);
    let main_menu = make_list_view(
        MAIN_MENU,
        vec![
            "Browser".into(),
            "Network".into(),
            "Settings".into(),
            "Info".into(),
            "close".into(),
        ],
        0,
    )
    .position_at(0, 0);

    scene.add_view_to_root(main_menu);
    scene.hide_view(MAIN_MENU);

    scene.add_view_to_root(
        make_list_view(WIFI_MENU, vec!["status", "scan", "close"], 0)
            .position_at(20, 20)
            .with_visible(false)
    );

    let browser_menu = make_list_view(
        BROWSER_MENU,
        vec![
            "Bookmarks",
            "SDCard",
            "Open URL",
            "Back",
            "Forward",
            "close",
        ],
        0,
    )
    .position_at(20, 20)
        .with_visible(false);

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
    scene
}

pub fn update_view_from_keyboard_input(scene: &mut Scene, evt: &TextAction) {
    match evt {
        TextAction::TypedAscii(key ) => {
            if *key == b' ' {
                if scene.is_visible(MAIN_MENU) == false && scene.is_focused(PAGE_VIEW) {
                    scene.show_view(MAIN_MENU);
                    scene.set_focused(MAIN_MENU);
                }
            }
        }
        _ => {}
    };
}
pub fn update_view_from_input(event: &mut GuiEvent, _app: &mut AppState) {
    match &event.event_type {
        InputEvent::Text(TextAction::TypedAscii(key)) => {
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
        InputEvent::Tap(pt) => {
            info!("tapped on point {pt:?}");
        }
        _ => {}
    }
}

pub fn load_page(scene: &mut Scene, page: Page) {
    if let Some(state) = scene.get_view_state::<PageView>(PAGE_VIEW) {
        info!("page got a new page: {:?}", page);
        state.load_page(page);
    }
    scene.mark_dirty_view(PAGE_VIEW);
}
