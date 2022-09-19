use std::time::Duration;

use druid::{kurbo::TranslateScale, Selector, Vec2};
use force_graph::SimulationParameters;

pub const DEFAULT_BORDER_SIZE: f64 = 3.0;
pub const DEFAULT_EDGE_WIDTH: f64 = 3.0;

pub const DEFAULT_SIMULATION_PARAMTERS: SimulationParameters = SimulationParameters {
    force_charge: 7000.0,
    force_spring: 1.9,
    force_max: 280.0,
    node_speed: 7000.0,
    damping_factor: 0.50
};

pub const DEFAULT_TRANSLATE: TranslateScale = TranslateScale::new(
    Vec2::new(0.0, 0.0), 1.0
);

pub const DEFAULT_OFFSET_X: f64 = 0.0;
pub const DEFAULT_OFFSET_Y: f64 = 0.0;

pub const DEFAULT_SCALE: TranslateScale = TranslateScale::new(
    Vec2::new(0.0, 0.0), 1.0
);
pub const DEBUG_SHOW_EVENT_VISUALS: bool = false;

pub const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(200);

pub const TAKE_FOCUS: Selector = Selector::new("take-focus");

pub const TAKEN_FOCUS: Selector = Selector::new("taken-focus");

pub const SUBMIT_CHANGES: Selector = Selector::new("submit-changes");