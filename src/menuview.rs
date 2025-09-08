use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use gui2::geom::Bounds;
use gui2::{EventType, HAlign, View};
use log::info;

pub struct MenuState {
    pub data:Vec<String>,
    pub selected:i32,
}
const MH:i32 = 20;
pub fn make_menuview<C, F>(name:&str, data:Vec<String>) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds {
            x:0,
            y:0,
            w:100,
            h: (MH*(data.len() as i32)) as i32,
        },
        visible:true,
        children: vec![],
        draw: Some(|view, ctx, theme| {
            ctx.fill_rect(&view.bounds, &theme.bg);
            ctx.stroke_rect(&view.bounds, &theme.fg);
            if let Some(state) = &view.state {
                if let Some(state) = state.downcast_ref::<MenuState>() {
                    for (i,item) in (&state.data).iter().enumerate() {
                        let b = Bounds {
                            x: view.bounds.x,
                            y: view.bounds.y + (i as i32) * MH,
                            w: view.bounds.w,
                            h: 20,
                        };
                        if state.selected == (i as i32) {
                            ctx.fill_rect(&b,&theme.fg);
                            ctx.fill_text(&b,item.as_str(),&theme.bg, &HAlign::Left);
                        }else {
                            ctx.fill_text(&b, item.as_str(), &theme.fg, &HAlign::Left);
                        }
                    }
                }
            }
        }),
        input: Some(|event|{
            match &event.event_type {
                EventType::Tap(pt) => {
                    if let Some(view) = event.scene.get_view_mut(event.target) {
                        let name = view.name.clone();
                        if view.bounds.contains(pt) {
                            let y = pt.y - view.bounds.y;
                            let selected = y/MH;
                            if let Some(state) = &mut view.state {
                                if let Some(state) = state.downcast_mut::<MenuState>() {
                                    if selected >= 0 && selected < state.data.len() as i32 {
                                        state.selected = selected;
                                        event.scene.set_focused(&name);
                                    }
                                }
                            }
                        }
                    }
                }
                EventType::Scroll(dx,dy) => {
                    if let Some(view) = event.scene.get_view_mut(event.target) {
                        if let Some(state) = &mut view.state {
                            if let Some(state) = state.downcast_mut::<MenuState>() {
                                let len = state.data.len() as i32;
                                if *dy > 0 {
                                    state.selected = (state.selected + 1) % len;
                                }
                                if *dy < 0 {
                                    state.selected = (state.selected -1 + len) % len;
                                }
                                event.scene.dirty = true;
                            }
                        }
                    }
                }
                _ => {
                    info!("unknown event type");
                }
            }
        }),
        layout: Some(|scene, name|{
            info!("doing layout on menuview");
            if let Some(parent) = scene.get_view_mut(name) {
                if let Some(state) = &parent.state {
                    if let Some(state) = state.downcast_ref::<MenuState>() {
                        parent.bounds.h = MH * (state.data.len() as i32)
                    }
                }
            };
        }),
        state: Some(Box::new(MenuState{data,selected:0})),
    }
}