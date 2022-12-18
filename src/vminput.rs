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

use std::{collections::{HashMap}, path::PathBuf};

use druid::{keyboard_types::Key, EventCtx, Modifiers, TimerToken, KeyEvent, RawMods};
use regex::Regex;
use common_macros::hash_map;
use crate::{constants::*, vmsave::VMSaveState, vmdialog::VMDialogParams};

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum KeybindMode {
    Start,
    Dialog,
    Sheet,
    EditBrowse,
    Jump,
    Mark,
    Edit,
    Move,
    SearchedSheet,
    SearchBuild,
    KeybindBuild,
    Global,
}


//The action payload allows regex keybinds to define custom parameters associated with the action.
#[derive(Clone, Debug)]
pub struct ActionPayload {
    pub action: Action,
    pub float: Option<f64>,
    pub index: Option<u16>,
    pub string: Option<String>,
    pub mode: Option<KeybindMode>,
    pub save_state: Option<VMSaveState>,
    pub dialog_params: Option<VMDialogParams>,
    pub path: Option<PathBuf>,
}

impl Default for ActionPayload {
    fn default() -> Self {
        ActionPayload {
            action: Action::NullAction,
            float: None,
            index: None,
            string: None,
            mode: None,
            save_state: None,
            dialog_params: None,
            path: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    NullAction,
    CreateNewSheet,
    OpenExistingSheet,
    SaveSheet,
    SaveSheetAs,
    SaveSheetAsOverwrite,
    QuitWithoutSaveGuard,
    QuitWithSaveGuard,
    SetSaveState,
    CycleNodeForward,
    CycleNodeBackward,
    CreateNewNode,
    CreateNewNodeAndEdit,
    CreateNewExternalNode,
    IncreaseActiveNodeMass,
    DecreaseActiveNodeMass,
    ResetActiveNodeMass,
    ToggleAnchorActiveNode,
    ActivateTargetedNode,
    EditActiveNodeSelectAll,
    EditActiveNodeAppend,
    EditActiveNodeInsert,
    DeleteActiveNode,
    DeleteTargetNode,
    MarkActiveNode,
    JumpToMarkedNode,
    TargetNode,
    ChangeModeWithTimeoutRevert,
    ChangeMode,
    MoveActiveNodeDown,
    MoveActiveNodeUp,
    MoveActiveNodeLeft,
    MoveActiveNodeRight,
    CenterNode,
    CenterActiveNode,
    SearchNodes,
    PanUp,
    PanDown,
    PanLeft,
    PanRight,
    ZoomOut,
    ZoomIn,
    DeleteWordWithWhitespace,
    DeleteWord,
    DeleteToEndOfWord,
    DeleteToNthCharacter,
    DeleteWithNthCharacter,
    ChangeWordWithWhitespace,
    ChangeWord,
    ChangeToEndOfWord,
    ChangeToNthCharacter,
    ChangeWithNthCharacter,
    CursorForward,
    CursorBackward,
    CursorForwardToEndOfWord,
    CursorForwardToBeginningOfWord,
    CursorBackwardToEndOfWord,
    CursorBackwardToBeginningOfWord,
    CursorToNthCharacter,
    ToggleColorScheme,
    ToggleDebug,
    ToggleMenuVisible,
    CreateDialog,
    PrintToLogInfo,
}

#[derive(Clone, PartialEq, Debug)]
pub enum KeybindType {
    Key,
    String,
}

#[derive(Clone, Debug)]
struct Keybind {
    #[allow(dead_code)]
    kb_type: KeybindType,
    regex: Option<Regex>,
    group_actions: Option<HashMap<String, HashMap<String, Vec<Option<ActionPayload>>>>>,
    key: Option<Key>,
    modifiers: Option<Modifiers>,
    action_payloads: Vec<Option<ActionPayload>>,
    mode: KeybindMode,
}

pub struct VMInputManager {
    mode: KeybindMode,
    keybinds: Vec<Keybind>,
    string: String,
    timeout_token: Option<TimerToken>,
    timeout_revert_mode: Option<KeybindMode>,
}

impl Default for VMInputManager {
    fn default() -> Self {
        VMInputManager {
            mode: KeybindMode::Start,
            keybinds: vec![
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("o"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::OpenExistingSheet,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("s"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::SaveSheet,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("S"))),
                    modifiers: Some(Modifiers::CONTROL | Modifiers::SHIFT), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::SaveSheetAs,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeForward,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("N"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeBackward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeForward,
                            ..Default::default()
                        })],
                    mode: KeybindMode::SearchedSheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("N"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeBackward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::SearchedSheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("o"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewNodeAndEdit,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("O"))),
                    modifiers: Some(Modifiers::CONTROL | Modifiers::SHIFT), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewExternalNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("O"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("c"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::EditActiveNodeSelectAll,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("d"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DeleteActiveNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("D"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DeleteTargetNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanUp,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ZoomIn,
                            float: Some(1.25),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("K"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanUp,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanDown,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ZoomOut,
                            float: Some(0.75),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("J"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanDown,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanRight,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("L"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanRight,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanLeft,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("H"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanLeft,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Enter),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ActivateTargetedNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::F12),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleDebug,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::F10),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleColorScheme,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::F11),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleMenuVisible,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::F11),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleMenuVisible,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Start,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::F10),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleColorScheme,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Start,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("m"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeModeWithTimeoutRevert,
                            mode: Some(KeybindMode::Mark),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("'"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeModeWithTimeoutRevert,
                            mode: Some(KeybindMode::Jump),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("/"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                            ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::SearchBuild),
                                ..Default::default()
                            }
                        ),
                        Some(
                            ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(String::from("")),
                                ..Default::default()
                            }
                        )],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("G"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CenterNode,
                            index: Some(0),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("-"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DecreaseActiveNodeMass,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("+"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::IncreaseActiveNodeMass,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("="))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ResetActiveNodeMass,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("@"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleAnchorActiveNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("`"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Move),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("`"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("@"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleAnchorActiveNode,
                            ..Default::default()
                        }),
                        Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeDown,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeUp,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeLeft,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeRight,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("J"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeDown,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("K"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeUp,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("H"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeLeft,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("L"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeRight,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    regex: Some(Regex::new(r"^(?x)
                    g(?P<n>.{1})
                    $
                    ").ok().expect("Keybind regex failed to compile.")),
                    group_actions: Some(hash_map!{
                        String::from("n") => hash_map!{
                            String::from("g") => vec![Some(ActionPayload {
                            action: Action::CenterActiveNode,
                            ..Default::default()
                            })]
                        }
                    }),
                    key: None,
                    modifiers: None, 
                    action_payloads: vec![None],
                    mode: KeybindMode::Sheet,
                },
            ],
            string: String::new(),
            timeout_token: None,
            timeout_revert_mode: None,
        }
    }
}

impl VMInputManager {
    pub fn new() -> VMInputManager {
        VMInputManager {
            mode: KeybindMode::Sheet,
            ..Default::default()
        }
    }

    pub fn accept_key(&mut self, event: KeyEvent, ctx: &mut EventCtx) -> Vec<Option<ActionPayload>> {
        let mut key_event = event.clone();
        key_event.mods.set(Modifiers::NUM_LOCK, false);
        key_event.mods.set(Modifiers::SCROLL_LOCK, false);
        key_event.mods.set(Modifiers::CAPS_LOCK, false);
        //Discard solo modifier key presses
        if key_event.key == Key::Shift || 
            key_event.key == Key::Control ||
            key_event.key == Key::Alt ||
            key_event.key == Key::CapsLock || 
            key_event.key == Key::Meta ||
            key_event.key == Key::ScrollLock ||
            key_event.key == Key::NumLock
        {
            return vec![None];
        } 
        #[cfg(debug_assertions)]
        {
            tracing::debug!("{:?}", key_event);
        }
        match self.mode {
            KeybindMode::Start => {
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode == self.mode || keybind.mode == KeybindMode::Global) {
                        if let Some(mods) = keybind.modifiers {
                            if key_event.mods == mods {
                                return keybind.action_payloads.clone();
                            }
                        } else if key_event.mods == RawMods::None {
                            self.clear_timeout();
                            return keybind.action_payloads.clone();
                        }
                    }
                }
                return vec![None];
            },
            KeybindMode::Dialog => {
                return vec![None];
            },
            KeybindMode::Sheet => {
                self.set_new_timeout(ctx);
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode == self.mode || keybind.mode == KeybindMode::Global) {
                        if let Some(mods) = keybind.modifiers {
                            if key_event.mods == mods {
                                self.clear_timeout();
                                return keybind.action_payloads.clone();
                            }
                        } else if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                            self.clear_timeout();
                            return keybind.action_payloads.clone();
                        }
                    }
                }
                if let Key::Character(character) = key_event.key {
                    if key_event.mods.alt() || key_event.mods.ctrl() {
                        return vec![None];
                    }
                    if character == String::from(" ") {
                        return vec![None];
                    } else {
                        self.set_timeout_revert_mode(Some(self.mode.clone()));
                        // self.set_keybind_mode(KeybindMode::KeybindBuild);
                        self.string += &character;
                        // return vec![None];
                        return vec![Some(
                            ActionPayload {
                                action: Action::ChangeModeWithTimeoutRevert,
                                mode: Some(KeybindMode::KeybindBuild),
                                ..Default::default()
                            }
                        )];
                    }
                }
                if let Key::Escape = key_event.key {
                    self.clear_build();
                    return vec![None];
                }
                return vec![None];
            },
            KeybindMode::Move => {
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode == self.mode || keybind.mode == KeybindMode::Global) {
                        if let Some(mods) = keybind.modifiers {
                            if key_event.mods == mods {
                                return keybind.action_payloads.clone();
                            }
                        } else if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                            return keybind.action_payloads.clone();
                        }
                        self.clear_timeout();
                    }
                }
                if key_event.key == Key::Escape || key_event.key == Key::Enter {
                    return vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                        }
                    )];
                }
                return vec![None];
            }
            KeybindMode::Jump => {
                if let Key::Character(character) = key_event.key {
                    if key_event.mods.alt() || key_event.mods.ctrl() {
                        self.set_new_timeout(ctx);
                        return vec![None];
                    }
                    self.clear_build();
                    self.clear_timeout();
                    return vec![
                        Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                        }),
                        Some(ActionPayload {
                            action: Action::JumpToMarkedNode,
                            string: Some(character),
                            ..Default::default()
                    })];
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    return vec![
                        Some(
                            ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            }
                        )
                    ];
                }
            },
            KeybindMode::Mark => {
                if let Key::Character(character) = key_event.key {
                    if !(key_event.mods == RawMods::None || key_event.mods == RawMods::Shift) {
                        self.set_new_timeout(ctx);
                        return vec![None];
                    }
                    self.clear_build();
                    self.clear_timeout();
                    return vec![
                        Some(
                            ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            }
                        ),
                        Some(
                            ActionPayload {
                                action: Action::MarkActiveNode,
                                string: Some(character),
                                ..Default::default()
                            }
                        )
                    ];
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    return vec![
                        Some(
                            ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            }
                        ),
                    ];
                }
            },
            KeybindMode::KeybindBuild => {
                if let Key::Character(character) = key_event.key {
                    if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                        if character == String::from(" ") {
                            self.clear_build();
                            self.clear_timeout();
                            self.revert_mode();
                            return vec![None];
                        } else {
                            self.set_new_timeout(ctx);
                            self.string += &character;
                            let keybinds = self.keybinds.clone();
                            for keybind in keybinds {
                                if let Some(regex) = &keybind.regex {
                                    if regex.is_match(&self.string) {
                                        let matched = self.string.clone();
                                        self.clear_build();
                                        self.clear_timeout();
                                        self.revert_mode();
                                        return VMInputManager::process_regex_keybind(keybind.clone(), matched);
                                    }
                                }
                            }
                            return vec![None];
                        }
                    } else {
                        self.set_new_timeout(ctx);
                        return vec![None];
                    }
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    self.revert_mode();
                    return vec![None];
                }
            },
            KeybindMode::Edit => {
                return vec![None];
            },
            KeybindMode::EditBrowse => {
                return vec![None];
            },
            KeybindMode::SearchedSheet => {
                self.clear_timeout();
                for keybind in &self.keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode == self.mode || keybind.mode == KeybindMode::Global) {
                        if let Some(mods) = keybind.modifiers {
                            if key_event.mods == mods {
                                return keybind.action_payloads.clone();
                            }
                        } else if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                            return keybind.action_payloads.clone();
                        }
                    }
                }
                if key_event.key == Key::Enter {
                    return vec![
                        Some(
                            ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            }
                        ),
                        Some(
                            ActionPayload {
                                action: Action::ActivateTargetedNode,
                                ..Default::default()
                            }
                        )
                    ];
                }
                if key_event.key == Key::Escape {
                    return vec![Some(ActionPayload {
                        action: Action::ChangeMode,
                        mode: Some(KeybindMode::Sheet),
                        ..Default::default()
                    })];
                }
                return vec![None];
            },
            KeybindMode::SearchBuild => {
                if let Key::Character(character) = key_event.key {
                    if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                        self.string += &character;
                        return vec![Some(ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(self.string[1..].to_string()),
                                ..Default::default()
                            })]
                    } else {
                        self.set_new_timeout(ctx);
                        return vec![None];
                    }
                } else if let Key::Backspace = key_event.key {
                    if self.string == "/".to_string() {
                        self.clear_build();
                        self.clear_timeout();
                        return vec![Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            })];
                    } else {
                        self.string.pop();
                        return vec![Some(ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(self.string[1..].to_string()),
                                ..Default::default()
                            })];
                    }

                } else if let Key::Enter = key_event.key {
                    self.clear_build();
                    self.clear_timeout();
                    return vec![Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::SearchedSheet),
                            ..Default::default()
                        })];
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    return vec![Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                        })];
                }
            },
            KeybindMode::Global => {
                tracing::error!("KeybindMode::Global should never be set!");
                panic!();
            }
        }
    }

    pub fn clear_build(&mut self) {
        self.string = String::from("");
    }

    fn process_regex_keybind(keybind: Keybind, string: String) -> Vec<Option<ActionPayload>> {
        let cap = keybind.clone().regex.unwrap().captures(&string).unwrap();
        for name in keybind.clone().regex.unwrap().capture_names() {
            if let Some(string) = name {
                if let Some(map) = keybind.group_actions.as_ref().unwrap().get(string) {
                    if let Some(payload) = map.get(&cap.name(string).unwrap().as_str().to_string()) {
                        return (*payload).clone();
                    }
                }
            }
        }
        return vec![Some(ActionPayload {
            ..Default::default()
        })]
    }

    pub fn set_new_timeout(&mut self, ctx: &mut EventCtx) {
        self.timeout_token = Some(ctx.request_timer(DEFAULT_COMPOSE_TIMEOUT));
    }

    pub fn clear_timeout(&mut self) {
        self.timeout_token = None;
    }

    pub fn set_timeout_revert_mode(&mut self, mode: Option<KeybindMode>) {
        self.timeout_revert_mode = mode;
    }

    #[allow(dead_code)]
    pub fn clear_timeout_revert_mode(&mut self) {
        self.timeout_revert_mode = None;
    }

    pub fn revert_mode(&mut self) {
        if let Some(mode) = self.timeout_revert_mode.clone() {
            self.set_keybind_mode(mode);
        }
    }

    pub fn timeout(&mut self) {
        self.clear_build();
        if let Some(mode) = &self.timeout_revert_mode {
            self.set_keybind_mode((*mode).clone());
            self.set_timeout_revert_mode(None);
        }
    }

    pub fn get_keybind_mode(&self) -> KeybindMode {
        return self.mode.clone();
    }

    pub fn set_keybind_mode(&mut self, mode: KeybindMode) {
        match mode {
            KeybindMode::Start => {

            }
            KeybindMode::Dialog => {

            }
            KeybindMode::Sheet => {
                self.string = String::from("");
            },
            KeybindMode::KeybindBuild => {

            },
            KeybindMode::Move => {
                self.string = String::from("<move>");
            },
            KeybindMode::EditBrowse => {
                self.string = String::from("<edit>");
            },
            KeybindMode::Jump => {
                self.string = String::from("'");
            },
            KeybindMode::Mark => {
                self.string = String::from("m");
            },
            KeybindMode::Edit => {
                self.string = String::from("<insert>");
            },
            KeybindMode::SearchedSheet => {
                self.string = String::from("<search>");
            }
            KeybindMode::SearchBuild => {
                self.string = String::from("/");
            }
            KeybindMode::Global => {
                tracing::error!("KeybindMode::Global should never be set!");
                panic!()
            }
        }
        self.mode = mode;
    }

    pub fn get_string(&self) -> String {
        return self.string.clone();
    }

    pub fn get_timout_token(&self) -> Option<TimerToken> {
        return self.timeout_token;
    }
}