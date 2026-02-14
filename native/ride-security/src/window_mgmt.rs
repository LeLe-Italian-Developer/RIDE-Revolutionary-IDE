use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
#[derive(Clone)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_maximized: bool,
    pub is_full_screen: bool,
}

#[napi]
pub struct WindowService {
    state: WindowState,
}

#[napi]
impl WindowService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            state: WindowState {
                x: 0,
                y: 0,
                width: 1024,
                height: 768,
                is_maximized: false,
                is_full_screen: false,
            }
        }
    }

    #[napi]
    pub fn get_state(&self) -> WindowState {
        self.state.clone()
    }

    #[napi]
    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }
}
