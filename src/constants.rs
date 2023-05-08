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

#![allow(dead_code)]
use std::time::Duration;

use druid::{kurbo::TranslateScale, Selector, Vec2, Point};
use vm_force_graph_rs::SimulationParameters;
use crate::{vminput::ActionPayload, vmgraphclip::VMGraphClip};

pub const CURRENT_SAVE_FILE_VERSION: &str = "0.5.0";
pub const CURRENT_CONFIG_FILE_VERSION: &str = "0.4.0";

pub const DEFAULT_BORDER_WIDTH: f64 = 3.;
pub const DEFAULT_ACTIVE_BORDER_WIDTH: f64 = 8.;
pub const DEFAULT_TARGET_BORDER_WIDTH: f64 = 7.;
pub const DEFAULT_BORDER_RADIUS: f64 = 5.;
pub const DEFAULT_STACK_SPACING: f64 = 12.;
pub const DEFAULT_STACK_PADDING: f64 = 5.;
pub const DEFAULT_STACK_SCALE: f64 = 0.75;
pub const DEFAULT_STACK_X_MARGIN: f64 = 10.;
pub const DEFAULT_STACK_Y_TOP_MARGIN: f64 = 25.;
pub const DEFAULT_STACK_Y_BOTTOM_MARGIN: f64 = 10.;
pub const DEFAULT_MARK_BORDER_WIDTH: f64 = 1.5;
pub const DEFAULT_EDGE_WIDTH: f64 = 3.;
pub const DEFAULT_LABEL_FONT_SIZE: f64 = 20.;
pub const DEFAULT_SEARCH_TERM_FONT_SIZE: f64 = 16.;
pub const DEFAULT_COMPOSE_INDICATOR_FONT_SIZE: f64 = 28.;
pub const DIALOG_LABEL_BUTTON_SPACER: f64 = 40.;

pub const DEFAULT_COMPOSE_INDICATOR_INSET: Vec2 = Vec2::new(20., -20.);

pub const DEFAULT_NODE_MASS: f64 = 10.;
pub const DEFAULT_UPDATE_DELTA: f64 = 0.032;
pub const DEFAULT_SIMULATION_PARAMETERS: SimulationParameters = SimulationParameters {
    force_charge: 1000.,
    force_spring: 4.0,
    force_max: 280.,
    node_speed: 3000.,
    damping_factor: 0.5,
    min_attract_distance: 180.,
};

pub const DEFAULT_MIN_NODE_WIDTH_DATA: f64 = 40.;
pub const DEFUALT_TEXT_CURSOR_WIDTH: f64 = 1.5;

pub const DEFAULT_MASS_INCREASE_AMOUNT: f64 = 2.;
pub const ANIMATION_MOVEMENT_THRESHOLD: f64 = 0.1;

pub const DEFAULT_NODE_DRAG_THRESHOLD: f64 = 4.;

pub const DEFAULT_PAN_AMOUNT_SMALL: f64 = 30.;
pub const DEFAULT_PAN_AMOUNT_LARGE: f64 = 150.;

pub const DEFAULT_NODE_MOVE_AMOUNT_SMALL: f64 = 20.;
pub const DEFAULT_NODE_MOVE_AMOUNT_LARGE: f64 = 100.;

pub const NODE_LABEL_MAX_CONSTRAINTS: (f64, f64) = (200., 115.);
pub const BADGE_BORDER_INFLATION_AMOUNT: f64 = 3.;

pub const ACCEPTED_DELIMITERS: [(&'static str, &'static str); 6] = [
    (r#"""#, r#"""#), 
    (r"'", r"'"), 
    (r"[", r"]"), 
    (r"(", r")"), 
    (r"<",">"), 
    (r"{",r"}")
];

pub const DEFAULT_TAB_LABEL_FONT_SIZE: f64 = 16.;
pub const TAB_BAR_HEIGHT: f64 = DEFAULT_TAB_LABEL_FONT_SIZE*2.;
pub const TAB_BAR_LABEL_PADDING_X: f64 = 6.0;
pub const TAB_BAR_LABEL_PADDING_Y: f64 = 2.0;
pub const TAB_BAR_INDICATOR_HEIGHT: f64 = 4.0;
pub const TAB_DIVIDER_WIDTH: f64 = 1.0;

pub const VEC_ORIGIN: Vec2 = Vec2::new(0., 0.);
pub const DEFAULT_TRANSLATE: TranslateScale = TranslateScale::new(
    Vec2::new(0., 0.), 1.
);

pub const DEFAULT_OFFSET_X: f64 = 0.;
pub const DEFAULT_OFFSET_Y: f64 = 0.;

//Default amount of padding to add when scrolling a given rect into view;
pub const DEFAULT_SCROLL_PADDING: f64 = 20.;
pub const ZOOM_LEVELS: [f64; 13] = [2.25, 2., 1.75, 1.5, 1.25, 1., 0.85, 0.70, 0.5, 0.33, 0.25, 0.16, 0.1];
pub const DEFAULT_ZOOM_INDEX: usize = 5;
pub const DEFAULT_SCALE: TranslateScale = TranslateScale::new(
    Vec2::new(0., 0.),
    ZOOM_LEVELS[DEFAULT_ZOOM_INDEX]
);

pub const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(200);
pub const DEFAULT_COMPOSE_TIMEOUT: Duration = Duration::from_millis(1150);
pub const DEFAULT_BUILD_TIMEOUT: Duration = Duration::from_millis(1150);
pub const DEFAULT_ANIMATION_TIMEOUT: Duration = Duration::from_millis(2000);

pub const SUBMIT_CHANGES: Selector = Selector::new("submit-changes");

pub const EXECUTE_ACTION: Selector<ActionPayload> = Selector::<ActionPayload>::new("execute-action");

pub const SET_REGISTER: Selector<(String, VMGraphClip)> = Selector::<(String, VMGraphClip)>::new("set-register");

pub const GET_REGISTER: Selector<(String, bool, Option<Point>)> = Selector::<(String, bool, Option<Point>)>::new("get-register");

pub const OFFER_REGISTER: Selector<(String, VMGraphClip, bool, Option<Point>)> = Selector::<(String, VMGraphClip, bool, Option<Point>)>::new("offer-register");

pub const CANCEL_CHANGES: Selector = Selector::new("cancel-changes");

pub const TOGGLE_MAIN_MENU: Selector = Selector::new("toggle-main-menu");

pub const SUBMIT_INPUT_ACTION: Selector = Selector::new("execute-input-action");

pub const DIALOG_EXECUTE_ACTIONS: Selector<Vec<ActionPayload>> = Selector::<Vec<ActionPayload>>::new("dialog-execute-action");

pub const REFRESH: Selector = Selector::new("refresh");

pub const DEFAULT_NEW_NODE_LABEL: &str = "New node";
pub const DEFAULT_SAVE_NAME: &str = "NewSheet.vmd";

pub const DEFAULT_CONFIG_DIR_NAME: &str = "vim-mapper";
pub const DEFAULT_CONFIG_FILE_NAME: &str = "vmconfig";

pub const DEFAULT_ROOT_LABEL: &str = "Root";

pub const DEFAULT_YANK_REGISTER: &str = "0";
pub const DEFAULT_CUT_REGISTER: &str = "1";