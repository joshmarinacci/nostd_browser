use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use gui2::geom::Bounds;
use gui2::{find_children, Action, DrawEvent, DrawingContext, GuiEvent, HAlign, Scene, Theme, View};
use hashbrown::HashMap;
use log::info;

pub fn make_overlay_label<C,F>(name:&str, title:&str) -> View<C,F> {
    View {
        name:name.into(),
        title: title.into(),
        bounds: Bounds::new(0,0,100,20),
        visible:true,
        state: None,
        layout: None,
        input: None,
        draw: Some(|v,ctx,theme|{
            ctx.fill_rect(&v.bounds,&theme.fg);
            ctx.fill_text(&v.bounds, &v.title, &theme.bg, &HAlign::Right);
        }),
        draw2: None,
    }
}

struct SelectedState {
    selected: bool
}

impl SelectedState {
    fn new() -> Box<dyn Any> {
        Box::new(SelectedState { selected: false })
    }
}

struct SelectOneOfState {
    items: Vec<String>,
    selected: usize
}

impl SelectOneOfState {
    fn new_with(items: Vec<&str>, selected: usize) -> Box<dyn Any> {
        Box::new(SelectOneOfState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        })
    }
}

fn draw_toggle_button<C, F>(view: &mut View<C, F>, ctx: &mut dyn DrawingContext<C, F>, theme: &Theme<C, F>) {
    let (button_fill, button_color) = if let Some(state) = view.get_state::<SelectedState>() {
        if state.selected {
            (&theme.bg, &theme.fg)
        } else {
            (&theme.bg, &theme.fg)
        }
    } else {
        (&theme.bg, &theme.fg)
    };

    ctx.fill_rect(&view.bounds,button_fill);
    ctx.stroke_rect(&view.bounds,&theme.fg);
    ctx.fill_text(&view.bounds, &view.title, button_color, &HAlign::Center);
}

fn input_toggle_button<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action>{
    if let Some(state)= event.scene.get_view_state::<SelectedState>(event.target) {
        state.selected = !state.selected;
    }
    None
}
pub fn make_toggle_button<C,F>(name:&str, title:&str) -> View<C,F> {
    View {
        name:name.into(),
        title:title.into(),
        bounds: Bounds::new(0,0,80,30),
        visible: true,
        state: Some(SelectedState::new()),
        draw: Some(draw_toggle_button),
        draw2:None,
        layout:None,
        input:Some(input_toggle_button),
    }
}


fn make_toggle_group<C,F>(name:&str, data:Vec<&str>, selected:usize) -> View<C,F> {
    View {
        name:name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, (data.len() * 80) as i32, 30),
        state:Some(SelectOneOfState::new_with(data,selected)),
        draw:Some(draw_toggle_group),
        input:Some(input_toggle_group),
        layout: None,
        draw2: None,
        visible: true,
    }
}

fn input_toggle_group<C,F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
    None
}

fn draw_toggle_group<C, F>(view: &mut View<C, F>, ctx: &mut dyn DrawingContext<C, F>, theme: &Theme<C, F>) {
    let bounds = view.bounds.clone();
    if let Some(state) = view.get_state::<SelectOneOfState>() {
        let cell_width = bounds.w / (state.items.len() as i32);
        for (i,item) in state.items.iter().enumerate() {
            let fill = if i == state.selected {
                &theme.fg
            } else {
                &theme.bg
            };
            ctx.fill_rect(&Bounds::new((i as i32)*cell_width,bounds.y, cell_width, bounds.h),fill);
        }
    }
}


pub fn make_rect_view<C,F>(name:&str) -> View<C, F> {
    View {
        name:name.into(),
        title:name.into(),
        bounds: Bounds::new(0,0,20,20),
        visible:true,
        draw:None,
        draw2:Some(|e|{
            info!("drawing rect")
        }),
        layout:None,
        state: None,
        input:None,
    }
}

struct FormLayoutState {
    pub constraints: HashMap<String, LayoutConstraint>,
    row_count: usize,
    col_count: usize,
    col_width: usize,
    row_height: usize,
}

struct LayoutConstraint {
    col:usize,
    row:usize,
    col_span:usize,
    row_span:usize,
    halign: HAlign,
    // valign: VAlign;
}
pub fn make_form<C,F>(name:&str) -> View<C, F> {
    View {
        name:name.into(),
        title:name.into(),
        bounds:Bounds::new(0,0,100,100),
        input: None,
        state:Some(Box::new(FormLayoutState{
            constraints: HashMap::new(),
            col_count: 2,
            row_count: 2,
            col_width: 100,
            row_height: 30,
        })),
        layout:Some(layout_form),
        draw:None,
        draw2: Some(common_draw_panel),
        visible: true,
    }
}

fn common_draw_panel<C,F>(evt:&mut DrawEvent<C, F>) {
    evt.ctx.fill_rect(&evt.view.bounds, &evt.theme.panel_bg);
    evt.ctx.stroke_rect(&evt.view.bounds,&evt.theme.fg);
}

fn layout_form<C,F>(scene:&mut Scene<C,F>, name:&str) {
    let kids = find_children(scene, name);
    for kid in kids {
        if let Some(state) = scene.get_view_state::<FormLayoutState>(name) {
            let bounds = if let Some(cons) = &state.constraints.get(&kid) {
                let x = (cons.col * state.col_width) as i32;
                let y = (cons.row * state.row_height) as i32;
                let w = (state.col_width) as i32;
                let h = (state.row_height) as i32;
                Bounds::new(x,y,w,h)
            } else {
                Bounds::new(0,0,0,0)
            };
            if let Some(view) = scene.get_view_mut(&kid) {
                view.bounds = bounds;
            }
        }
    }
}

//                     30..126 => {
//                         info!("printable key: {:?}", key);
//                         self.text.push_str(&String::from_utf8_lossy(&[key]))
//                     13 => {
//                         info!("text input return key")
//                     8 => {
//                         info!("backspace");
//                         self.text.pop();
//                     0_u8..=29_u8 | 126_u8..=u8::MAX => {
//                         info!("unprintable key: {:?}", key);
