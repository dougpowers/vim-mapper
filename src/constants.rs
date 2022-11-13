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

use druid::{kurbo::TranslateScale, Selector, Vec2};
use vm_force_graph::SimulationParameters;
use crate::vminput::ActionPayload;

pub const CURRENT_SAVE_FILE_VERSION: &str = "0.4.0";
pub const CURRENT_CONFIG_FILE_VERSION: &str = "0.4.0";

pub const DEFAULT_BORDER_WIDTH: f64 = 3.;
pub const DEFAULT_ACTIVE_BORDER_WIDTH: f64 = 8.;
pub const DEFAULT_TARGET_BORDER_WIDTH: f64 = 7.;
pub const DEFAULT_BORDER_RADIUS: f64 = 5.;
pub const DEFAULT_MARK_BORDER_WIDTH: f64 = 1.5;
pub const DEFAULT_EDGE_WIDTH: f64 = 3.;
pub const DEFAULT_LABEL_FONT_SIZE: f64 = 20.;
pub const DEFAULT_COMPOSE_INDICATOR_FONT_SIZE: f64 = 28.;

pub const DEFAULT_COMPOSE_INDICATOR_INSET: Vec2 = Vec2::new(20., -20.);

pub const DEFAULT_NODE_MASS: f64 = 10.;
pub const DEFAULT_UPDATE_DELTA: f64 = 0.032;
pub const DEFAULT_SIMULATION_PARAMTERS: SimulationParameters = SimulationParameters {
    force_charge: 7000.,
    force_spring: 5.0,
    force_max: 280.,
    node_speed: 7000.,
    damping_factor: 0.5,
    // min_attract_distance: 0.,
    min_attract_distance: 180.,
};

pub const DEFAULT_MASS_INCREASE_AMOUNT: f64 = 2.;
pub const ANIMATION_MOVEMENT_THRESHOLD: f64 = 0.06;

pub const DEFAULT_PAN_AMOUNT_SMALL: f64 = 30.;
pub const DEFAULT_PAN_AMOUNT_LARGE: f64 = 150.;

pub const DEFAULT_NODE_MOVE_AMOUNT_SMALL: f64 = 20.;
pub const DEFAULT_NODE_MOVE_AMOUNT_LARGE: f64 = 100.;

pub const NODE_LABEL_MAX_CONSTRAINTS: (f64, f64) = (200., 115.);
pub const BADGE_BORDER_INFLATION_AMOUNT: f64 = 3.;

pub const VEC_ORIGIN: Vec2 = Vec2::new(0., 0.);
pub const DEFAULT_TRANSLATE: TranslateScale = TranslateScale::new(
    Vec2::new(0., 0.), 1.
);

pub const DEFAULT_OFFSET_X: f64 = 0.;
pub const DEFAULT_OFFSET_Y: f64 = 0.;

//Default amount of padding to add when scrolling a given rect into view;
pub const DEFAULT_SCROLL_PADDING: f64 = 20.;

pub const DEFAULT_SCALE: TranslateScale = TranslateScale::new(
    Vec2::new(0., 0.), 1.
);

pub const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(200);
pub const DEFAULT_COMPOSE_TIMEOUT: Duration = Duration::from_millis(1150);
pub const DEFAULT_ANIMATION_TIMEOUT: Duration = Duration::from_millis(1000);

pub const TAKE_FOCUS: Selector = Selector::new("take-focus");

pub const TAKEN_FOCUS: Selector = Selector::new("taken-focus");

pub const SUBMIT_CHANGES: Selector = Selector::new("submit-changes");

pub const EXECUTE_ACTION: Selector<ActionPayload> = Selector::<ActionPayload>::new("execute-action");

pub const CANCEL_CHANGES: Selector = Selector::new("cancel-changes");

pub const TOGGLE_MAIN_MENU: Selector = Selector::new("toggle-main-menu");

pub const DIALOG_EXECUTE_ACTIONS: Selector<Vec<ActionPayload>> = Selector::<Vec<ActionPayload>>::new("dialog-execute-action");

pub const REFRESH: Selector = Selector::new("refresh");

pub const DEFAULT_NEW_NODE_LABEL: &str = "New node";
pub const DEFAULT_SAVE_NAME: &str = "NewSheet.vmd";

pub const DEFAULT_CONFIG_DIR_NAME: &str = "vim-mapper";
pub const DEFAULT_CONFIG_FILE_NAME: &str = "vmconfig";

pub const DEFAULT_ROOT_LABEL: &str = "Root";