use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Default, Debug)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
