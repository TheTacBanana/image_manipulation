use cgmath::{Vector2, Zero};

// Enum to represent all cursor actions
#[derive(Debug, Clone, Copy)]
pub enum CursorEvent {
    ButtonPressed,
    ButtonReleased,
    StartTouch(u64, Vector2<f32>),
    TouchMove(u64, Vector2<f32>),
    EndTouch(u64),
    Position(Vector2<f32>),
    Scroll(f32),
}

// Stores data about the current input state
#[derive(Debug)]
pub struct InputContext {
    pub last_mouse_pos: Vector2<f32>,
    pub mouse_pressed: bool,
    pub mouse_over_ui: bool,
    pub touches: Vec<Option<Vector2<f32>>>,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            last_mouse_pos: Vector2::zero(),
            mouse_pressed: false,
            mouse_over_ui: false,
            touches: Vec::new(),
        }
    }
}

impl InputContext {
    pub fn start_touch(&mut self, id : u64, pos : Vector2<f32>) {
        let id = id as usize;
        if id >= self.touches.len() {
            self.touches.extend([id].repeat((id - self.touches.len()) + 1).iter().map(|_| None))
        }
        *self.touches.get_mut(id).unwrap() = Some(pos);
    }

    pub fn update_touch(&mut self, id : u64, pos : Vector2<f32>) -> Option<Vector2<f32>> {
        let touch = (self.touches.get_mut(id as usize)?).as_mut()?;
        let delta = pos - *touch;
        *touch = pos;
        Some(delta)
    }

    pub fn end_touch(&mut self, id : u64) {
        self.touches.get_mut(id as usize).unwrap().take();
    }

    pub fn active_touches(&self) -> Vec<Vector2<f32>> {
        self.touches.iter().filter_map(|x| *x).collect::<Vec<_>>()
    }

    pub fn touch_count(&self) -> usize {
        self.touches.iter().filter(|x| x.is_some()).count()
    }
}