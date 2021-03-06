use color::Color;
use dimensions::Dimensions;
use mouse_state::{
    MouseState,
    MouseButtonState
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UiContext,
};
use vecmath::vec2_add;
use widget::Widget::DropDownList;

/// Tuple / Callback params.
pub type Idx = uint;
pub type Len = uint;

/// Represents the state of the menu.
#[deriving(PartialEq, Clone)]
pub enum State {
    Closed(DrawState),
    Open(DrawState),
}

/// Represents the state of the DropDownList widget.
#[deriving(PartialEq, Clone)]
pub enum DrawState {
    Normal,
    Highlighted(Idx, Len),
    Clicked(Idx, Len),
}

impl DrawState {
    /// Translate the DropDownList's DrawState to the equivalent rectangle::State.
    fn as_rect_state(&self) -> rectangle::State {
        match self {
            &DrawState::Normal => rectangle::State::Normal,
            &DrawState::Highlighted(_, _) => rectangle::State::Highlighted,
            &DrawState::Clicked(_, _) => rectangle::State::Clicked,
        }
    }
}

impl State {
    /// Translate the DropDownList's State to the equivalent rectangle::State.
    fn as_rect_state(&self) -> rectangle::State {
        match self {
            &State::Open(draw_state) | &State::Closed(draw_state) => draw_state.as_rect_state(),
        }
    }
}

widget_fns!(DropDownList, State, DropDownList(State::Closed(DrawState::Normal)))

/// Is the cursor currently over the 
fn is_over(pos: Point,
           mouse_pos: Point,
           dim: Dimensions,
           state: State,
           len: Len) -> Option<Idx> {
    match state {
        State::Closed(_) => {
            match rectangle::is_over(pos, mouse_pos, dim) {
                false => None,
                true => Some(0u),
            }
        },
        State::Open(_) => {
            let total_h = dim[1] * len as f64;
            match rectangle::is_over(pos, mouse_pos, [dim[0], total_h]) {
                false => None,
                true => Some((((mouse_pos[1] - pos[1]) / total_h) * len as f64) as uint),
            }
        },
    }
}

/// Determine and return the new State by comparing the mouse state
/// and position to the previous State.
fn get_new_state(is_over_idx: Option<Idx>,
                 len: Len,
                 state: State,
                 mouse: MouseState) -> State {
    match state {
        State::Closed(draw_state) => {
            match is_over_idx {
                Some(_) => {
                    match (draw_state, mouse.left) {
                        (DrawState::Normal, MouseButtonState::Down) => State::Closed(DrawState::Normal),
                        (DrawState::Normal, MouseButtonState::Up) | (DrawState::Highlighted(_, _), MouseButtonState::Up) => State::Closed(DrawState::Highlighted(0u, len)),
                        (DrawState::Highlighted(_, _), MouseButtonState::Down) => State::Closed(DrawState::Clicked(0u, len)),
                        (DrawState::Clicked(_, _), MouseButtonState::Down) => State::Closed(DrawState::Clicked(0u, len)),
                        (DrawState::Clicked(_, _), MouseButtonState::Up) => State::Open(DrawState::Normal),
                    }
                },
                None => State::Closed(DrawState::Normal),
            }
        },
        State::Open(draw_state) => {
            match is_over_idx {
                Some(idx) => {
                    match (draw_state, mouse.left) {
                        (DrawState::Normal, MouseButtonState::Down) => State::Open(DrawState::Normal),
                        (DrawState::Normal, MouseButtonState::Up) | (DrawState::Highlighted(_, _), MouseButtonState::Up) => State::Open(DrawState::Highlighted(idx, len)),
                        (DrawState::Highlighted(_, _), MouseButtonState::Down) => State::Open(DrawState::Clicked(idx, len)),
                        (DrawState::Clicked(p_idx, _), MouseButtonState::Down) => State::Open(DrawState::Clicked(p_idx, len)),
                        (DrawState::Clicked(_, _), MouseButtonState::Up) => State::Closed(DrawState::Normal),
                    }
                },
                None => {
                    match (draw_state, mouse.left) {
                        (DrawState::Highlighted(p_idx, _), MouseButtonState::Up) => State::Open(DrawState::Highlighted(p_idx, len)),
                        _ => State::Closed(DrawState::Normal),
                    }
                },
            }
        }
    }
}

/// A context on which the builder pattern can be implemented.
pub struct DropDownListContext<'a> {
    uic: &'a mut UiContext,
    ui_id: UIID,
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<|&mut Option<Idx>, Idx, String|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

pub trait DropDownListBuilder<'a> {
    /// A dropdownlist builder method to be implemented by the UiContext.
    fn drop_down_list(&'a mut self, ui_id: UIID, strings: &'a mut Vec<String>,
                      selected: &'a mut Option<Idx>) -> DropDownListContext<'a>;
}

impl<'a> DropDownListBuilder<'a> for UiContext {
    fn drop_down_list(&'a mut self, ui_id: UIID, strings: &'a mut Vec<String>,
                      selected: &'a mut Option<Idx>) -> DropDownListContext<'a> {
        DropDownListContext {
            uic: self,
            ui_id: ui_id,
            strings: strings,
            selected: selected,
            pos: [0.0, 0.0],
            dim: [128.0, 32.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }
}

impl_callable!(DropDownListContext, |&mut Option<Idx>, Idx, String|:'a)
impl_colorable!(DropDownListContext)
impl_frameable!(DropDownListContext)
impl_labelable!(DropDownListContext)
impl_positionable!(DropDownListContext)
impl_shapeable!(DropDownListContext)

impl<'a> ::draw::Drawable for DropDownListContext<'a> {
    fn draw(&mut self, graphics: &mut Gl) {

        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let is_over_idx = is_over(self.pos, mouse.pos, self.dim, state, self.strings.len());
        let new_state = get_new_state(is_over_idx, self.strings.len(), state, mouse);

        let sel = match *self.selected {
            Some(idx) if idx < self.strings.len() => { Some(idx) },
            _ => None,
        };
        let color = self.maybe_color.unwrap_or(self.uic.theme.shape_color);
        let t_size = self.maybe_label_font_size.unwrap_or(self.uic.theme.font_size_medium);
        let t_color = self.maybe_label_color.unwrap_or(self.uic.theme.label_color);

        // Call the `callback` closure if mouse was released
        // on one of the DropDownMenu items.
        match (state, new_state) {
            (State::Open(o_d_state), State::Closed(c_d_state)) => {
                match (o_d_state, c_d_state) {
                    (DrawState::Clicked(idx, _), DrawState::Normal) => {
                        match self.maybe_callback {
                            Some(ref mut callback) => (*callback)(self.selected, idx, (*self.strings)[idx].clone()),
                            None => (),
                        }
                    }, _ => (),
                }
            }, _ => (),
        }

        let frame_w = self.maybe_frame.unwrap_or(self.uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(self.uic.theme.frame_color))),
            false => None,
        };

        match new_state {

            State::Closed(_) => {
                let rect_state = new_state.as_rect_state();
                let text = match sel {
                    Some(idx) => (*self.strings)[idx][],
                    None => match self.maybe_label {
                        Some(text) => text,
                        None => (*self.strings)[0][],
                    },
                };
                rectangle::draw_with_centered_label(
                    self.uic.win_w, self.uic.win_h, graphics, self.uic, rect_state,
                    self.pos, self.dim, maybe_frame, color,
                    text, t_size, t_color
                )
            },

            State::Open(draw_state) => {
                for (i, string) in self.strings.iter().enumerate() {
                    let rect_state = match sel {
                        None => {
                            match draw_state {
                                DrawState::Normal => rectangle::State::Normal,
                                DrawState::Highlighted(idx, _) => {
                                    if i == idx { rectangle::State::Highlighted }
                                    else { rectangle::State::Normal }
                                },
                                DrawState::Clicked(idx, _) => {
                                    if i == idx { rectangle::State::Clicked }
                                    else { rectangle::State::Normal }
                                },
                            }
                        },
                        Some(sel_idx) => {
                            if sel_idx == i { rectangle::State::Clicked }
                            else {
                                match draw_state {
                                    DrawState::Normal => rectangle::State::Normal,
                                    DrawState::Highlighted(idx, _) => {
                                        if i == idx { rectangle::State::Highlighted }
                                        else { rectangle::State::Normal }
                                    },
                                    DrawState::Clicked(idx, _) => {
                                        if i == idx { rectangle::State::Clicked }
                                        else { rectangle::State::Normal }
                                    },
                                }
                            }
                        },
                    };
                    let idx_y = self.dim[1] * i as f64 - i as f64 * frame_w;
                    let idx_pos = vec2_add(self.pos, [0.0, idx_y]);
                    rectangle::draw_with_centered_label(
                        self.uic.win_w, self.uic.win_h, graphics, self.uic, rect_state, idx_pos,
                        self.dim, maybe_frame, color, string.as_slice(), 
                        t_size, t_color
                    )
                }
            },

        }

        set_state(self.uic, self.ui_id, new_state, self.pos, self.dim);

    }
}

