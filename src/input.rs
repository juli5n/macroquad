//! Cross-platform mouse, keyboard (and gamepads soon) module.

use crate::math::vec2;
use crate::prelude::screen_height;
use crate::prelude::screen_width;
use crate::Vec2;
use crate::{get_context, get_quad_context};
pub use miniquad::{KeyCode, MouseButton};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchPhase {
    Started,
    Stationary,
    Moved,
    Ended,
    Cancelled,
}

impl From<miniquad::TouchPhase> for TouchPhase {
    fn from(miniquad_phase: miniquad::TouchPhase) -> TouchPhase {
        match miniquad_phase {
            miniquad::TouchPhase::Started => TouchPhase::Started,
            miniquad::TouchPhase::Moved => TouchPhase::Moved,
            miniquad::TouchPhase::Ended => TouchPhase::Ended,
            miniquad::TouchPhase::Cancelled => TouchPhase::Cancelled,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Touch {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: Vec2,
}

/// Constrain mouse to window
pub fn set_cursor_grab(grab: bool) {
    let context = get_context();
    context.cursor_grabbed = grab;
    get_quad_context().set_cursor_grab(grab);
}

/// Set mouse cursor visibility
pub fn show_mouse(shown: bool) {
    get_quad_context().show_mouse(shown);
}

/// Return mouse position in pixels.
pub fn mouse_position() -> (f32, f32) {
    let context = get_context();

    (
        context.mouse_position.x / get_quad_context().dpi_scale(),
        context.mouse_position.y / get_quad_context().dpi_scale(),
    )
}

/// Return mouse position in range [-1; 1].
pub fn mouse_position_local() -> Vec2 {
    let (pixels_x, pixels_y) = mouse_position();

    convert_to_local(Vec2::new(pixels_x, pixels_y))
}

/// Returns the difference between the current mouse position and the mouse position on the previous frame.
pub fn mouse_delta_position() -> Vec2 {
    let context = get_context();

    let current_position = mouse_position_local();
    let last_position = context.last_mouse_position.unwrap_or(current_position);

    // Calculate the delta
    let delta = last_position - current_position;

    // Store the current mouse position for the next frame
    context.last_mouse_position = Some(current_position);

    delta
}

/// This is set to true by default, meaning touches will raise mouse events in addition to raising touch events.
/// If set to false, touches won't affect mouse events.
pub fn is_simulating_mouse_with_touch() -> bool {
    get_context().simulate_mouse_with_touch
}

/// This is set to true by default, meaning touches will raise mouse events in addition to raising touch events.
/// If set to false, touches won't affect mouse events.
pub fn simulate_mouse_with_touch(option: bool) {
    get_context().simulate_mouse_with_touch = option;
}

/// Return touches with positions in pixels.
pub fn touches() -> Vec<Touch> {
    update_mouse_touch_if_necessary();
    get_context().touches.values().cloned().collect()
}

/// Return touches with positions in range [-1; 1].
pub fn touches_local() -> Vec<Touch> {
    update_mouse_touch_if_necessary();

    get_context()
        .touches
        .values()
        .map(|touch| {
            let mut touch = touch.clone();
            touch.position = convert_to_local(touch.position);
            touch
        })
        .collect()
}

fn update_mouse_touch_if_necessary() {
    let context = get_context();

    if context.simulate_touch_with_mouse {
        let mut remove_touch: bool = false;
        if let Some(touch) = context.touches.get_mut(&context.mouse_touch_id) {

            if is_mouse_button_released(MouseButton::Left) {
                touch.phase = TouchPhase::Ended;
            } else if !is_mouse_button_down(MouseButton::Left) {
                remove_touch = true;
            } 
            let mouse_position = mouse_position();
            let mouse_vec = vec2(mouse_position.0, mouse_position.1);

            // phase update
            if touch.phase != TouchPhase::Ended {
                if touch.position != mouse_vec {
                    touch.phase = TouchPhase::Moved;
                } else {
                    touch.phase = TouchPhase::Stationary;
                }
            } 

            touch.position = mouse_vec;
        } else {
            if is_mouse_button_pressed(MouseButton::Left) {
                let mouse_position = mouse_position();
                let mouse_vec = vec2(mouse_position.0, mouse_position.1);
                context.touches.insert(
                    context.mouse_touch_id,
                    Touch {
                        id: context.mouse_touch_id,
                        phase: TouchPhase::Started,
                        position: mouse_vec,
                    },
                );
            }
        }

        if remove_touch {
            context.touches.remove(&context.mouse_touch_id);
        }
    }
}

pub fn mouse_wheel() -> (f32, f32) {
    let context = get_context();

    (context.mouse_wheel.x, context.mouse_wheel.y)
}

/// Detect if the key has been pressed once
pub fn is_key_pressed(key_code: KeyCode) -> bool {
    let context = get_context();

    context.keys_pressed.contains(&key_code)
}

/// Detect if the key is being pressed
pub fn is_key_down(key_code: KeyCode) -> bool {
    let context = get_context();

    context.keys_down.contains(&key_code)
}

/// Detect if the key has been released this frame
pub fn is_key_released(key_code: KeyCode) -> bool {
    let context = get_context();

    context.keys_released.contains(&key_code)
}

/// Return the last pressed char.
/// Each "get_char_pressed" call will consume a character from the input queue.
pub fn get_char_pressed() -> Option<char> {
    let context = get_context();

    context.chars_pressed_queue.pop()
}

pub(crate) fn get_char_pressed_ui() -> Option<char> {
    let context = get_context();

    context.chars_pressed_ui_queue.pop()
}

/// Return the last pressed key.
pub fn get_last_key_pressed() -> Option<KeyCode> {
    let context = get_context();
    // TODO: this will return a random key from keys_pressed HashMap instead of the last one, fix me later
    context.keys_pressed.iter().next().cloned()
}

/// Detect if the button is being pressed
pub fn is_mouse_button_down(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_down.contains(&btn)
}

/// Detect if the button has been pressed once
pub fn is_mouse_button_pressed(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_pressed.contains(&btn)
}

/// Detect if the button has been released this frame
pub fn is_mouse_button_released(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_released.contains(&btn)
}

/// Convert a position in pixels to a position in the range [-1; 1].
fn convert_to_local(pixel_pos: Vec2) -> Vec2 {
    Vec2::new(pixel_pos.x / screen_width(), pixel_pos.y / screen_height()) * 2.0
        - Vec2::new(1.0, 1.0)
}

/// Prevents quit
pub fn prevent_quit() {
    get_context().prevent_quit_event = true;
}

/// Detect if quit has been requested
pub fn is_quit_requested() -> bool {
    get_context().quit_requested
}

/// Functions for advanced input processing.
///
/// Functions in this module should be used by external tools that uses miniquad system, like different UI libraries. User shouldn't use this function.
pub mod utils {
    use crate::{get_context, get_quad_context};

    /// Register input subscriber. Returns subscriber identifier that must be used in `repeat_all_miniquad_input`.
    pub fn register_input_subscriber() -> usize {
        let context = get_context();

        context.input_events.push(vec![]);

        context.input_events.len() - 1
    }

    /// Repeats all events that came since last call of this function with current value of `subscriber`. This function must be called at each frame.
    pub fn repeat_all_miniquad_input<T: miniquad::EventHandler>(t: &mut T, subscriber: usize) {
        let context = get_context();
        let mut ctx = get_quad_context();

        for event in &context.input_events[subscriber] {
            event.repeat(&mut ctx, t);
        }
        context.input_events[subscriber].clear();
    }
}
