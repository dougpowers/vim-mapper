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

use std::{path::PathBuf};

use druid::{keyboard_types::Key, EventCtx, Modifiers, TimerToken, KeyEvent, RawMods, Data, text::EditableText};
use unicode_segmentation::UnicodeSegmentation;
use crate::{constants::*, vmsave::VMSaveState, vmdialog::VMDialogParams, vmtextinput::VMTextInput};

#[allow(dead_code)]
#[derive(Data, Clone, Copy, PartialEq, Debug)]
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
    SnipActiveNode,
    DeleteNodeTree,
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
    InsertCharacter,
    Highlight,
    ExecuteTextAction,
    Delete,
    DeleteBackspace,
    DeleteForward,
    DeleteWord,
    DeleteToNthCharacter,
    DeleteWithNthCharacter,
    ChangeWordWithWhitespace,
    Change,
    ChangeWord,
    ChangeToNthCharacter,
    ChangeWithNthCharacter,
    CursorForward,
    CursorBackward,
    CursorForwardToEndOfWord,
    CursorForwardToBeginningOfWord,
    CursorBackwardToEndOfWord,
    CursorBackwardToBeginningOfWord,
    CursorToNthCharacter,
    SetCursortStyleBlock,
    SetCursortStyleLine,
    SetCursortStyleNone,
    AcceptNodeText,
    ToggleColorScheme,
    ToggleDebug,
    ToggleMenuVisible,
    CreateDialog,
    PrintToLogInfo,
    CreateNewTab,
    OpenNewTabInput,
    DeleteTab,
    OpenDeleteTabPrompt,
    RenameTab,
    OpenRenameTabInput,
    GoToNextTab,
    GoToPreviousTab,
    GoToTab,
}

#[allow(dead_code)]
#[derive(Data, Clone, PartialEq, Debug)]
pub enum TextObj {
    InnerWord,
    OuterWord,
    Inner(String),
    Outer(String),
    Sentence,
}


#[allow(dead_code)]
#[derive(Data, Clone, PartialEq, Debug)]
pub enum TextMotion {
    ForwardWord,
    BackwardWord,
    ForwardToN((usize, String)),
    BackwardToN((usize, String)),
    ForwardWithN((usize, String)),
    BackwardWithN((usize, String)),
}

#[allow(dead_code)]
#[derive(Data, Clone, Copy, PartialEq, Debug)]
pub enum TextOperation {
    DeleteText,
    ChangeText,
}

#[allow(dead_code)]
#[derive(Data, Clone, PartialEq, Debug)]
pub struct TextAction {
    pub (crate) operation: TextOperation,
    pub (crate) count: Option<usize>,
    pub (crate) text_obj: Option<TextObj>,
    pub (crate) text_motion: Option<TextMotion>,
}

#[allow(dead_code)]
#[derive(Data, Clone, Copy, PartialEq, Debug)]
pub enum KeybindMode {
    Start,
    Dialog,
    Sheet,
    EditBrowse,
    EditVisual,
    Edit,
    Jump,
    Mark,
    Move,
    SearchedSheet,
    SearchEnter,
    Global,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeybindType {
    Key,
    String,
}

//The action payload allows regex keybinds to define custom parameters associated with the action.
#[derive(Clone, Debug)]
pub struct ActionPayload {
    pub action: Action,
    pub float: Option<f64>,
    pub index: Option<u32>,
    pub tab_index: Option<usize>,
    pub text_action: Option<TextAction>,
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
            tab_index: None,
            text_action: None,
            string: None,
            mode: None,
            save_state: None,
            dialog_params: None,
            path: None,
        }
    }
}


#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Keybind {
    kb_type: KeybindType,
    key: Option<Key>,
    string: Option<String>,
    accepts_count: bool,
    modifiers: Option<Modifiers>,
    action_payloads: Vec<Option<ActionPayload>>,
    operation: Option<TextOperation>,
    motion: Option<TextMotion>,
    obj: Option<TextObj>,
    mode: KeybindMode,
}

pub enum TextTarget {
    TextObj,
    TextMotion,
}

impl Default for Keybind {
    fn default() -> Self {
        Keybind {
            kb_type: KeybindType::Key,
            key: None,
            string: None,
            accepts_count: false,
            modifiers: None,
            action_payloads: vec![None],
            operation: None,
            motion: None,
            obj: None,
            mode: KeybindMode::Global,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Data)]
pub enum BuildState {
    AwaitOuterCount,
    AwaitOperator,
    AwaitInnerCount,
    AwaitTarget,
}

pub struct VMInputManager {
    mode: KeybindMode,
    build_state: BuildState,
    pub(crate) text_input: VMTextInput,
    keybinds: Vec<Keybind>,
    input_string: String,
    outer_count: String,
    operator: Option<TextOperation>,
    inner_count: String,
    target_string: String,
    text_obj: Option<TextObj>,
    text_motion: Option<TextMotion>,
    mode_prompt: String,
    timeout_build_token: Option<TimerToken>,
    timeout_revert_token: Option<TimerToken>,
    timeout_revert_mode: Option<KeybindMode>,
    string_keybind_cache: Vec<Keybind>,
}

impl Default for VMInputManager {
    fn default() -> Self {
        VMInputManager {
            mode: KeybindMode::Start,
            build_state: BuildState::AwaitOuterCount,
            text_input: VMTextInput::new(),
            mode_prompt: String::new(),
            input_string: String::new(),
            outer_count: String::new(),
            operator: None,
            inner_count: String::new(),
            target_string: String::new(),
            text_obj: None,
            text_motion: None,
            timeout_build_token: None,
            timeout_revert_token: None,
            timeout_revert_mode: None,
            string_keybind_cache: vec![],
            keybinds: vec![
                Keybind { 
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
                    ..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("o"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::OpenExistingSheet,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
                    ..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("s"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::SaveSheet,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("S"))),
                    modifiers: Some(Modifiers::CONTROL | Modifiers::SHIFT), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::SaveSheetAs,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Global,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeForward,
                            ..Default::default()
                        })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("N"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeBackward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeForward,
                            ..Default::default()
                        })],
                    mode: KeybindMode::SearchedSheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("N"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CycleNodeBackward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::SearchedSheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("o"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewNodeAndEdit,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("O"))),
                    modifiers: Some(Modifiers::CONTROL | Modifiers::SHIFT), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewExternalNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("O"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CreateNewNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("c"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::EditBrowse),
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("v"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::EditVisual),
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("a"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::EditActiveNodeAppend,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("i"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::EditActiveNodeInsert,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("a"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Edit),
                            ..Default::default()
                    }),
                    Some(
                        ActionPayload {
                            action: Action::CursorForward,
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("i"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Edit),
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("x"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DeleteForward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CursorForward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CursorBackward,
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("w"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CursorForwardToBeginningOfWord,
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("e"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CursorForwardToEndOfWord,
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("b"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CursorBackwardToBeginningOfWord,
                            ..Default::default()
                    })],
                    mode: KeybindMode::EditBrowse,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("d"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DeleteActiveNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("x"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::SnipActiveNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("D"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DeleteTargetNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanUp,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ZoomIn,
                            float: Some(1.25),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("K"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanUp,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanDown,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ZoomOut,
                            float: Some(0.75),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("J"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanDown,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanRight,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("L"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanRight,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanLeft,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("H"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PanLeft,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Enter),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ActivateTargetedNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::F12),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleDebug,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::F10),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleColorScheme,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::F11),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleMenuVisible,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::F11),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleMenuVisible,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Start,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::F10),
                    modifiers: Some(Modifiers::ALT),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleColorScheme,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Start,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("m"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeModeWithTimeoutRevert,
                            mode: Some(KeybindMode::Mark),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("'"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeModeWithTimeoutRevert,
                            mode: Some(KeybindMode::Jump),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("/"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                            ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::SearchEnter),
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
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("G"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CenterNode,
                            index: Some(0),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("-"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::DecreaseActiveNodeMass,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("+"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::IncreaseActiveNodeMass,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("="))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ResetActiveNodeMass,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("@"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ToggleAnchorActiveNode,
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("`"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Move),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("`"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
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
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeDown,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeUp,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeLeft,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeRight,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("J"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeDown,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("K"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeUp,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("H"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeLeft,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("L"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeRight,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    })],
                    mode: KeybindMode::Move,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Tab),
                    modifiers: Some(Modifiers::CONTROL),
                    action_payloads: vec![Some(ActionPayload {
                        action: Action::GoToNextTab,
                        ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Tab),
                    modifiers: Some(Modifiers::CONTROL | Modifiers::SHIFT),
                    action_payloads: vec![Some(ActionPayload {
                        action: Action::GoToPreviousTab,
                        ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character(String::from("t"))),
                    modifiers: Some(Modifiers::CONTROL),
                    action_payloads: vec![Some(ActionPayload {
                        action: Action::CreateNewTab,
                        ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character(String::from("T"))),
                    modifiers: Some(Modifiers::CONTROL | Modifiers::SHIFT),
                    action_payloads: vec![Some(ActionPayload {
                        action: Action::OpenNewTabInput,
                        ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character(String::from("r"))),
                    modifiers: Some(Modifiers::CONTROL),
                    action_payloads: vec![Some(ActionPayload {
                        action: Action::OpenRenameTabInput,
                        ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character(String::from("w"))),
                    modifiers: Some(Modifiers::CONTROL),
                    action_payloads: vec![Some(ActionPayload {
                        action: Action::OpenDeleteTabPrompt,
                        ..Default::default()
                    })],
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("c".to_string()),
                    operation: Some(TextOperation::ChangeText),
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("iw".to_string()),
                    obj: Some(TextObj::InnerWord),
                    ..Default::default()
                }
            ],
        }
    }
}

impl VMInputManager {
    pub fn new() -> VMInputManager {
        let mut vim = VMInputManager {
            mode: KeybindMode::Sheet,
            ..Default::default()
        };

        vim.string_keybind_cache = vim.keybinds.clone().into_iter().filter(|v| {
            v.kb_type == KeybindType::String
        }).collect::<Vec<Keybind>>();

        return vim;
    }

    pub fn validate_keybinds() {
        tracing::debug!("Validating keybinds");
        for i in 0..VMInputManager::default().keybinds.len() {
            for j in i+1..VMInputManager::default().keybinds.len() {
                let keybind = &VMInputManager::default().keybinds[i];
                let check = &VMInputManager::default().keybinds[j];
                if keybind.kb_type == KeybindType::Key &&
                keybind.key == check.key && 
                keybind.modifiers == check.modifiers && 
                keybind.mode == check.mode
                {
                    tracing::warn!("Key-type keybind {:?}::{:?}({:?}) at index {:?} has a duplicate at index {:?}",
                        keybind.mode,
                        keybind.key,
                        keybind.modifiers,
                        i,
                        j,
                    )
                }
            }
        }
    }

    fn build_keybind_string(&mut self, string: String) -> Option<Result<Vec<Option<ActionPayload>>, ()>> {
        match self.build_state {
            BuildState::AwaitOuterCount => {
                if string.contains(char::is_numeric) {
                    self.outer_count += &string;
                    return None;
                } else {
                    self.build_state = BuildState::AwaitOperator;
                    return self.validate_keybind_string(string);
                }
            },
            BuildState::AwaitOperator => {
                if string.contains(char::is_alphabetic) {
                    return self.validate_keybind_string(string);
                } else {
                    return Some(Err(()))
                }
            },
            BuildState::AwaitInnerCount => {
                if string.contains(char::is_numeric) {
                    self.outer_count += &string;
                    return None;
                } else {
                    self.build_state = BuildState::AwaitTarget;
                    return self.validate_keybind_string(string);
                }
            },
            BuildState::AwaitTarget => {
                if string.contains(char::is_alphabetic) {
                    return self.validate_keybind_string(string);
                } else {
                    return Some(Err(()))
                }
            },
        }
    }

    fn validate_keybind_string(&mut self, string: String) -> Option<Result<Vec<Option<ActionPayload>>, ()>> {
        if self.build_state == BuildState::AwaitOperator {
            if string.chars().all(char::is_alphabetic) {
                for keybind in &self.string_keybind_cache {
                    if let Some(k_string) = keybind.string.clone() {
                        if let Some(operation) = keybind.operation {
                            if k_string.slice(0..k_string.next_grapheme_offset(0).unwrap()).unwrap() == string {
                                if k_string == string {
                                    tracing::debug!("matched {} with {}", string, k_string);
                                    self.operator = Some(operation);
                                    self.build_state = BuildState::AwaitInnerCount;
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
        } else if self.build_state == BuildState::AwaitTarget {
            if string.chars().all(char::is_alphabetic) {
                self.target_string += &string;
                for keybind in &self.string_keybind_cache {
                    if let Some(k_string) = keybind.string.clone() {
                        let k_graphs = k_string.graphemes(true).collect::<Vec<&str>>();
                        let s_graphs = self.target_string.graphemes(true).collect::<Vec<&str>>();
                        let mut partial = false;
                        for s_i in 0..s_graphs.len() {
                            if Some(s_graphs[s_i]) == k_graphs.get(s_i).copied() {
                                partial = true;
                                if s_i == k_graphs.len()-1 {
                                    tracing::debug!("full match {} with {}", self.target_string, k_string);
                                    if let Some(motion) = keybind.motion.clone() {
                                        self.text_motion = Some(motion)
                                    } else if let Some(object) = keybind.obj.clone() {
                                        self.text_obj = Some(object);
                                    }
                                    return Some(Ok(self.build_payload()));
                                }
                            }
                        }
                        if partial {
                            tracing::debug!("partial match {} with {}", self.target_string, k_string);
                            return None;
                        }
                    }
                }
            }
        }
        return Some(Err(()));
    }

    fn build_payload(&mut self) -> Vec<Option<ActionPayload>> {
        tracing::debug!("Build payload for {:?} {:?} {:?} {:?} {:?}",
            self.outer_count,
            self.operator,
            self.inner_count,
            self.text_obj,
            self.text_motion
        );
        let mut payload = ActionPayload {
            ..Default::default()
        };

        if let Some(operator) = self.operator {
            match operator {
                TextOperation::DeleteText => {
                    payload.action = Action::ExecuteTextAction;

                    let mut text_action = TextAction {
                        operation: TextOperation::DeleteText,
                        count: None,
                        text_motion: None,
                        text_obj: None,
                    };

                    if let Some(motion) = &self.text_motion {
                        text_action.text_motion = Some(motion.clone());
                    } else if let Some(obj) = &self.text_obj {
                        text_action.text_obj = Some(obj.clone());
                    }

                    if self.inner_count.len() > 0 {
                        text_action.count = Some(self.inner_count.parse::<usize>().unwrap());
                    }

                    payload.text_action = Some(text_action);
                    self.clear_build();
                    return vec![Some(payload)];
                },
                TextOperation::ChangeText => {
                    payload.action = Action::ExecuteTextAction;

                    let mut text_action = TextAction {
                        operation: TextOperation::ChangeText,
                        count: None,
                        text_motion: None,
                        text_obj: None,
                    };

                    if let Some(motion) = &self.text_motion {
                        text_action.text_motion = Some(motion.clone());
                    } else if let Some(obj) = &self.text_obj {
                        text_action.text_obj = Some(obj.clone());
                    }

                    if self.inner_count.len() > 0 {
                        text_action.count = Some(self.inner_count.parse::<usize>().unwrap());
                    }

                    payload.text_action = Some(text_action);
                    self.clear_build();
                    return vec![Some(payload)];
                },
            }
        } 
        return vec![None];
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
        if key_event.is_composing {
            return vec![None];
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
                            self.clear_revert_timeout();
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
                self.set_new_revert_timeout(ctx);
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode == self.mode || keybind.mode == KeybindMode::Global) {
                        if let Some(mods) = keybind.modifiers {
                            if key_event.mods == mods {
                                self.clear_revert_timeout();
                                return keybind.action_payloads.clone();
                            }
                        } else if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                            self.clear_revert_timeout();
                            return keybind.action_payloads.clone();
                        }
                    }
                }
                if let Key::Character(character) = key_event.key {
                    if character == String::from(" ") {
                        return vec![None];
                    } else {
                        self.set_timeout_revert_mode(Some(self.mode.clone()));
                        self.input_string += &character;
                        if let Some(Ok(payloads)) = self.build_keybind_string(character) {
                            return payloads;
                        } else {
                            return vec![None];
                        }
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
                        self.clear_revert_timeout();
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
                        self.set_new_revert_timeout(ctx);
                        return vec![None];
                    }
                    self.clear_build();
                    self.clear_revert_timeout();
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
                    self.clear_revert_timeout();
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
                        self.set_new_revert_timeout(ctx);
                        return vec![None];
                    }
                    self.clear_build();
                    self.clear_revert_timeout();
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
                    self.clear_revert_timeout();
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
            KeybindMode::Edit => {
                match key_event.key {
                    Key::Character(character) => {
                        return vec![Some(ActionPayload {
                            action: Action::InsertCharacter,
                            string: Some(character),
                            ..Default::default() 
                        })]
                    },
                    Key::Backspace => {
                        return vec![Some(ActionPayload {
                            action: Action::DeleteBackspace,
                            ..Default::default()
                        })]
                    },
                    Key::Delete => {
                        return vec![Some(ActionPayload {
                            action: Action::DeleteForward,
                            ..Default::default()
                        })]
                    },
                    Key::Enter => {
                        return vec![Some(ActionPayload {
                            action: Action::AcceptNodeText,
                            ..Default::default() 
                        }), Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                        })]
                    },
                    Key::Escape => {
                        return vec![Some(ActionPayload {
                            action: Action::AcceptNodeText,
                            ..Default::default()    
                            }),
                            Some(ActionPayload {
                            action: Action::CursorBackward,
                            ..Default::default()
                            }) ,
                            Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::EditBrowse),
                            ..Default::default()
                        })]
                    },
                    Key::ArrowRight => {
                        return vec![Some(ActionPayload {
                            action: Action::CursorForward,
                            ..Default::default()
                        })]
                    },
                    Key::ArrowLeft => {
                        return vec![Some(ActionPayload {
                            action: Action::CursorBackward,
                            ..Default::default()
                        })]
                    },
                    _ => {
                        return vec![None];
                    }
                }
            },
            KeybindMode::EditBrowse => {
                match &key_event.key {
                    Key::Character(character) => {
                        if *character == String::from(" ") {
                            return vec![None];
                        } else {
                            self.set_new_build_timeout(ctx);
                            self.input_string += &character;
                            let ret = self.build_keybind_string(character.clone());
                            tracing::debug!("{:?}", ret);
                            if let Some(Ok(payloads)) = ret {
                                return payloads;
                            } else if let None = ret {
                                return vec![None];
                            } else if let Some(Err(_)) = ret {
                            }
                        }
                    }
                    Key::Escape => {
                        return vec![
                            Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            })
                        ]
                    },
                    Key::Enter => {
                        return vec![
                            Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            })
                        ]
                    }
                    _ => ()
                }
                self.clear_build();
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
                return vec![None];
            },
            KeybindMode::EditVisual => {
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
                match key_event.key {
                    Key::Escape => {
                        return vec![
                            Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::EditBrowse),
                                ..Default::default()
                            })
                        ]
                    },
                    Key::Enter => {
                        return vec![
                            Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::EditBrowse),
                                ..Default::default()
                            })
                        ]
                    }
                    _ => ()
                }
                return vec![None];
            },
            KeybindMode::SearchedSheet => {
                self.clear_revert_timeout();
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
            KeybindMode::SearchEnter => {
                if let Key::Character(character) = key_event.key {
                    if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                        self.input_string += &character;
                        return vec![Some(ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(self.input_string[1..].to_string()),
                                ..Default::default()
                            })]
                    } else {
                        self.set_new_revert_timeout(ctx);
                        return vec![None];
                    }
                } else if let Key::Backspace = key_event.key {
                    if self.input_string == "/".to_string() {
                        self.clear_build();
                        self.clear_revert_timeout();
                        return vec![Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            })];
                    } else {
                        self.input_string.pop();
                        return vec![Some(ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(self.input_string[1..].to_string()),
                                ..Default::default()
                            })];
                    }

                } else if let Key::Enter = key_event.key {
                    self.clear_build();
                    self.clear_revert_timeout();
                    return vec![Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::SearchedSheet),
                            ..Default::default()
                        })];
                } else {
                    self.clear_build();
                    self.clear_revert_timeout();
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
        self.input_string = String::new();
        self.build_state = BuildState::AwaitOuterCount;
        self.target_string = String::new();
        self.outer_count = String::new();
        self.operator = None;
        self.inner_count = String::new();
        self.text_motion = None;
        self.text_obj = None;
    }

    pub fn set_new_revert_timeout(&mut self, ctx: &mut EventCtx) {
        self.timeout_revert_token = Some(ctx.request_timer(DEFAULT_COMPOSE_TIMEOUT));
    }

    pub fn set_new_build_timeout(&mut self, ctx: &mut EventCtx) {
        self.timeout_build_token = Some(ctx.request_timer(DEFAULT_BUILD_TIMEOUT));
    }

    pub fn clear_revert_timeout(&mut self) {
        self.timeout_revert_token = None;
    }

    pub fn clear_build_timeout(&mut self) {
        self.timeout_build_token = None;
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

    pub fn revert_timeout(&mut self) {
        self.clear_build();
        if let Some(mode) = &self.timeout_revert_mode {
            self.set_keybind_mode((*mode).clone());
            self.set_timeout_revert_mode(None);
        }
    }

    pub fn build_timeout(&mut self) {
        self.clear_build();
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
                self.mode_prompt = String::from("<SHEET>");
                self.input_string = String::from("");
            },
            KeybindMode::Move => {
                self.mode_prompt = String::from("<MOVE>");
                self.input_string = String::from("");
            },
            KeybindMode::EditBrowse => {
                self.text_input.set_keybind_mode(mode);
                self.mode_prompt = String::from("<EDIT>");
                self.input_string = String::from("");
            },
            KeybindMode::EditVisual => {
                self.text_input.set_keybind_mode(mode);
                self.mode_prompt = String::from("<VISUAL>");
                self.input_string = String::from("");
            }
            KeybindMode::Jump => {
                self.mode_prompt = String::from("<JUMP>");
                self.input_string = String::from("");
            },
            KeybindMode::Mark => {
                self.mode_prompt = String::from("<MARK>");
                self.input_string = String::from("");
            },
            KeybindMode::Edit => {
                self.text_input.set_keybind_mode(mode);
                self.mode_prompt = String::from("<INSERT>");
                self.input_string = String::from("");
            },
            KeybindMode::SearchedSheet => {
                self.mode_prompt = String::from("<SELECT>");
                self.input_string = String::from("");
            }
            KeybindMode::SearchEnter => {
                self.mode_prompt = String::from("<SEARCH>");
                self.input_string = String::from("");
            }
            KeybindMode::Global => {
                tracing::error!("KeybindMode::Global should never be set!");
                panic!()
            }
        }
        self.mode = mode;
    }

    pub fn get_string(&self) -> String {
        return self.input_string.clone();
    }

    pub fn get_mode_prompt(&self) -> &str {
        return self.mode_prompt.as_str();
    }

    pub fn get_timeout_revert_token(&self) -> Option<TimerToken> {
        return self.timeout_revert_token;
    }

    pub fn get_timeout_build_token(&self) -> Option<TimerToken> {
        return self.timeout_build_token;
    }
}