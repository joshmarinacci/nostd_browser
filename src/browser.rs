use alloc::string::ToString;
use log::info;
use crate::common::{NetCommand, TDeckDisplay, NET_COMMANDS};
use alloc::{format, vec};
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::MonoFont;
use nostd_html_parser::blocks::{Block, BlockType};
// use embedded_graphics::mono_font::ascii::{FONT_6X13, FONT_6X13_BOLD, FONT_8X13, FONT_8X13_BOLD, FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::pixelcolor::Rgb565;
use gui2::comps::make_panel;
use gui2::geom::Bounds;
use gui2::{EventType, GuiEvent, Scene};
use crate::menuview::make_menuview;
use crate::page::Page;
use crate::pageview::PageView;

const MAIN_MENU: &'static str = "main";
const FONT_MENU: &'static str = "font";
const THEME_MENU: &'static str = "theme";
pub const PAGE_VIEW: &'static str = "page";

pub async fn handle_action<C, F>(scene: &mut Scene<C, F>, display: &TDeckDisplay) {
    let panel_bounds = Bounds::new(20,20,
                                   (display.bounding_box().size.width-40) as i32,
                                   (display.bounding_box().size.height-40) as i32);
    // scene.info();
    // if scene.is_focused(MAIN_MENU) {
    //     if scene.menu_equals(MAIN_MENU, "Theme") {
    //         scene.show_menu(THEME_MENU);
    //         return;
    //     }
    //     if scene.menu_equals(MAIN_MENU, "Font") {
    //         scene.show_menu(FONT_MENU);
    //         return;
    //     }
    //     if scene.menu_equals(MAIN_MENU, "Browser") {
    //         scene.show_menu("browser");
    //         return;
    //     }
    //     if scene.menu_equals(MAIN_MENU, "Wifi") {
    //         let panel = make_panel("panel1", panel_bounds.clone());
    //         let panel = Panel::new(panel_bounds);
    //         let label1a = Label::new("SSID", Point::new(60, 80));
    //         // let label1b = Label::new(SSID.unwrap_or("----"), Point::new(150, 80));
    //         let label2a = Label::new("PASSWORD", Point::new(60, 100));
    //         // let label2b = Label::new(PASSWORD.unwrap_or("----"), Point::new(150, 100));
    //         let button = Button::new("done", Point::new(160 - 20, 200 - 20));
    //
    //         scene.add("wifi-panel", panel);
    //         scene.add("wifi-label1a", label1a);
    //         // scene.add("wifi-label1b", label1b);
    //         scene.add("wifi-label2a", label2a);
    //         // scene.add("wifi-label2b", label2b);
    //         scene.add("wifi-button", button);
    //         scene.hide(MAIN_MENU);
    //         scene.set_focused("wifi-button");
    //         return;
    //     }
    //     if scene.menu_equals(MAIN_MENU, "Info") {
    //         info!("showing the info panel");
    //         let panel = Panel::new(panel_bounds);
    //         scene.add("info-panel", panel);
    //
    //         let free = esp_alloc::HEAP.free();
    //         let used = esp_alloc::HEAP.used();
    //         scene.add("info-label1", Label::new("Heap", Point::new(120, 50)));
    //         scene.add(
    //             "info-label2a",
    //             Label::new("Free  memory ", Point::new(60, 80)),
    //         );
    //         scene.add(
    //             "info-label2b",
    //             Label::new(&format!("{:?}", free), Point::new(200, 80)),
    //         );
    //         scene.add(
    //             "info-label3a",
    //             Label::new("Used  memory ", Point::new(60, 100)),
    //         );
    //         scene.add(
    //             "info-label3b",
    //             Label::new(&format!("{:?}", used), Point::new(200, 100)),
    //         );
    //         scene.add(
    //             "info-label4a",
    //             Label::new("Total memory", Point::new(60, 120)),
    //         );
    //         scene.add(
    //             "info-label4b",
    //             Label::new(&format!("{:?}", free + used), Point::new(200, 120)),
    //         );
    //
    //         let button = Button::new("done", Point::new(160 - 20, 200 - 20));
    //         scene.add("info-button", button);
    //         scene.hide(MAIN_MENU);
    //         scene.set_focused("info-button");
    //         return;
    //     }
    //     if scene.menu_equals(MAIN_MENU, "close") {
    //         scene.hide(MAIN_MENU);
    //         scene.set_focused(PAGE_VIEW);
    //         return;
    //     }
    //     return;
    // }
    // if scene.is_focused("wifi-button") {
    //     info!("clicked the button");
    //     scene.remove("wifi-panel");
    //     scene.remove("wifi-label1a");
    //     scene.remove("wifi-label1b");
    //     scene.remove("wifi-label2a");
    //     scene.remove("wifi-label2b");
    //     scene.remove("wifi-button");
    //     return;
    // }
    // if scene.is_focused("info-button") {
    //     info!("clicked the info button");
    //     scene.remove("info-panel");
    //     scene.remove("info-label1");
    //     scene.remove("info-label2a");
    //     scene.remove("info-label2b");
    //     scene.remove("info-label3a");
    //     scene.remove("info-label3b");
    //     scene.remove("info-label4a");
    //     scene.remove("info-label4b");
    //     scene.remove("info-button");
    //     return;
    // }
    // if scene.is_focused(THEME_MENU) {
    //     if scene.menu_equals(THEME_MENU, "Dark") {
    //         scene.set_theme(DARK_THEME);
    //         return;
    //     }
    //     if scene.menu_equals(THEME_MENU, "Light") {
    //         scene.set_theme(LIGHT_THEME);
    //         return;
    //     }
    //     if scene.menu_equals(THEME_MENU, "close") {
    //         scene.hide(THEME_MENU);
    //         scene.set_focused(MAIN_MENU);
    //         return;
    //     }
    // }
    // if scene.is_focused(FONT_MENU) {
    //     if scene.menu_equals(FONT_MENU, "Small") {
    //         scene.set_font(FONT_6X13, FONT_6X13_BOLD);
    //     }
    //     if scene.menu_equals(FONT_MENU, "Medium") {
    //         scene.set_font(FONT_8X13, FONT_8X13_BOLD);
    //     }
    //     if scene.menu_equals(FONT_MENU, "Large") {
    //         scene.set_font(FONT_9X15, FONT_9X15_BOLD);
    //     }
    //     // close
    //     if scene.menu_equals(FONT_MENU, "close") {
    //         scene.hide(FONT_MENU);
    //         scene.set_focused(MAIN_MENU);
    //         return;
    //     }
    // }
    // if scene.is_focused("wifi") {
    //     if scene.menu_equals("wifi", "close") {
    //         scene.hide("wifi");
    //         scene.set_focused(MAIN_MENU);
    //         return;
    //     }
    // }
    // if scene.is_focused("browser") {
    //     if scene.menu_equals("browser", "Bookmarks") {
    //         // show the bookmarks
    //         NET_COMMANDS
    //             .send(NetCommand::Load("bookmarks.html".to_string()))
    //             .await;
    //         scene.hide(MAIN_MENU);
    //         scene.hide("browser");
    //         scene.set_focused(PAGE_VIEW);
    //         return;
    //     }
    //     if scene.menu_equals("browser","Open URL") {
    //         let panel = Panel::new(panel_bounds);
    //         let label1a = Label::new("URL", Point::new(40, 40));
    //         let input = TextInput::new("https://apps.josh.earth/", Rectangle::new(Point::new(40, 70), Size::new(240,30)));
    //         let button = Button::new("load", Point::new(160 - 20, 200 - 20));
    //         scene.add("url-panel",panel);
    //         scene.add("url-label",label1a);
    //         scene.add("url-input",input);
    //         scene.add("url-button",button);
    //         // scene.hide(MAIN_MENU);
    //         scene.hide("browser");
    //         scene.set_focused("url-input");
    //         return;
    //     }
    //     if scene.menu_equals("browser","Back") {
    //         scene.hide(MAIN_MENU);
    //         scene.hide("browser");
    //         if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
    //             if let Some(tv) = view.as_any_mut().downcast_mut::<PageView>() {
    //                 tv.prev_page();
    //             }
    //         }
    //         scene.set_focused(PAGE_VIEW);
    //     }
    //     if scene.menu_equals("browser","Forward") {
    //         scene.hide(MAIN_MENU);
    //         scene.hide("browser");
    //         if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
    //             if let Some(tv) = view.as_any_mut().downcast_mut::<PageView>() {
    //                 tv.next_page();
    //             }
    //         }
    //         scene.set_focused(PAGE_VIEW);
    //     }
    //     if scene.menu_equals("browser", "close") {
    //         scene.hide("browser");
    //         scene.set_focused(MAIN_MENU);
    //         return;
    //     }
    // }
    // if scene.is_focused("url-input") {
    //     scene.remove("url-panel");
    //     scene.remove("url-label");
    //     scene.remove("url-button");
    //     scene.hide(MAIN_MENU);
    //     scene.hide("browser");
    //     scene.set_focused(PAGE_VIEW);
    //     if let Some(view) = scene.get_view_mut("url-input") {
    //         if let Some(input) = view.as_any().downcast_ref::<TextInput>() {
    //             let href = &input.text;
    //             NET_COMMANDS
    //                 .send(NetCommand::Load(href.to_string()))
    //                 .await;
    //         }
    //     }
    //     scene.remove("url-input");
    //     return;
    // }
    // if scene.is_focused("url-button") {
    //     info!("clicked the button");
    //     return;
    // }
}

pub fn make_gui_scene() -> Scene<Rgb565, MonoFont<'static>> {
    let mut scene = Scene::new();

    let panel = make_panel("panel", Bounds::new(20,20,260,200));
    scene.add_view_to_root(panel);

    let full_screen_bounds = Bounds::new(0,0,320,240);
    let textview = PageView::new(full_screen_bounds, Page::new());
    scene.add_view_to_root(textview);
    let mut menuview = make_menuview("main",vec![
        "Browser".into(),
        "Network".into(),
        "Settings".into(),
        "Info".into(),
        "close".into(),
    ]);
    menuview.bounds.x = 0;
    menuview.bounds.y = 0;
    menuview.visible = false;
    scene.add_view_to_root(menuview);
    scene.set_focused("menu");

    let mut theme_menu = make_menuview(THEME_MENU, vec!["Light".into(), "Dark".into(), "close".into()]);
    theme_menu.bounds.x = 20;
    theme_menu.bounds.y = 20;
    theme_menu.visible = false;
    scene.add_view_to_root(theme_menu);

    let mut font_menu = make_menuview(FONT_MENU, vec!["Small".into(), "Medium".into(), "Large".into(), "close".into()]);
    font_menu.bounds.x = 20;
    font_menu.bounds.y = 20;
    font_menu.visible = false;
    scene.add_view_to_root(font_menu);

    let mut wifi_menu = make_menuview("wifi", vec!["status".into(), "scan".into(), "close".into()]);
    wifi_menu.bounds.x = 20;
    wifi_menu.bounds.y = 20;
    wifi_menu.visible = false;
    scene.add_view_to_root(wifi_menu);


    let mut browser_menu = make_menuview("browser", vec!["Bookmarks".into(),"SDCard".into(), "Open URL".into(), "Back".into(), "Forward".into(), "close".into()]);
    browser_menu.bounds.x = 20;
    browser_menu.bounds.y = 20;
    browser_menu.visible = false;
    scene.add_view_to_root(browser_menu);

    // set up a fake page
    if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
        if let Some(state) = &mut view.state {
            if let Some(tv) = state.downcast_mut::<PageView>() {
                let mut blocks = vec![];
                blocks.push(Block::new_of_type(BlockType::Header, "Header Text"));
                blocks.push(Block::new_of_type(BlockType::ListItem, "list item one"));
                blocks.push(Block::new_of_type(BlockType::ListItem, "list item two"));
                blocks.push(Block::new_of_type(BlockType::ListItem, "list item three"));
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
    info!("update view from input {:?} {:?}", event.target, event.event_type);
    match event.event_type {
        EventType::Keyboard(key) => {
            if key == b' ' {
                show_and_focus(event, MAIN_MENU);
            } else {
                // scene.mutate_view(PAGE_VIEW, |view| {
                //     view.handle_input(event);
                // });
            }
        }
        _ => {
            // scene.handle_input(event);
        }
    }

    // match event {
    //     GuiEvent::KeyEvent(key_event) => match key_event {
    //         13 => {
    //             handle_action(scene, display).await;
    //         }
    //         _ => {}
    //     },
    //     GuiEvent::ScrollEvent(_, _) => {}
    //     GuiEvent::ClickEvent() => {
    //         info!("clicked the button");
    //         handle_action(scene, display).await;
    //     },
    //     GuiEvent::TouchEvent(pt) => {
    //         info!("touch event {pt}")
    //     }
    // }
}

fn show_and_focus<C, F>(event: &mut GuiEvent<C, F>, name: &str) {
    if let Some(menu) = event.scene.get_view_mut(name) {
        menu.visible = true;
    }
    event.scene.set_focused(name);
    event.scene.dirty = true;

}

