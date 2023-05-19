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
use std::{path::PathBuf};

use druid::{keyboard_types::Key, EventCtx, Modifiers, TimerToken, KeyEvent, RawMods, Data, text::EditableText, Target, Command, Point};
use unicode_segmentation::UnicodeSegmentation;
use bitflags::bitflags;

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
    InsertNewNode,
    CreateNewNodeAndEdit,
    CreateNewExternalNode,
    IncreaseNodeMass,
    DecreaseNodeMass,
    ResetNodeMass,
    ToggleNodeAnchor,
    ActivateTargetedNode,
    EditActiveNodeSelectAll,
    EditActiveNodeAppend,
    EditActiveNodeInsert,
    CutNode,
    CutNodeTree,
    AttemptNodeDeletion,
    CutTargetNode,
    YankNodeTree,
    YankNode,
    PasteNodeTree,
    PasteNodeTreeExternal,
    PasteNodeTreeAsTab,
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
    InsertCharacterUnconfirmed,
    ConfirmInserts,
    RollBackInserts,
    Highlight,
    ExecuteTextAction,
    AcceptNodeText,
    UndoNodeText,
    RedoNodeText,
    ToggleColorScheme,
    ToggleDebug,
    ToggleMenuVisible,
    CreateDialog,
    PrintToLogInfo,
    CreateNewTab,
    OpenNewTabInput,
    DeleteActiveTab,
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
    None,
}


#[allow(dead_code)]
#[derive(Data, Copy, Clone, PartialEq, Debug)]
pub enum TextMotion {
    ForwardCharacter,
    BackwardCharacter,
    ForwardWordStart,
    BackwardWordStart,
    ForwardWordEnd,
    BackwardWordEnd,
    ForwardToN,
    BackwardToN,
    ForwardWithN,
    BackwardWithN,
    BeginningLine,
    EndLine,
    WholeLine,
}

#[allow(dead_code)]
#[derive(Data, Clone, Copy, PartialEq, Debug)]
pub enum TextOperation {
    None,
    DeleteText,
    ChangeText,
    ReplaceText,
}

#[allow(dead_code)]
#[derive(Data, Clone, PartialEq, Debug)]
pub struct TextAction {
    pub (crate) operation: TextOperation,
    pub (crate) outer_count: Option<usize>,
    pub (crate) inner_count: Option<usize>,
    pub (crate) text_obj: Option<TextObj>,
    pub (crate) text_motion: Option<TextMotion>,
    pub (crate) character_string: Option<String>,
}

impl Default for TextAction {
    fn default() -> Self {
        TextAction {
            operation: TextOperation::None,
            outer_count: None,
            inner_count: None,
            text_motion: None,
            text_obj: None,
            character_string: None,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct KeybindMode: u32 {
        const Start =           0b000000000001;
        const Dialog =          0b000000000010;
        const Sheet =           0b000000000100;
        const Edit =            0b000000001000;
        const Visual =          0b000000010000;
        const Insert =          0b000000100000;
        const Jump =            0b000001000000;
        const Mark =            0b000010000000;
        const Move =            0b000100000000;
        const SearchedSheet =   0b001000000000;
        const SearchEntry =     0b010000000000;
        const Global =          0b100000000000;
    }
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
    pub pos: Option<Point>,
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
            pos: None,
        }
    }
}


#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Keybind {
    kb_type: KeybindType,
    key: Option<Key>,
    string: Option<String>,
    modifiers: Option<Modifiers>,
    action_payloads: Vec<Option<ActionPayload>>,
    operation: Option<TextOperation>,
    motion: Option<TextMotion>,
    obj: Option<TextObj>,
    next: Option<BuildState>,
    subcommands: Option<Vec<Subcommand>>,
    accepts_inner_count: Option<bool>,
    accepts_outer_count: Option<bool>,
    mode: KeybindMode,
}

#[derive(Clone, Debug)]
struct Subcommand {
    string: String,
    action_payloads: Vec<Option<ActionPayload>>,
}

impl Default for Keybind {
    fn default() -> Self {
        Keybind {
            kb_type: KeybindType::Key,
            key: None,
            string: None,
            modifiers: None,
            action_payloads: vec![],
            operation: None,
            motion: None,
            next: None,
            subcommands: None,
            accepts_inner_count: None,
            accepts_outer_count: None,
            obj: None,
            mode: KeybindMode::Global,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Data, Debug)]
pub enum BuildState {
    AwaitOuterCount,
    AwaitOperator,
    AwaitInnerCount,
    AwaitTarget,
    AwaitSubcommand,
    AwaitCharacter,
    Complete,
}

pub struct VMInputManager {
    mode: KeybindMode,
    build_state: BuildState,
    // pub(crate) text_input: VMTextInput,
    keybinds: Vec<Keybind>,
    input_string: String,
    outer_count: String,
    accepts_outer_count: Option<bool>,
    operation: Option<TextOperation>,
    subcommands: Option<Vec<Subcommand>>,
    payloads: Option<Vec<Option<ActionPayload>>>,
    inner_count: String,
    accepts_inner_count: Option<bool>,
    target_string: String,
    character_string: String,
    text_obj: Option<TextObj>,
    text_motion: Option<TextMotion>,
    mode_label: String,
    mode_prompt: String,
    timeout_build_token: Option<TimerToken>,
    timeout_revert_token: Option<TimerToken>,
    timeout_revert_mode: Option<KeybindMode>,
    string_keybind_cache: Vec<Keybind>,
}

impl Default for VMInputManager {
    fn default() -> Self {
        let mut vim = VMInputManager {
            mode: KeybindMode::Start,
            build_state: BuildState::AwaitOuterCount,
            // text_input: VMTextInput::new(),
            mode_label: String::new(),
            input_string: String::new(),
            mode_prompt: String::new(),
            outer_count: String::new(),
            accepts_outer_count: None,
            operation: None,
            payloads: None,
            inner_count: String::new(),
            accepts_inner_count: None,
            target_string: String::new(),
            character_string: String::new(),
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
                            mode: Some(KeybindMode::Edit),
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
                            mode: Some(KeybindMode::Visual),
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Edit,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("a"))),
                    modifiers: None, 
                    action_payloads: vec![
                        Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Insert),
                            ..Default::default()
                        }),
                        Some(ActionPayload {
                            action: Action::ExecuteTextAction,
                            text_action: Some(TextAction { 
                                operation: TextOperation::None, 
                                outer_count: None, 
                                inner_count: None, 
                                text_obj: None, 
                                text_motion: Some(TextMotion::ForwardCharacter), 
                                character_string: None,
                            }),
                            ..Default::default()
                        })
                    ],
                    mode: (KeybindMode::Edit | KeybindMode::Sheet),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("i"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Insert),
                            ..Default::default()
                    })],
                    mode: (KeybindMode::Edit),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("s"))),
                    modifiers: None, 
                    action_payloads: vec![
                        Some(ActionPayload {
                            action: Action::ExecuteTextAction,
                            text_action: Some(TextAction {
                                operation: TextOperation::DeleteText,
                                text_motion: Some(TextMotion::ForwardCharacter),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        Some(ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Insert),
                            ..Default::default()
                    })],
                    mode: (KeybindMode::Edit),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("i"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::InsertNewNode,
                            ..Default::default()
                    })],
                    mode: (KeybindMode::Sheet),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("A"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Insert),
                            ..Default::default()
                    }),
                    Some(
                        ActionPayload {
                            action: Action::ExecuteTextAction,
                            text_action: Some(TextAction { 
                                operation: TextOperation::None, 
                                outer_count: None, 
                                inner_count: None, 
                                text_obj: None, 
                                text_motion: Some(TextMotion::EndLine), 
                                character_string: None,
                            }),
                            ..Default::default()
                        }
                    )],
                    mode: (KeybindMode::Edit | KeybindMode::Sheet),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("I"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Insert),
                            ..Default::default()
                    }),
                    Some(
                        ActionPayload {
                            action: Action::ExecuteTextAction,
                            text_action: Some(TextAction { 
                                operation: TextOperation::None, 
                                outer_count: None, 
                                inner_count: None, 
                                text_obj: None, 
                                text_motion: Some(TextMotion::BeginningLine), 
                                character_string: None,
                            }),
                            ..Default::default()
                        }
                    )],
                    mode: (KeybindMode::Edit | KeybindMode::Sheet),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("d"))),
                    modifiers: None, 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::AttemptNodeDeletion,
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
                            action: Action::CutNode,
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
                            action: Action::CutTargetNode,
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ZoomOut,
                            float: Some(0.75),
                            ..Default::default()
                    })],
                    mode: (KeybindMode::Sheet | KeybindMode::Move | KeybindMode::Edit | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::ZoomIn,
                            float: Some(1.25),
                            ..Default::default()
                    })],
                    mode: (KeybindMode::Sheet | KeybindMode::Move | KeybindMode::Edit | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
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
                    mode: (KeybindMode::Sheet | KeybindMode::SearchedSheet),
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
                    mode: KeybindMode::Global,
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
                                mode: Some(KeybindMode::SearchEntry),
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
                            action: Action::DecreaseNodeMass,
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
                            action: Action::IncreaseNodeMass,
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
                            action: Action::ResetNodeMass,
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
                            action: Action::ToggleNodeAnchor,
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
                            action: Action::ToggleNodeAnchor,
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
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(true),
                    mode: KeybindMode::Edit,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("d".to_string()),
                    operation: Some(TextOperation::DeleteText),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(true),
                    mode: KeybindMode::Edit,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("D".to_string()),
                    operation: Some(TextOperation::DeleteText),
                    motion: Some(TextMotion::EndLine),
                    mode: KeybindMode::Edit,
                    next: Some(BuildState::Complete),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("C".to_string()),
                    operation: Some(TextOperation::ChangeText),
                    motion: Some(TextMotion::EndLine),
                    mode: KeybindMode::Edit,
                    next: Some(BuildState::Complete),
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("S".to_string()),
                    operation: Some(TextOperation::ChangeText),
                    motion: Some(TextMotion::WholeLine),
                    mode: KeybindMode::Edit,
                    next: Some(BuildState::Complete),
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("iw".to_string()),
                    obj: Some(TextObj::InnerWord),
                    accepts_outer_count: Some(false),
                    accepts_inner_count: Some(false),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("aw".to_string()),
                    obj: Some(TextObj::OuterWord),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("t".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::ForwardToN),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::AwaitCharacter),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("T".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::BackwardToN),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::AwaitCharacter),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("f".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::ForwardWithN),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::AwaitCharacter),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("F".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::BackwardWithN),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::AwaitCharacter),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("l".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::ForwardCharacter),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("$".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::EndLine),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("^".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::BeginningLine),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some(" ".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::ForwardCharacter),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("h".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::BackwardCharacter),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("w".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::ForwardWordStart),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("e".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::ForwardWordEnd),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("b".to_string()),
                    operation: Some(TextOperation::None),
                    motion: Some(TextMotion::BackwardWordStart),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("x".to_string()),
                    operation: Some(TextOperation::DeleteText),
                    motion: Some(TextMotion::ForwardCharacter),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::Complete),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("r".to_string()),
                    operation: Some(TextOperation::ReplaceText),
                    accepts_outer_count: Some(true),
                    accepts_inner_count: Some(false),
                    next: Some(BuildState::AwaitCharacter),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some("j".to_string()),
                    operation: Some(TextOperation::None),
                    subcommands: Some(vec![
                        Subcommand{
                            string: "j".to_string(),
                            action_payloads: vec![Some(
                                ActionPayload {
                                    action: Action::AcceptNodeText,
                                    ..Default::default()
                                }),
                                Some(ActionPayload {
                                    action: Action::ChangeMode,
                                    mode: Some(KeybindMode::Edit),
                                    ..Default::default()
                                }
                            )],
                        },
                    ]),
                    next: Some(BuildState::AwaitSubcommand),
                    mode: KeybindMode::Insert,
                    ..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("g".to_string()),
                    operation: Some(TextOperation::None),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CenterNode,
                            index: Some(0),
                            ..Default::default()
                    })],
                    subcommands: Some(vec![
                        Subcommand {
                            string: "g".to_string(),
                            action_payloads: vec![Some(
                                ActionPayload {
                                    action: Action::CenterActiveNode,
                                    index: Some(0),
                                    ..Default::default()
                                }
                            )]
                        }
                    ]),
                    next: Some(BuildState::AwaitSubcommand),
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind { 
                    kb_type: KeybindType::String, 
                    string: Some("y".to_string()),
                    operation: Some(TextOperation::None),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::CenterNode,
                            index: Some(0),
                            ..Default::default()
                    })],
                    subcommands: Some(vec![
                        Subcommand {
                            string: "i".to_string(),
                            action_payloads: vec![Some(
                                ActionPayload {
                                    action: Action::YankNode,
                                    ..Default::default()
                                }
                            )],
                        },
                        Subcommand {
                            string: "y".to_string(),
                            action_payloads: vec![Some(
                                ActionPayload {
                                    action: Action::YankNodeTree,
                                    ..Default::default()
                                }
                            )]
                        },
                    ]),
                    next: Some(BuildState::AwaitSubcommand),
                    mode: KeybindMode::Sheet,
					..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character("p".to_string())),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PasteNodeTree,
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Sheet,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character("u".to_string())),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::UndoNodeText,
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character("r".to_string())),
                    modifiers: Some(Modifiers::CONTROL),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::RedoNodeText,
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Edit,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character("P".to_string())),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PasteNodeTreeExternal,
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Sheet,
                    ..Default::default()
                },
                Keybind {
                    kb_type: KeybindType::Key,
                    key: Some(Key::Character("p".to_string())),
                    modifiers: Some(Modifiers::CONTROL),
                    action_payloads: vec![Some(
                        ActionPayload {
                            action: Action::PasteNodeTreeAsTab,
                            ..Default::default()
                        }
                    )],
                    mode: KeybindMode::Sheet,
                    ..Default::default()
                },
            ],
            subcommands: None,
        };

        for (od, cd) in ACCEPTED_DELIMITERS {
            vim.keybinds.push(
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some(format!("i{}", od)),
                    obj: Some(TextObj::Inner(format!("{}{}", od, cd))),
                    accepts_outer_count: Some(false),
                    accepts_inner_count: Some(false),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                }
            );
            vim.keybinds.push(
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some(format!("a{}", od)),
                    obj: Some(TextObj::Outer(format!("{}{}", od, cd))),
                    accepts_outer_count: Some(false),
                    accepts_inner_count: Some(false),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                }
            );
            vim.keybinds.push(
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some(format!("i{}", cd)),
                    obj: Some(TextObj::Inner(format!("{}{}", od, cd))),
                    accepts_outer_count: Some(false),
                    accepts_inner_count: Some(false),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                }
            );
            vim.keybinds.push(
                Keybind {
                    kb_type: KeybindType::String,
                    string: Some(format!("a{}", cd)),
                    obj: Some(TextObj::Outer(format!("{}{}", od, cd))),
                    accepts_outer_count: Some(false),
                    accepts_inner_count: Some(false),
                    mode: KeybindMode::Edit,
                    ..Default::default()
                }
            );
        }

        return vim;
    }
}

impl VMInputManager {
    pub fn new() -> VMInputManager {
        let mut vim = VMInputManager {
            mode: KeybindMode::Sheet,
            mode_label: String::from("<sheet>"),
            input_string: String::from(""),
            mode_prompt: String::from(""),
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
                    return self.process_keybind_string(string);
                }
            },
            BuildState::AwaitOperator => {
                if !string.contains(char::is_numeric) {
                    return self.process_keybind_string(string);
                } else {
                    return Some(Err(()))
                }
            },
            BuildState::AwaitInnerCount => {
                if string.contains(char::is_numeric) {
                    self.inner_count += &string;
                    return None;
                } else {
                    self.build_state = BuildState::AwaitTarget;
                    return self.process_keybind_string(string);
                }
            },
            BuildState::AwaitTarget => {
                if !string.contains(char::is_numeric) {
                    return self.process_keybind_string(string);
                } else {
                    return Some(Err(()))
                }
            },
            BuildState::AwaitCharacter => {
                return self.process_keybind_string(string)
            },
            BuildState::Complete => {
                return self.process_keybind_string("".to_string());
            },
            BuildState::AwaitSubcommand => {
                return self.process_keybind_string(string);
            }
        }
    }

    fn process_keybind_string(&mut self, string: String) -> Option<Result<Vec<Option<ActionPayload>>, ()>> {
        if self.build_state == BuildState::AwaitOperator {
            for keybind in &self.string_keybind_cache {
                if keybind.mode.contains(self.mode) {
                    if let Some(k_string) = keybind.string.clone() {
                        if let Some(operation) = keybind.operation {
                            if k_string.slice(0..k_string.next_grapheme_offset(0).unwrap()).unwrap() == string {
                                if k_string == string {
                                    // tracing::debug!("matched {} with {}", string, k_string);
                                    self.accepts_outer_count = keybind.accepts_outer_count;
                                    self.accepts_inner_count = keybind.accepts_inner_count;
                                    self.text_motion = keybind.motion.clone();
                                    self.operation = Some(operation);
                                    self.subcommands = keybind.subcommands.clone();
                                    if Some(BuildState::Complete) == keybind.next {
                                        return Some(Ok(self.build_text_action()));
                                    } else if let Some(state) = keybind.next {
                                        self.build_state = state;
                                    } else if Some(true) == self.accepts_inner_count {
                                        self.build_state = BuildState::AwaitInnerCount;
                                    }
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
        } else if self.build_state == BuildState::AwaitTarget {
            self.target_string += &string;
            let mut partial_count: usize = 0;
            for keybind in &self.string_keybind_cache {
                if keybind.mode.contains(self.mode) {
                    if let Some(k_string) = keybind.string.clone() {
                        if keybind.obj.is_some() || keybind.motion.is_some() || keybind.action_payloads.len() > 0 {
                            let k_graphs = k_string.graphemes(true).collect::<Vec<&str>>();
                            let s_graphs = self.target_string.graphemes(true).collect::<Vec<&str>>();
                            let mut partial = false;
                            for k_i in 0..k_graphs.len() {
                                if let Some(s_graph) = s_graphs.get(k_i) {
                                    if Some(k_graphs[k_i]) == Some(s_graph) {
                                        partial = true;
                                        if k_i == k_graphs.len()-1 {
                                            // tracing::debug!("full match {} with {}", self.target_string, k_string);
                                            if let Some(outer_accepted) = keybind.accepts_outer_count {
                                                self.accepts_outer_count = Some(outer_accepted);
                                            }
                                            if let Some(inner_accepted) = keybind.accepts_inner_count {
                                                self.accepts_inner_count = Some(inner_accepted);
                                            }
                                            if let Some(motion) = keybind.motion.clone() {
                                                self.text_motion = Some(motion);
                                            } else if let Some(object) = keybind.obj.clone() {
                                                self.text_obj = Some(object);
                                            } else if keybind.action_payloads.len() > 0 {
                                                self.payloads = Some(keybind.action_payloads.clone());
                                            }
                                            if Some(BuildState::Complete) == keybind.next {
                                                // tracing::debug!("Complete match");
                                                return Some(Ok(self.build_text_action()));
                                            } else if let Some(next_state) = keybind.next {
                                                self.build_state = next_state;
                                                return None;
                                            } else {
                                                return Some(Ok(self.build_text_action()));
                                            }
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    partial_count += 1;
                                    if partial {
                                        // tracing::debug!("partial match {} with {}", self.target_string, k_string);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if partial_count > 0 {
                // tracing::debug!("indicates there are still partial matches");
                return None;
            }
        } else if self.build_state == BuildState::AwaitCharacter {
            self.character_string = string;
            return Some(Ok(self.build_text_action()));
        } else if self.build_state == BuildState::Complete {
            return Some(Ok(self.build_text_action()));
        } else if self.build_state == BuildState::AwaitSubcommand {
            if let Some(subcommands) = &self.subcommands {
                for subcommand in subcommands {
                    if subcommand.string == string {
                        self.payloads = Some(subcommand.action_payloads.clone());
                        return Some(Ok(self.build_text_action()));
                    }
                }
            }
        }
        return Some(Err(()));
    }

    fn build_text_action(&mut self) -> Vec<Option<ActionPayload>> {
        let mut payload = ActionPayload {
            ..Default::default()
        };

        let mut text_action = TextAction {
            operation: TextOperation::None,
            outer_count: None,
            inner_count: None,
            text_motion: None,
            text_obj: None,
            character_string: None,
        };

        if let Some(operator) = self.operation {
            match operator {
                TextOperation::DeleteText => {
                    payload.action = Action::ExecuteTextAction;
                    text_action.operation = TextOperation::DeleteText;

                    if let Some(motion) = &self.text_motion {
                        text_action.text_motion = Some(motion.clone());
                    } else if let Some(obj) = &self.text_obj {
                        text_action.text_obj = Some(obj.clone());
                    }

                    if self.character_string.len() > 0 {
                        text_action.character_string = Some(self.character_string.clone());
                    }

                    if self.inner_count.len() > 0 && Some(true) == self.accepts_inner_count {
                        text_action.inner_count = Some(self.inner_count.parse::<usize>().unwrap());
                    }

                    if Some(true) == self.accepts_outer_count && self.outer_count.len() > 0 {
                        text_action.outer_count = Some(self.outer_count.parse::<usize>().unwrap());
                    }
                    payload.text_action = Some(text_action);
                    self.clear_build();
                    return vec![Some(payload)];
                },
                TextOperation::ChangeText => {
                    payload.action = Action::ExecuteTextAction;
                    text_action.operation = TextOperation::ChangeText;

                    if let Some(motion) = &self.text_motion {
                        text_action.text_motion = Some(motion.clone());
                    } else if let Some(obj) = &self.text_obj {
                        text_action.text_obj = Some(obj.clone());
                    }

                    if self.character_string.len() > 0 {
                        text_action.character_string = Some(self.character_string.clone());
                    }

                    if self.inner_count.len() > 0 && Some(true) == self.accepts_inner_count {
                        text_action.inner_count = Some(self.inner_count.parse::<usize>().unwrap());
                    }

                    if Some(true) == self.accepts_outer_count && self.outer_count.len() > 0 {
                        text_action.outer_count = Some(self.outer_count.parse::<usize>().unwrap());
                    }
                    payload.text_action = Some(text_action);
                    self.clear_build();
                    return vec![Some(payload)];
                },
                TextOperation::ReplaceText => {
                    payload.action = Action::ExecuteTextAction;
                    text_action.operation = TextOperation::ReplaceText;

                    if self.character_string.len() > 0 {
                        text_action.character_string = Some(self.character_string.clone());
                    }

                    if self.inner_count.len() > 0 && Some(true) == self.accepts_inner_count {
                        text_action.inner_count = Some(self.inner_count.parse::<usize>().unwrap());
                    }

                    if Some(true) == self.accepts_outer_count && self.outer_count.len() > 0 {
                        text_action.outer_count = Some(self.outer_count.parse::<usize>().unwrap());
                    }
                    payload.text_action = Some(text_action);
                    self.clear_build();
                    return vec![Some(payload)];
                },
                TextOperation::None => {
                    if let Some(motion) = &self.text_motion {
                        payload.action = Action::ExecuteTextAction;
                        text_action.operation = TextOperation::None;
                        text_action.text_motion = Some(motion.clone());

                        if self.character_string.len() > 0 {
                            text_action.character_string = Some(self.character_string.clone());
                        }

                        if self.inner_count.len() > 0 && Some(true) == self.accepts_inner_count {
                            text_action.inner_count = Some(self.inner_count.parse::<usize>().unwrap());
                        }

                        if Some(true) == self.accepts_outer_count && self.outer_count.len() > 0 {
                            text_action.outer_count = Some(self.outer_count.parse::<usize>().unwrap());
                        }
                        payload.text_action = Some(text_action);
                        self.clear_build();
                        return vec![Some(payload)]
                    } else if self.payloads.is_some() {
                        let payloads = self.payloads.clone().unwrap();
                        self.clear_build();
                        return payloads;
                    }
                }
            }
        } 
        self.clear_build();
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

        // tracing::debug!("{:?}", key_event.key);

        match self.mode {
            KeybindMode::Start => {
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode.intersects(self.mode | KeybindMode::Global)) {
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
                if let Key::Character(character) = &key_event.key {
                    self.input_string += &character;
                    self.set_timeout_revert_mode(Some(self.mode));
                    let ret = self.build_keybind_string(character.clone());
                    if let Some(Ok(payloads)) = ret {
                        return payloads;
                    } else if let None = ret {
                        return vec![None];
                    } else if let Some(Err(_)) = ret {
                        self.clear_build();
                        if self.input_string.len() >= 1 {
                            self.clear_build();
                            return vec![None];
                        }
                    }
                }
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode.intersects(self.mode | KeybindMode::Global)) {
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
                if let Key::Escape = &key_event.key {
                    self.clear_build();
                    return vec![None];
                }
                return vec![None];
            },
            KeybindMode::Move => {
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode.intersects(self.mode | KeybindMode::Global)) {
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
            KeybindMode::Insert => {
                match key_event.key {
                    Key::Character(character) => {
                        self.input_string += &character;
                        let ret = self.build_keybind_string(character.clone());
                        if let Some(Ok(mut payloads)) = ret {
                            self.clear_build();
                            payloads.push(Some(ActionPayload {
                                action: Action::RollBackInserts,
                                ..Default::default()
                            }));
                            return payloads;
                        } else if let None = ret {
                            self.set_new_build_timeout(ctx);
                            return vec![Some(ActionPayload {
                                action: Action::InsertCharacterUnconfirmed,
                                string: Some(character),
                                ..Default::default()
                            })];
                        } else if let Some(Err(_)) = ret {
                            self.clear_build();
                            return vec![
                                Some(ActionPayload {
                                    action: Action::ConfirmInserts,
                                    ..Default::default()
                                }),
                                Some(ActionPayload {
                                    action: Action::InsertCharacter,
                                    string: Some(character),
                                    ..Default::default()
                                })
                            ];
                        } else {
                            return vec![None];
                        }
                                    }
                    Key::Backspace => {
                    self.clear_build();
                    return vec![
                        Some(ActionPayload {
                        action: Action::ExecuteTextAction,
                        text_action: Some(TextAction {
                                operation: TextOperation::DeleteText,
                                text_motion: Some(TextMotion::BackwardCharacter),
                                ..Default::default()
                            }),
                        ..Default::default()
                        }),
                        Some(ActionPayload {
                            action: Action::ConfirmInserts,
                            ..Default::default()
                        }),
                    ]
                    },
                    Key::Delete => {
                    self.clear_build();
                    return vec![
                        Some(ActionPayload {
                        action: Action::ExecuteTextAction,
                        text_action: Some(TextAction {
                                operation: TextOperation::DeleteText,
                                text_motion: Some(TextMotion::ForwardCharacter),
                                ..Default::default()
                            }),
                        ..Default::default()
                        }),
                        Some(ActionPayload {
                            action: Action::ConfirmInserts,
                            ..Default::default()
                        })
                    ]
                    },
                    Key::Enter => {
                    self.clear_build();
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
                    self.clear_build();
                    return vec![Some(ActionPayload {
                        action: Action::AcceptNodeText,
                        ..Default::default()    
                        }),
                        Some(ActionPayload {
                        action: Action::ChangeMode,
                        mode: Some(KeybindMode::Edit),
                        ..Default::default()
                    })]
                    },
                    Key::ArrowRight => {
                    self.clear_build();
                    return vec![
                        Some(ActionPayload {
                        action: Action::ExecuteTextAction,
                        text_action: Some(TextAction {
                            text_motion: Some(TextMotion::ForwardCharacter),
                                ..Default::default()
                            }),
                        ..Default::default()
                        }),
                    ]
                    },
                    Key::ArrowLeft => {
                    self.clear_build();
                    return vec![
                        Some(ActionPayload {
                        action: Action::ExecuteTextAction,
                        text_action: Some(TextAction {
                            text_motion: Some(TextMotion::BackwardCharacter),
                                ..Default::default()
                            }),
                        ..Default::default()
                        }),
                    ]
                    },
                    _ => {
                    return vec![None];
                    }
                }
            },
            KeybindMode::Edit => {
                match &key_event.key {
                    Key::Character(character) => {
                        if key_event.mods.is_empty() || key_event.mods.shift() {
                            self.input_string += &character;
                            let ret = self.build_keybind_string(character.clone());
                            if let Some(Ok(payloads)) = ret {
                                return payloads;
                            } else if let None = ret {
                                return vec![None];
                            } else if let Some(Err(_)) = ret {
                                self.clear_build();
                                if self.input_string.len() >= 1 {
                                    self.clear_build();
                                    return vec![None];
                                }
                            }
                        } 
                    }
                    Key::Escape => {
                        if self.build_state == BuildState::AwaitOuterCount && self.outer_count.len() == 0 {
                            self.clear_build();
                            return vec![
                                Some(ActionPayload {
                                    action: Action::ChangeMode,
                                    mode: Some(KeybindMode::Sheet),
                                    ..Default::default()
                                })
                            ]
                        } else {
                            self.clear_build();
                        }
                    },
                    Key::Enter => {
                        self.clear_build();
                        return vec![
                            Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Sheet),
                                ..Default::default()
                            })
                        ]
                    },
                    Key::ArrowRight => {
                        return vec![
                            Some(ActionPayload {
                            action: Action::ExecuteTextAction,
                            text_action: Some(TextAction {
                                text_motion: Some(TextMotion::ForwardCharacter),
                                    ..Default::default()
                                }),
                            ..Default::default()
                            }),
                        ]
                    },
                    Key::ArrowLeft => {
                        return vec![
                            Some(ActionPayload {
                            action: Action::ExecuteTextAction,
                            text_action: Some(TextAction {
                                text_motion: Some(TextMotion::BackwardCharacter),
                                    ..Default::default()
                                }),
                            ..Default::default()
                            }),
                        ]
                    },
                    _ => ()
                }
                self.clear_build();
                for keybind in &self.keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode.intersects(self.mode | KeybindMode::Global)) {
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
            KeybindMode::Visual => {
                for keybind in &self.keybinds {
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode.intersects(self.mode | KeybindMode::Global)) {
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
                                mode: Some(KeybindMode::Edit),
                                ..Default::default()
                            })
                        ]
                    },
                    Key::Enter => {
                        return vec![
                            Some(ActionPayload {
                                action: Action::ChangeMode,
                                mode: Some(KeybindMode::Edit),
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
                    if Some(key_event.key.clone()) == keybind.key && (keybind.mode.intersects(self.mode | KeybindMode::Global)) {
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
                        ),
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
            KeybindMode::SearchEntry => {
                if let Key::Character(character) = key_event.key {
                    if key_event.mods == RawMods::None || key_event.mods == RawMods::Shift {
                        self.input_string += &character;
                        return vec![Some(ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(self.input_string.clone()),
                                ..Default::default()
                            })]
                    } else {
                        self.set_new_revert_timeout(ctx);
                        return vec![None];
                    }
                } else if let Key::Backspace = key_event.key {
                    if self.input_string.is_empty() {
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
                                string: Some(self.input_string.clone()),
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
            },
            _ => { 
                tracing::error!("Non-existent KeybindMode mode set!");
                panic!();
            },
        }
    }

    pub fn clear_build(&mut self) {
        self.clear_build_timeout();
        self.input_string = String::new();
        self.payloads = None;
        self.build_state = BuildState::AwaitOuterCount;
        self.target_string = String::new();
        self.outer_count = String::new();
        self.operation = None;
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

    pub fn revert_timeout(&mut self, _ctx: &mut EventCtx) {
        self.clear_build();
        if let Some(mode) = &self.timeout_revert_mode {
            self.set_keybind_mode((*mode).clone());
            self.set_timeout_revert_mode(None);
        }
    }

    pub fn build_timeout(&mut self, ctx: &mut EventCtx) {
        self.clear_build();
        ctx.submit_command(
            Command::new(EXECUTE_ACTION, 
                ActionPayload {
                    action: Action::ConfirmInserts,
                    ..Default::default()
                },
                Target::Global,
        ));
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
                self.mode_label = String::from("<sheet>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("");
            },
            KeybindMode::Move => {
                self.mode_label = String::from("<move>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("");
            },
            KeybindMode::Edit => {
                // self.text_input.set_keybind_mode(mode);
                self.mode_label = String::from("<edit>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("");
            },
            KeybindMode::Visual => {
                // self.text_input.set_keybind_mode(mode);
                self.mode_label = String::from("<visual>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("");
            }
            KeybindMode::Jump => {
                self.mode_label = String::from("<jump>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("'");
            },
            KeybindMode::Mark => {
                self.mode_label = String::from("<mark>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("m");
            },
            KeybindMode::Insert => {
                // self.text_input.set_keybind_mode(mode);
                self.mode_label = String::from("<insert>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("");
            },
            KeybindMode::SearchedSheet => {
                self.mode_label = String::from("<select>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("");
            }
            KeybindMode::SearchEntry => {
                self.mode_label = String::from("<search>");
                self.input_string = String::from("");
                self.mode_prompt = String::from("/");
            }
            KeybindMode::Global => {
                tracing::error!("KeybindMode::Global should never be set!");
                panic!();
            }
            _ => {
                tracing::error!("Non-existent KeybindMode set!");
                panic!();
            }
        }
        self.mode = mode;
    }

    pub fn get_string(&self) -> String {
        return self.input_string.clone();
    }

    pub fn get_mode_label(&self) -> &str {
        return self.mode_label.as_str();
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