// Copyright 2022 Doug Powers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::Duration;

use druid::{kurbo::TranslateScale, Selector, Vec2, Color};
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

pub const ACTIVE_BORDER_COLOR: Color = Color::rgb8(125, 125, 255);
pub const TARGET_BORDER_COLOR: Color = Color::rgb8(255, 125, 125);

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