use alloc::boxed::Box;
use core::any::Any;
use gui2::geom::Bounds;
use gui2::{Action, DrawingContext, GuiEvent, HAlign, Theme, View};

pub fn make_toggle_button<C, F>(name: &str, title: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds::new(0, 0, 80, 30),
        visible: true,
        state: Some(SelectedState::new()),
        draw: Some(draw_toggle_button),
        draw2: None,
        layout: None,
        input: Some(input_toggle_button),
    }
}

pub struct SelectedState {
    selected: bool,
}

impl SelectedState {
    fn new() -> Box<dyn Any> {
        Box::new(SelectedState { selected: false })
    }
}

fn draw_toggle_button<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    let (button_fill, button_color) = if let Some(state) = view.get_state::<SelectedState>() {
        if state.selected {
            (&theme.bg, &theme.fg)
        } else {
            (&theme.bg, &theme.fg)
        }
    } else {
        (&theme.bg, &theme.fg)
    };

    ctx.fill_rect(&view.bounds, button_fill);
    ctx.stroke_rect(&view.bounds, &theme.fg);
    ctx.fill_text(&view.bounds, &view.title, button_color, &HAlign::Center);
}

fn input_toggle_button<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
    if let Some(state) = event.scene.get_view_state::<SelectedState>(event.target) {
        state.selected = !state.selected;
    }
    None
}
