use core::option::Option::Some;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use gui2::geom::Bounds;
use gui2::{Action, DrawingContext, GuiEvent, Theme, View};

pub fn make_toggle_group<C, F>(name: &str, data: Vec<&str>, selected: usize) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, (data.len() * 80) as i32, 30),
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

fn input_toggle_group<C, F>(_event: &mut GuiEvent<C, F>) -> Option<Action> {
    None
}

fn draw_toggle_group<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    let bounds = view.bounds.clone();
    if let Some(state) = view.get_state::<SelectOneOfState>() {
        let cell_width = bounds.w / (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let fill = if i == state.selected {
                &theme.fg
            } else {
                &theme.bg
            };
            ctx.fill_rect(
                &Bounds::new((i as i32) * cell_width, bounds.y, cell_width, bounds.h),
                fill,
            );
        }
    }
}
