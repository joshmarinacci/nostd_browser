use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use gui2::geom::Bounds;
use gui2::{Action, DrawEvent, EventType, TextStyle};
use gui2::scene::Scene;
use gui2::view::View;
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
            h: MH * (data.len() as i32),
        },
        visible: true,
        draw: Some(|e: &mut DrawEvent<C, F>| {
            let bounds = e.view.bounds.clone();
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
            if let Some(state) = &e.view.get_state::<MenuState>() {
                for (i, item) in (&state.data).iter().enumerate() {
                    let b = Bounds {
                        x: bounds.x,
                        y: bounds.y + (i as i32) * MH,
                        w: bounds.w,
                        h: MH - 1,
                    };
                    if state.selected == (i as i32) {
                        e.ctx.fill_rect(&b, &e.theme.fg);
                        let style = TextStyle::new(&e.theme.font, &e.theme.bg);
                        e.ctx.fill_text(&b, item.as_str(), &style);
                    } else {
                        let style = TextStyle::new(&e.theme.font, &e.theme.fg);
                        e.ctx.fill_text(&b, item.as_str(), &style);
                    }
                }
            }
        }),
        input: Some(|event| {
            match &event.event_type {
                EventType::Tap(pt) => {
                    event.scene.set_focused(event.target);
                    if let Some(view) = event.scene.get_view_mut(event.target) {
                        // let name = view.name.clone();
                        if view.bounds.contains(pt) {
                            let y = pt.y - view.bounds.y;
                            let selected = y / MH;
                            if let Some(state) = view.get_state::<MenuState>() {
                                if selected >= 0 && selected < state.data.len() as i32 {
                                    state.selected = selected;
                                    return Some(Action::Command(
                                        state.data[state.selected as usize].clone(),
                                    ));
                                }
                            }
                        }
                    }
                    return Some(Action::Command("select".into()));
                }
                EventType::Scroll(_dx, dy) => {
                    if *dy > 0 {
                        scroll_by(event.scene, event.target, 1);
                    }
                    if *dy < 0 {
                        scroll_by(event.scene, event.target, -1);
                    }
                }
                EventType::Keyboard(key) => match *key {
                    b'j' => {
                        scroll_by(event.scene, event.target, 1);
                    }
                    b'k' => {
                        scroll_by(event.scene, event.target, -1);
                    }
                    13 => {
                        info!("enter");
                        if let Some(state) = event.scene.get_view_state::<MenuState>(event.target) {
                            return Some(Action::Command(
                                state.data[state.selected as usize].clone(),
                            ));
                        }
                    }
                    _ => {
                        info!("other keypress {key}");
                    }
                },
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
        }),
        layout: Some(|event| {
            info!("doing layout on menuview");
            if let Some(view) = event.scene.get_view_mut(event.target) {
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
        state.selected = (state.selected + amt + len) % len;
        scene.mark_dirty_view(name);
    }
}
