use std::time::Duration;

use druid::{kurbo::TranslateScale, Selector, Vec2};
use force_graph::SimulationParameters;

pub const DEFAULT_BORDER_SIZE: f64 = 3.;
pub const DEFAULT_BORDER_RADIUS: f64 = 5.;
pub const DEFAULT_EDGE_WIDTH: f64 = 3.;
pub const LABEL_FONT_SIZE: f64 = 20.;

pub const DEFAULT_SIMULATION_PARAMTERS: SimulationParameters = SimulationParameters {
    force_charge: 8000.,
    force_spring: 1.0,
    force_max: 280.,
    node_speed: 7000.,
    damping_factor: 0.5
};

pub const DEFAULT_TRANSLATE: TranslateScale = TranslateScale::new(
    Vec2::new(0., 0.), 1.
);

pub const DEFAULT_OFFSET_X: f64 = 0.;
pub const DEFAULT_OFFSET_Y: f64 = 0.;

pub const DEFAULT_SCALE: TranslateScale = TranslateScale::new(
    Vec2::new(0., 0.), 1.
);
pub const DEBUG_SHOW_EVENT_VISUALS: bool = false;

pub const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(200);

pub const TAKE_FOCUS: Selector = Selector::new("take-focus");

pub const TAKEN_FOCUS: Selector = Selector::new("taken-focus");

pub const SUBMIT_CHANGES: Selector = Selector::new("submit-changes");

pub const CANCEL_CHANGES: Selector = Selector::new("cancel-changes");

pub const DEFAULT_SAVE_NAME: &str = "NewSheet.vmd";