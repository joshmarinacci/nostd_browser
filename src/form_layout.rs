use alloc::boxed::Box;
use alloc::string::String;
use gui2::geom::Bounds;
use gui2::{find_children, DrawEvent, DrawingContext, HAlign, Scene, View};
use hashbrown::HashMap;

struct FormLayoutState {
    pub constraints: HashMap<String, LayoutConstraint>,
    row_count: usize,
    col_count: usize,
    col_width: usize,
    row_height: usize,
}

struct LayoutConstraint {
    col: usize,
    row: usize,
    col_span: usize,
    row_span: usize,
    halign: HAlign,
    // valign: VAlign;
}

pub fn make_form<C, F>(name: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, 100, 100),
        input: None,
        state: Some(Box::new(FormLayoutState {
            constraints: HashMap::new(),
            col_count: 2,
            row_count: 2,
            col_width: 100,
            row_height: 30,
        })),
        layout: Some(layout_form),
        draw: None,
        draw2: Some(common_draw_panel),
        visible: true,
    }
}

fn common_draw_panel<C, F>(evt: &mut DrawEvent<C, F>) {
    evt.ctx.fill_rect(&evt.view.bounds, &evt.theme.panel_bg);
    evt.ctx.stroke_rect(&evt.view.bounds, &evt.theme.fg);
}

fn layout_form<C, F>(scene: &mut Scene<C, F>, name: &str) {
    let kids = find_children(scene, name);
    for kid in kids {
        if let Some(state) = scene.get_view_state::<FormLayoutState>(name) {
            let bounds = if let Some(cons) = &state.constraints.get(&kid) {
                let x = (cons.col * state.col_width) as i32;
                let y = (cons.row * state.row_height) as i32;
                let w = (state.col_width) as i32;
                let h = (state.row_height) as i32;
                Bounds::new(x, y, w, h)
            } else {
                Bounds::new(0, 0, 0, 0)
            };
            if let Some(view) = scene.get_view_mut(&kid) {
                view.bounds = bounds;
            }
        }
    }
}
