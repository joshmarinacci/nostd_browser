use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use gui2::geom::Bounds;
use gui2::{Action, EventType, GuiEvent, HAlign, Scene, View};
use log::info;

pub struct MenuState {
    pub data: Vec<String>,
    pub selected: i32,
}
const MH: i32 = 20;
pub fn make_menuview<C, F>(name: &str, data: Vec<&str>) -> View<C, F> {
    let elements = data
        .iter()
        .map(|d| String::from(*d))
        .collect::<Vec<String>>();
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: (MH * (data.len() as i32)) as i32,
        },
        visible: true,
        draw: Some(|view, ctx, theme| {
            let bounds = view.bounds.clone();
            ctx.fill_rect(&view.bounds, &theme.bg);
            ctx.stroke_rect(&view.bounds, &theme.fg);
            if let Some(state) = &view.get_state::<MenuState>() {
                for (i, item) in (&state.data).iter().enumerate() {
                    let b = Bounds {
                        x: bounds.x,
                        y: bounds.y + (i as i32) * MH,
                        w: bounds.w,
                        h: MH-1,
                    };
                    if state.selected == (i as i32) {
                        ctx.fill_rect(&b, &theme.fg);
                        ctx.fill_text(&b, item.as_str(), &theme.bg, &HAlign::Left);
                    } else {
                        ctx.fill_text(&b, item.as_str(), &theme.fg, &HAlign::Left);
                    }
                }
            }
        }),
        draw2: None,
        input: Some(|event| {
            match &event.event_type {
                EventType::Tap(pt) => {
                    event.scene.set_focused(event.target);
                    if let Some(view) = event.scene.get_view_mut(event.target) {
                        let name = view.name.clone();
                        if view.bounds.contains(pt) {
                            let y = pt.y - view.bounds.y;
                            let selected = y / MH;
                            if let Some(state) = view.get_state::<MenuState>() {
                                if selected >= 0 && selected < state.data.len() as i32 {
                                    state.selected = selected;
                                    return Some(Action::Command(state.data[state.selected as usize].clone()));
                                }
                            }
                        }
                    }
                    return Some(Action::Command("select".into()))
                }
                EventType::Scroll(dx, dy) => {
                    if *dy > 0 {
                        scroll_by(event.scene,event.target,1);
                    }
                    if *dy < 0 {
                        scroll_by(event.scene,event.target,-1);
                    }
                }
                EventType::Keyboard(key) => {
                    match *key {
                        b'j' => {
                            scroll_by(event.scene, event.target, 1);
                        }
                        b'k' => {
                            scroll_by(event.scene, event.target, -1);
                        }
                        13 => {
                            info!("enter");
                            if let Some(state) = event.scene.get_view_state::<MenuState>(event.target) {
                                return Some(Action::Command(state.data[state.selected as usize].clone()));
                            }
                        },
                        _ => {
                            info!("other keypress {key}");
                        }
                    }
                }
                EventType::Action() => {
                    if let Some(state) = event.scene.get_view_state::<MenuState>(event.target) {
                        return Some(Action::Command(state.data[state.selected as usize].clone()));
                    }
                }
                _ => {
                    info!("unknown event type");
                }
            }
            None
        }
        ),
        layout: Some(|scene, name| {
            info!("doing layout on menuview");
            if let Some(view) = scene.get_view_mut(name) {
                if let Some(state) = view.get_state::<MenuState>() {
                    view.bounds.h = MH * (state.data.len() as i32)
                }
            };
        }),
        state: Some(Box::new(MenuState {
            data: elements,
            selected: 0,
        })),
    }
}

fn scroll_by<C, F>(scene: &mut Scene<C, F>, name: &str, amt: i32) {
    if let Some(state) = scene.get_view_state::<MenuState>(name) {
        let len = state.data.len() as i32;
        state.selected = (state.selected  + amt + len) % len;
        scene.mark_dirty_view(name);
    }
}