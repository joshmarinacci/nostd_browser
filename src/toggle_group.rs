use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::option::Option::Some;
use gui2::geom::Bounds;
use gui2::{Action, DrawingContext, EventType, GuiEvent, HAlign, Theme, View};

pub fn make_toggle_group<C, F>(name: &str, data: Vec<&str>, selected: usize) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, (data.len() * 60) as i32, 30),
        state: Some(SelectOneOfState::new_with(data, selected)),
        draw: Some(draw_toggle_group),
        input: Some(input_toggle_group),
        layout: None,
        draw2: None,
        visible: true,
    }
}

pub struct SelectOneOfState {
    items: Vec<String>,
    selected: usize,
}

impl SelectOneOfState {
    fn new_with(items: Vec<&str>, selected: usize) -> Box<dyn Any> {
        Box::new(SelectOneOfState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        })
    }
}

fn input_toggle_group<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
    match &event.event_type {
        EventType::Tap(pt) => {
            event.scene.mark_dirty_view(event.target);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<SelectOneOfState>() {
                    let cell_width = bounds.w / (state.items.len() as i32);
                    let x = pt.x - bounds.x;
                    let n = x / cell_width as i32;
                    if n >= 0 && n < state.items.len() as i32 {
                        state.selected = n as usize;
                        return Some(Action::Command(
                            state.items[state.selected as usize].clone(),
                        ));
                    }
                }
            }
        }
        _ => {

        }
    }
    None
}

fn draw_toggle_group<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    let bounds = view.bounds.clone();
    ctx.fill_rect(&view.bounds, &theme.bg);
    ctx.stroke_rect(&view.bounds, &theme.fg);
    if let Some(state) = view.get_state::<SelectOneOfState>() {
        let cell_width = bounds.w / (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let (fill, color) = if i == state.selected {
                (&theme.fg, &theme.bg)
            } else {
                (&theme.bg, &theme.fg)
            };
            let bds = Bounds::new(
                bounds.x + (i as i32) * cell_width,
                bounds.y,
                cell_width,
                bounds.h,
            );
            ctx.fill_rect(&bds, fill);
            ctx.stroke_rect(&bds, &theme.fg);
            ctx.fill_text(&bds, item, color,&HAlign::Center);
        }
    }
}
