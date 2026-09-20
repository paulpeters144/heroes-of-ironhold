use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Input {
    Enter,
    Up,
    Down,
    Left,
    Right,
    Jump,
    Attack,
    Shift,
    Skills,
}

#[derive(Default)]
struct InputState {
    current: HashSet<Input>,
    previous: HashSet<Input>,
    now_ms: f32,
    last_fired: HashMap<Input, f32>,
}

thread_local! {
    static INPUT: RefCell<InputState> = RefCell::new(InputState::default());
}

pub fn set(input: Input, down: bool) {
    INPUT.with(|i| {
        let mut s = i.borrow_mut();
        if down {
            s.current.insert(input);
        } else {
            s.current.remove(&input);
        }
    });
}

pub fn end_frame(dt: f32) {
    INPUT.with(|i| {
        let mut s = i.borrow_mut();
        s.previous = s.current.clone();
        s.current.clear();
        s.now_ms += dt * 1000.0;
    });
}

pub fn down(input: Input) -> bool {
    INPUT.with(|i| i.borrow().current.contains(&input))
}

pub fn up(input: Input) -> bool {
    !down(input)
}

pub fn down_once(input: Input) -> bool {
    INPUT.with(|i| {
        let s = i.borrow();
        s.current.contains(&input) && !s.previous.contains(&input)
    })
}

pub fn up_once(input: Input) -> bool {
    INPUT.with(|i| {
        let s = i.borrow();
        !s.current.contains(&input) && s.previous.contains(&input)
    })
}

pub fn down_once_every(input: Input, cooldown_ms: f32) -> bool {
    INPUT.with(|i| {
        let mut s = i.borrow_mut();
        let edge = s.current.contains(&input) && !s.previous.contains(&input);
        if !edge {
            return false;
        }
        let last = s
            .last_fired
            .get(&input)
            .copied()
            .unwrap_or(f32::NEG_INFINITY);
        let now = s.now_ms;
        if now - last >= cooldown_ms {
            s.last_fired.insert(input, now);
            true
        } else {
            false
        }
    })
}

/// Read host key state and push it into the input buffer.
/// Called once per frame from the client's main loop.
pub fn poll_input() {
    use macroquad::prelude::{is_key_down, KeyCode};

    set(Input::Enter, is_key_down(KeyCode::Enter));
    set(Input::Up, is_key_down(KeyCode::Up));
    set(Input::Down, is_key_down(KeyCode::Down));
    set(Input::Left, is_key_down(KeyCode::Left));
    set(Input::Right, is_key_down(KeyCode::Right));
    set(Input::Jump, is_key_down(KeyCode::Space));
    set(Input::Attack, is_key_down(KeyCode::X));
    set(
        Input::Shift,
        is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
    );
    set(Input::Skills, is_key_down(KeyCode::A));
}
