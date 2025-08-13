use alloc::string::ToString;
use log::info;
use crate::common::{NetCommand, TDeckDisplay, NET_COMMANDS};
use alloc::{format, vec};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use alloc::boxed::Box;
use nostd_html_parser::blocks::{Block, BlockType};
use embedded_graphics::mono_font::ascii::{FONT_6X13, FONT_8X13, FONT_9X15};
use gui::comps::{Button, Label, MenuView, OverlayLabel, Panel, TextInput};
use gui::{GuiEvent, Scene, DARK_THEME, LIGHT_THEME};
use crate::brickbreaker::GameView;
use crate::page::Page;
use crate::pageview::PageView;

const MAIN_MENU: &'static str = "main";
const FONT_MENU: &'static str = "font";
const THEME_MENU: &'static str = "theme";
pub const PAGE_VIEW: &'static str = "page";

pub async fn handle_action(scene: &mut Scene, display: &TDeckDisplay) {
    let panel_bounds = Rectangle::new(
        Point::new(20, 20),
        Size::new(
            display.bounding_box().size.width - 40,
            display.bounding_box().size.height - 40,
        ),
    );
    // scene.info();
    if scene.is_focused(MAIN_MENU) {
        if scene.menu_equals(MAIN_MENU, "Theme") {
            scene.show_menu(THEME_MENU);
            return;
        }
        if scene.menu_equals(MAIN_MENU, "Font") {
            scene.show_menu(FONT_MENU);
            return;
        }
        if scene.menu_equals(MAIN_MENU, "Browser") {
            scene.show_menu("browser");
            return;
        }
        if scene.menu_equals(MAIN_MENU, "Wifi") {
            let panel = Panel::new(panel_bounds);
            let label1a = Label::new("SSID", Point::new(60, 80));
            // let label1b = Label::new(SSID.unwrap_or("----"), Point::new(150, 80));
            let label2a = Label::new("PASSWORD", Point::new(60, 100));
            // let label2b = Label::new(PASSWORD.unwrap_or("----"), Point::new(150, 100));
            let button = Button::new("done", Point::new(160 - 20, 200 - 20));

            scene.add("wifi-panel", panel);
            scene.add("wifi-label1a", label1a);
            // scene.add("wifi-label1b", label1b);
            scene.add("wifi-label2a", label2a);
            // scene.add("wifi-label2b", label2b);
            scene.add("wifi-button", button);
            scene.hide(MAIN_MENU);
            scene.set_focused("wifi-button");
            return;
        }
        if scene.menu_equals(MAIN_MENU, "Info") {
            info!("showing the info panel");
            let panel = Panel::new(panel_bounds);
            scene.add("info-panel", panel);

            let free = esp_alloc::HEAP.free();
            let used = esp_alloc::HEAP.used();
            scene.add("info-label1", Label::new("Heap", Point::new(120, 50)));
            scene.add(
                "info-label2a",
                Label::new("Free  memory ", Point::new(60, 80)),
            );
            scene.add(
                "info-label2b",
                Label::new(&format!("{:?}", free), Point::new(200, 80)),
            );
            scene.add(
                "info-label3a",
                Label::new("Used  memory ", Point::new(60, 100)),
            );
            scene.add(
                "info-label3b",
                Label::new(&format!("{:?}", used), Point::new(200, 100)),
            );
            scene.add(
                "info-label4a",
                Label::new("Total memory", Point::new(60, 120)),
            );
            scene.add(
                "info-label4b",
                Label::new(&format!("{:?}", free + used), Point::new(200, 120)),
            );

            let button = Button::new("done", Point::new(160 - 20, 200 - 20));
            scene.add("info-button", button);
            scene.hide(MAIN_MENU);
            scene.set_focused("info-button");
            return;
        }
        if scene.menu_equals(MAIN_MENU, "Bricks") {
            scene.add("game", GameView::new());
            scene.hide(MAIN_MENU);
            scene.set_focused("game");
            scene.set_auto_redraw(true);
            scene.hide(PAGE_VIEW);
            return;
        }
        if scene.menu_equals(MAIN_MENU, "close") {
            scene.hide(MAIN_MENU);
            scene.set_focused(PAGE_VIEW);
            return;
        }
        return;
    }
    if scene.is_focused("wifi-button") {
        info!("clicked the button");
        scene.remove("wifi-panel");
        scene.remove("wifi-label1a");
        scene.remove("wifi-label1b");
        scene.remove("wifi-label2a");
        scene.remove("wifi-label2b");
        scene.remove("wifi-button");
        return;
    }
    if scene.is_focused("info-button") {
        info!("clicked the info button");
        scene.remove("info-panel");
        scene.remove("info-label1");
        scene.remove("info-label2a");
        scene.remove("info-label2b");
        scene.remove("info-label3a");
        scene.remove("info-label3b");
        scene.remove("info-label4a");
        scene.remove("info-label4b");
        scene.remove("info-button");
        return;
    }
    if scene.is_focused(THEME_MENU) {
        if scene.menu_equals(THEME_MENU, "Dark") {
            scene.set_theme(DARK_THEME);
            return;
        }
        if scene.menu_equals(THEME_MENU, "Light") {
            scene.set_theme(LIGHT_THEME);
            return;
        }
        if scene.menu_equals(THEME_MENU, "close") {
            scene.hide(THEME_MENU);
            scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if scene.is_focused(FONT_MENU) {
        if scene.menu_equals(FONT_MENU, "Small") {
            scene.set_font(FONT_6X13);
        }
        if scene.menu_equals(FONT_MENU, "Medium") {
            scene.set_font(FONT_8X13);
        }
        if scene.menu_equals(FONT_MENU, "Large") {
            scene.set_font(FONT_9X15);
        }
        // close
        if scene.menu_equals(FONT_MENU, "close") {
            scene.hide(FONT_MENU);
            scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if scene.is_focused("wifi") {
        if scene.menu_equals("wifi", "close") {
            scene.hide("wifi");
            scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if scene.is_focused("browser") {
        if scene.menu_equals("browser", "Bookmarks") {
            // show the bookmarks
            NET_COMMANDS
                .send(NetCommand::Load("bookmarks.html".to_string()))
                .await;
            scene.hide(MAIN_MENU);
            scene.hide("browser");
            scene.set_focused(PAGE_VIEW);
            return;
        }
        if scene.menu_equals("browser","Open URL") {
            let panel = Panel::new(panel_bounds);
            let label1a = Label::new("URL", Point::new(40, 40));
            let input = TextInput::new("https://apps.josh.earth/", Rectangle::new(Point::new(40, 70), Size::new(240,30)));
            let button = Button::new("load", Point::new(160 - 20, 200 - 20));
            scene.add("url-panel",panel);
            scene.add("url-label",label1a);
            scene.add("url-input",input);
            scene.add("url-button",button);
            // scene.hide(MAIN_MENU);
            scene.hide("browser");
            scene.set_focused("url-input");
            return;
        }
        if scene.menu_equals("browser","Back") {
            scene.hide(MAIN_MENU);
            scene.hide("browser");
            if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
                if let Some(tv) = view.as_any_mut().downcast_mut::<PageView>() {
                    tv.prev_page();
                }
            }
            scene.set_focused(PAGE_VIEW);
        }
        if scene.menu_equals("browser","Forward") {
            scene.hide(MAIN_MENU);
            scene.hide("browser");
            if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
                if let Some(tv) = view.as_any_mut().downcast_mut::<PageView>() {
                    tv.next_page();
                }
            }
            scene.set_focused(PAGE_VIEW);
        }
        if scene.menu_equals("browser", "close") {
            scene.hide("browser");
            scene.set_focused(MAIN_MENU);
            return;
        }
    }
    if scene.is_focused("url-input") {
        scene.remove("url-panel");
        scene.remove("url-label");
        scene.remove("url-button");
        scene.hide(MAIN_MENU);
        scene.hide("browser");
        scene.set_focused(PAGE_VIEW);
        if let Some(view) = scene.get_view_mut("url-input") {
            if let Some(input) = view.as_any().downcast_ref::<TextInput>() {
                let href = &input.text;
                NET_COMMANDS
                    .send(NetCommand::Load(href.to_string()))
                    .await;
            }
        }
        scene.remove("url-input");
        return;
    }
    if scene.is_focused("url-button") {
        info!("clicked the button");
        return;
    }
}

pub fn make_gui_scene<'a>() -> Scene {
    let mut scene = Scene::new();
    let full_screen_bounds = Rectangle {
        top_left: Point::new(0, 0),
        size: Size::new(320, 240),
    };
    let textview = PageView::new(full_screen_bounds, Page::new());
    scene.add(PAGE_VIEW, Box::new(textview));
    scene.add(
        MAIN_MENU,
        MenuView::start_hidden(
            vec![
                "Theme",
                "Font",
                "Wifi",
                "Browser",
                "Bricks",
                "Info",
                "close",
            ],
            Point::new(0, 0),
        ),
    );
    scene.add(
        THEME_MENU,
        MenuView::start_hidden(vec!["Light", "Dark", "close"], Point::new(20, 20)),
    );
    scene.add(
        FONT_MENU,
        MenuView::start_hidden(vec!["Small", "Medium", "Large", "close"], Point::new(20, 20)),
    );
    scene.add(
        "wifi",
        MenuView::start_hidden(vec!["status", "scan", "close"], Point::new(20, 20)),
    );
    scene.add(
        "browser",
        MenuView::start_hidden(vec!["Bookmarks", "Open URL", "Back", "Forward","close"], Point::new(20, 20)),
    );

    // set up a fake page
    if let Some(view) = scene.get_view_mut(PAGE_VIEW) {
        if let Some(tv) = view.as_any_mut().downcast_mut::<PageView>() {
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
            tv.load_page(page, 30);
        }
    }

    scene.set_focused(PAGE_VIEW);
    let info_panel_bounds = Rectangle::new(Point::new(120, 210), Size::new(200, 30));
    scene.add("status", OverlayLabel::new("some info", info_panel_bounds));
    scene
}

pub async fn update_view_from_input(event: GuiEvent, scene: &mut Scene, display: &TDeckDisplay) {
    // info!("update view from input {:?}", event);
    if scene.get_focused_view().is_none() {
        scene.set_focused("");
    }
    if let Some(menu) = scene.get_menu(MAIN_MENU) {
        if menu.visible {
            scene.handle_input(event);
        } else {
            match event {
                GuiEvent::KeyEvent(evt) => {
                    if evt == b' ' {
                        scene.show_menu(MAIN_MENU);
                    } else {
                        scene.mutate_view(PAGE_VIEW, |view| {
                            view.handle_input(event);
                        });
                    }
                }
                _ => {
                    scene.handle_input(event);
                }
            }
        }
    }

    match event {
        GuiEvent::KeyEvent(key_event) => match key_event {
            13 => {
                handle_action(scene, display).await;
            }
            _ => {}
        },
        GuiEvent::ScrollEvent(_, _) => {}
        GuiEvent::ClickEvent() => {
            info!("clicked the button");
            handle_action(scene, display).await;
        }
    }
}

