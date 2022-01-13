pub mod light_bulb;
pub mod toggle_switch;
pub mod gate;
pub mod connector;

use core::sync::atomic::AtomicI32;

pub static Z_INDEX: AtomicI32 = AtomicI32::new(1);

pub const GATE_SIZE: f32 = 128.;
pub const GATE_WIDTH: f32 = 64.;
pub const GATE_HEIGHT: f32 = 128.;
