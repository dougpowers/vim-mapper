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

use std::collections::{HashMap};

use druid::{keyboard_types::Key, EventCtx, Modifiers, TimerToken, KeyEvent, Event};
use regex::Regex;
use common_macros::hash_map;
use crate::constants::*;

#[derive(Clone, PartialEq, Debug)]
pub enum KeybindMode {
    Sheet,
    EditBrowse,
    Jump,
    Mark,
    Edit,
    Move,
    SearchedSheet,
    SearchBuild,
    KeybindBuild,
}


//The action payload allows regex keybinds to define custom parameters associated with the action.
#[derive(Clone, Debug)]
pub struct ActionPayload {
    pub action: Action,
    pub float: Option<f64>,
    pub index: Option<u16>,
    pub string: Option<String>,
    pub mode: Option<KeybindMode>,
}

impl Default for ActionPayload {
    fn default() -> Self {
        ActionPayload {
            action: Action::NullAction,
            float: None,
            index: None,
            string: None,
            mode: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    NullAction,
    CycleNodeForward,
    CycleNodeBackward,
    CreateNewNode,
    CreateNewNodeAndEdit,
    IncreaseActiveNodeMass,
    DecreaseActiveNodeMass,
    ResetActiveNodeMass,
    AnchorActiveNode,
    ActivateTargetedNode,
    EditActiveNodeSelectAll,
    EditActiveNodeAppend,
    EditActiveNodeInsert,
    DeleteActiveNode,
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
}

#[derive(Clone, PartialEq, Debug)]
pub enum KeybindType {
    Key,
    String,
}

#[derive(Clone, Debug)]
struct Keybind {
    kb_type: KeybindType,
    regex: Option<Regex>,
    group_actions: Option<HashMap<String, HashMap<String, ActionPayload>>>,
    key: Option<Key>,
    modifiers: Option<Modifiers>,
    action_payload: Option<ActionPayload>,
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
            mode: KeybindMode::Sheet,
            keybinds: vec![
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("n"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::CycleNodeForward,
                            ..Default::default()
                        }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("N"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::CycleNodeBackward,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("o"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::CreateNewNodeAndEdit,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("c"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::EditActiveNodeSelectAll,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("d"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::DeleteActiveNode,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanUp,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ZoomIn,
                            float: Some(1.25),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("K"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanUp,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanDown,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: Some(Modifiers::CONTROL), 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ZoomOut,
                            float: Some(0.75),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("J"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanDown,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanRight,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("L"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanRight,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanLeft,
                            float: Some(DEFAULT_PAN_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("H"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::PanLeft,
                            float: Some(DEFAULT_PAN_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Enter),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ActivateTargetedNode,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::F10),
                    modifiers: Some(Modifiers::ALT),
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ToggleColorScheme,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("m"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ChangeModeWithTimeoutRevert,
                            mode: Some(KeybindMode::Mark),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("'"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ChangeModeWithTimeoutRevert,
                            mode: Some(KeybindMode::Jump),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("/"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::SearchBuild),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("G"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::CenterNode,
                            index: Some(0),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("-"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::DecreaseActiveNodeMass,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("+"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::IncreaseActiveNodeMass,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("="))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ResetActiveNodeMass,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("@"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::AnchorActiveNode,
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("`"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Move),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Sheet,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("j"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeDown,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("k"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeUp,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("h"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeLeft,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("l"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeRight,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_SMALL),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("J"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeDown,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("K"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeUp,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("H"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeLeft,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    }),
                    mode: KeybindMode::Move,
                },
                Keybind { 
                    kb_type: KeybindType::Key, 
                    regex: None, 
                    group_actions: None,
                    key: Some(Key::Character(String::from("L"))),
                    modifiers: None, 
                    action_payload: Some(
                        ActionPayload {
                            action: Action::MoveActiveNodeRight,
                            float: Some(DEFAULT_NODE_MOVE_AMOUNT_LARGE),
                            ..Default::default()
                    }),
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
                            String::from("g") => ActionPayload {
                            action: Action::CenterActiveNode,
                            ..Default::default()
                            }
                        }
                    }),
                    key: None,
                    modifiers: None, 
                    action_payload: None,
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

    pub fn accept_key(&mut self, event: KeyEvent, ctx: &mut EventCtx) -> Option<ActionPayload> {
        //Discard modifier key presses
        if event.key == Key::Shift || 
            event.key == Key::Control ||
            event.key == Key::Alt ||
            event.key == Key::CapsLock || 
            event.key == Key::ScrollLock 
        {
            return None;
        } 
        match self.mode {
            KeybindMode::Sheet => {
                self.set_new_timeout(ctx);
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(event.key.clone()) == keybind.key && keybind.mode == self.mode {
                        if event.mods.alt() || event.mods.ctrl() {
                            if let Some(mods) = keybind.modifiers {
                                if event.mods.contains(mods) {
                                    return Some(keybind.action_payload.clone().unwrap());
                                }
                            }
                        } else {
                            return Some(keybind.action_payload.clone().unwrap());
                        }
                        self.clear_timeout();
                    }
                }
                if let Key::Character(character) = event.key {
                    self.set_timeout_revert_mode(Some(self.mode.clone()));
                    self.set_keybind_mode(KeybindMode::KeybindBuild);
                    self.string += &character;
                    return None;
                }
                if let Key::Escape = event.key {
                    self.clear_build();
                    return None;
                }
                return None;
            },
            KeybindMode::EditBrowse => todo!(),
            KeybindMode::Move => {
                let keybinds = self.keybinds.clone();
                for keybind in keybinds {
                    if Some(event.key.clone()) == keybind.key && keybind.mode == self.mode {
                        if event.mods.alt() || event.mods.ctrl() {
                            if let Some(mods) = keybind.modifiers {
                                if event.mods.contains(mods) {
                                    return Some(keybind.action_payload.clone().unwrap());
                                }
                            }
                        } else {
                            return Some(keybind.action_payload.clone().unwrap());
                        }
                        self.clear_timeout();
                    }
                }
                if event.key == Key::Escape || event.key == Key::Enter {
                    self.set_keybind_mode(KeybindMode::Sheet);
                }
                return None;
            }
            KeybindMode::Jump => {
                if let Key::Character(character) = event.key {
                    self.clear_build();
                    self.set_keybind_mode(KeybindMode::Sheet);
                    self.clear_timeout();
                    return Some(
                        ActionPayload {
                            action: Action::JumpToMarkedNode,
                            string: Some(character),
                            ..Default::default()
                    });
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    self.set_keybind_mode(KeybindMode::Sheet);
                    return None;
                }
            },
            KeybindMode::Mark => {
                if let Key::Character(character) = event.key {
                    self.clear_build();
                    self.set_keybind_mode(KeybindMode::Sheet);
                    self.clear_timeout();
                    return Some(
                        ActionPayload {
                            action: Action::MarkActiveNode,
                            string: Some(character),
                            ..Default::default()
                    });
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    self.set_keybind_mode(KeybindMode::Sheet);
                    return None;
                }
            },
            KeybindMode::KeybindBuild => {
                if let Key::Character(character) = event.key {
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
                                return Some(VMInputManager::process_regex_keybind(keybind.clone(), matched));
                            }
                        }
                    }
                    return None;
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    self.set_keybind_mode(KeybindMode::Sheet);
                    return None;
                }
            },
            KeybindMode::Edit => todo!(),
            KeybindMode::SearchedSheet => todo!(),
            KeybindMode::SearchBuild => {
                if let Key::Character(character) = event.key {
                    self.string += &character;
                    return Some(
                        ActionPayload {
                            action: Action::SearchNodes,
                            string: Some(self.string.clone()),
                            ..Default::default()
                        }
                    )
                } else if let Key::Backspace = event.key {
                    if self.string == "/".to_string() {
                        self.clear_build();
                        self.clear_timeout();
                        self.set_keybind_mode(KeybindMode::Sheet);
                        return None;
                    } else {
                        self.string.pop();
                        return Some(
                            ActionPayload {
                                action: Action::SearchNodes,
                                string: Some(self.string.clone()),
                                ..Default::default()
                            }
                        )
                    }

                } else if let Key::Enter = event.key {
                    self.clear_build();
                    self.clear_timeout();
                    self.set_keybind_mode(KeybindMode::SearchedSheet);
                    return None;
                } else {
                    self.clear_build();
                    self.clear_timeout();
                    self.set_keybind_mode(KeybindMode::Sheet);
                    return None;
                }
            },
        }
    }

    pub fn clear_build(&mut self) {
        self.string = String::from("");
    }

    fn process_regex_keybind(keybind: Keybind, string: String) -> ActionPayload {
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
        return ActionPayload {
            ..Default::default()
        }
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
        println!("Changing mode to {:?}", mode);
        match mode {
            KeybindMode::Sheet => {
                self.string = String::from("");
            },
            KeybindMode::KeybindBuild => {

            },
            KeybindMode::Move => {
                self.string = String::from("<MOVE>");
            },
            KeybindMode::EditBrowse => todo!(),
            KeybindMode::Jump => {
                self.string = String::from("'");
            },
            KeybindMode::Mark => {
                self.string = String::from("m");
            },
            KeybindMode::Edit => todo!(),
            KeybindMode::SearchedSheet => todo!(),
            KeybindMode::SearchBuild => {
                self.string = String::from("/");
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