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

use druid::{widget::{Flex, SizedBox, Label, MainAxisAlignment, Controller, TextBox}, WidgetExt, Command, Target, WidgetPod, Widget, RawMods, Data, Event, keyboard_types::Key};

use core::fmt::Debug;
use std::fmt::Display;

use crate::{vmconfig::{VMConfigVersion4, VMColor}, vminput::{Action, ActionPayload}, vmsave::VMSaveState};

use crate::constants::*;

use crate::vmbutton::VMButton;

pub struct VMDialog {
    pub(crate) inner: WidgetPod<String, Flex<String>>,
}

#[derive(Debug, Clone)]
pub struct VMDialogParams {
    pub buttons: Vec<(String, Vec<ActionPayload>, bool)>,
    pub prompts: Vec<(String, Option<VMColor>)>,
    // pub input: Option<(String, ActionPayload)>,
}

#[derive(Debug, Clone)]
pub struct VMInputParams {
    pub prompts: Vec<(String, Option<VMColor>)>,
    pub button: (String, bool),
    pub input_actions: Vec<ActionPayload>,
}

struct VMDialogController;

impl<T, W: Widget<T>> Controller<T, W> for VMDialogController {
    fn event(&mut self, child: &mut W, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut T, env: &druid::Env) {
        if ctx.has_focus() {
        }
        if let druid::Event::KeyUp(key_event) = event {
            if key_event.key == druid::keyboard_types::Key::Escape && key_event.mods == RawMods::None {
                ctx.submit_command(Command::new(
                    DIALOG_EXECUTE_ACTIONS,
                    vec![ActionPayload {
                        action: Action::NullAction,
                        ..Default::default()
                    }],
                    Target::Global
                ))
            }
        }
        child.event(ctx, event, data, env);
    }
    fn lifecycle(
            &mut self,
            child: &mut W,
            ctx: &mut druid::LifeCycleCtx,
            event: &druid::LifeCycle,
            data: &T,
            env: &druid::Env,
        ) {
        match event {
            druid::LifeCycle::FocusChanged(focused) => {
                if *focused {
                    tracing::debug!("Dialog body is focused");
                }
            }
            _ => ()
        }
        child.lifecycle(ctx, event, data, env);
    }
    fn update(&mut self, child: &mut W, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        child.update(ctx, old_data, data, env);
    }
}

#[derive(Data, Clone)]
struct VMDialogState {
    string: String,
}

struct VMDialogInputController {
    main_action: Vec<ActionPayload>,
}

impl<T, W: Widget<T>> Controller<T, W> for VMDialogInputController where T: Display {
    fn event(&mut self, child: &mut W, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut T, env: &druid::Env) {
        child.event(ctx, event, data, env);
        match event {
            Event::KeyDown(key_event) => {
                if let Key::Enter = key_event.key {
                    ctx.submit_command(
                        Command::new(
                            DIALOG_EXECUTE_ACTIONS,
                            vec![ActionPayload {
                                action: self.main_action[0].action.clone(),
                                string: Some(data.to_string()),
                                ..Default::default()
                            }],
                            Target::Auto
                        )
                    )
                } else if let Key::Escape = key_event.key {
                    ctx.submit_command(
                        Command::new(
                            DIALOG_EXECUTE_ACTIONS,
                            vec![ActionPayload { action: Action::NullAction, ..Default::default()}],
                            Target::Auto
                        )
                    )
                }
            },
            Event::Command(command) => {
                if command.is(SUBMIT_INPUT_ACTION) {
                    ctx.submit_command(
                        Command::new(
                            DIALOG_EXECUTE_ACTIONS,
                            vec![ActionPayload {
                                action: self.main_action[0].action.clone(),
                                string: Some(data.to_string()),
                                ..Default::default()
                            }],
                            Target::Auto
                        )
                    )
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        match event {
            druid::LifeCycle::FocusChanged(focused) => {
                if *focused {
                    tracing::debug!("VMDialog input is focused");
                }
            }
            _ => {

            }
        }
        child.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, child: &mut W, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        child.update(ctx, old_data, data, env);
    }
}

impl VMDialog {
    pub fn as_dialog(config: &VMConfigVersion4, params: VMDialogParams) -> VMDialog {
        let mut main_column = Flex::column();
        main_column.add_default_spacer();
        for (label, color) in &params.prompts {
            main_column.add_child(
                Label::<String>::new(
                    label.clone()
                )
                .with_text_color(
                    if let Some(color) = color {
                        config.get_color((*color).clone()).expect("Couldn't get custom label color from config")
                    } else {
                        config.get_color(VMColor::LabelTextColor).expect("Couldn't get label text color from config")
                    }
                )
            );
            main_column.add_default_spacer();
        }

        main_column.add_spacer(DIALOG_LABEL_BUTTON_SPACER);

        let mut button_row = Flex::<String>::row();

        if params.buttons.len() > 0 {
            for i in 0..&params.buttons.len()-1 {
                let params = params.clone();
                let (label, payloads, is_alert) = params.buttons[i].clone();
                button_row.add_child(
                    VMButton::<String>::new(
                        config,
                        label.clone(),
                    move |ctx| {
                        ctx.submit_command(
                            Command::new(
                                DIALOG_EXECUTE_ACTIONS,
                                payloads.clone(),
                                Target::Auto
                            )
                        )
                    }, is_alert).controller(VMDialogController)
                );
                button_row.add_default_spacer();
            }
            let idx = params.buttons.len()-1;
            let (label, payloads, is_alert) = params.buttons[idx].clone();
            button_row.add_child(
                VMButton::<String>::new(
                    config,
                    label.clone(),
                move |ctx| {
                    ctx.submit_command(
                        Command::new(
                            DIALOG_EXECUTE_ACTIONS,
                            payloads.clone(),
                            Target::Auto
                        )
                    )
                }, is_alert).controller(VMDialogController)
            );
        }

        main_column.add_child(button_row);
        main_column.add_default_spacer();

        let pod = WidgetPod::new(
            Flex::column()
                .with_child(
                    SizedBox::new(
                        main_column.main_axis_alignment(MainAxisAlignment::Center)
                    )
                    .padding(5.)
                    .border(config.get_color(VMColor::NodeBorderColor).expect("Couldn't get node border color from config")
                        , DEFAULT_BORDER_WIDTH)
                    .rounded(DEFAULT_BORDER_RADIUS)
                    .background(config.get_color(VMColor::NodeBackgroundColor).expect("Couldn't get node background color from config"))
                ).main_axis_alignment(MainAxisAlignment::Center));
        VMDialog {
            inner: pod,
        }
    }

    pub fn as_input(config: &VMConfigVersion4, params: VMInputParams) -> VMDialog {
        let mut main_column = Flex::column();
        main_column.add_default_spacer();
        for (label, color) in &params.prompts {
            main_column.add_child(
                Label::<String>::new(
                    label.clone()
                )
                .with_text_color(
                    if let Some(color) = color {
                        config.get_color((*color).clone()).expect("Couldn't get custom label color from config")
                    } else {
                        config.get_color(VMColor::LabelTextColor).expect("Couldn't get label text color from config")
                    }
                )
            );
            main_column.add_default_spacer();
        }

        main_column.add_default_spacer();        

        let mut button_row = Flex::<String>::row();
        button_row.add_child(
            VMButton::<String>::new(
                config,
                params.button.0.clone(),
            move |ctx| {
                ctx.submit_command(
                    Command::new(
                        SUBMIT_INPUT_ACTION,
                        (),
                        Target::Auto
                    )
                )
            }, 
            params.clone().button.1
        ).controller(VMDialogController));

        button_row.add_default_spacer();

        button_row.add_child(
            VMButton::<String>::new(
                config,
                String::from("Cancel"),
            move |ctx| {
                ctx.submit_command(
                    Command::new(
                        DIALOG_EXECUTE_ACTIONS,
                        vec![ActionPayload {
                            action: Action::NullAction,
                            ..Default::default()
                        }],
                        Target::Auto
                    )
                )
            }, params.clone().button.1).controller(VMDialogController)
        );

        let text_box = TextBox::<String>::new().controller(VMDialogInputController {main_action: params.clone().input_actions});

        main_column.add_child(text_box);
        main_column.add_default_spacer();

        main_column.add_child(button_row);
        main_column.add_default_spacer();

        let pod = WidgetPod::new(
            Flex::column()
                .with_child(
                    SizedBox::new(
                        main_column.main_axis_alignment(MainAxisAlignment::Center)
                    )
                    .padding(5.)
                    .border(config.get_color(VMColor::NodeBorderColor).expect("Couldn't get node border color from config")
                        , DEFAULT_BORDER_WIDTH)
                    .rounded(DEFAULT_BORDER_RADIUS)
                    .background(config.get_color(VMColor::NodeBackgroundColor).expect("Couldn't get node background color from config"))
                ).main_axis_alignment(MainAxisAlignment::Center));
        VMDialog {
            inner: pod,
        }
    }

    pub fn make_start_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                ("Do you want create a new sheet or load an existing one?".to_string(), None)
            ],
            buttons: vec![
                (
                    String::from("New"),
                    vec![ActionPayload {
                        action: Action::CreateNewSheet,
                        ..Default::default()
                    }],
                    false
                ),
                (
                    String::from("Open"),
                    vec![ActionPayload {
                        action: Action::OpenExistingSheet,
                        ..Default::default()
                    }],
                    false
                )
            ],
            // input: None
        }
    }

    pub fn make_save_and_quit_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This sheet has unsaved changes, do you want to save before closing this sheet?"), None)
            ],
            buttons: vec![
                (
                    String::from("Save and Quit"),
                    vec![
                        ActionPayload {
                            action: Action::SaveSheet,
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::Saved),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::QuitWithoutSaveGuard,
                            ..Default::default()
                        }
                    ],
                    false
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::QuitWithoutSaveGuard,
                            ..Default::default()
                        }
                    ],
                    true
                ),
            ],
            // input: None
        }
    }

    pub fn make_save_and_new_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This sheet has unsaved changes, do you want to save before closing this sheet?"), None)
            ],
            buttons: vec![
                (
                    String::from("Save"),
                    vec![
                        ActionPayload {
                            action: Action::SaveSheet,
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::Saved),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        }
                    ], false
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::DiscardChanges),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        }
                    ],
                    true
                ),
            ],
            // input: None
        }
    }

    pub fn make_save_and_open_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This sheet has unsaved changes, do you want to save before closing this sheet?"), None)
            ],
            buttons: vec![
                (
                    String::from("Save"),
                    vec![
                        ActionPayload {
                            action: Action::SaveSheet,
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::Saved),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::OpenExistingSheet,
                            ..Default::default()
                        }
                    ],
                    false
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            // save_state: Some(VMSaveState::Saved),
                            save_state: Some(VMSaveState::DiscardChanges),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::OpenExistingSheet,
                            ..Default::default()
                        }
                    ],
                    true
                ),
            ],
            // input: None
        }
    }

    pub fn make_save_as_and_new_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This sheet is unsaved, do you want to save before closing this sheet?"), None),
                (String::from("Existing files will be overwritten!"), Some(VMColor::AlertColor))
            ],
            buttons: vec![
                (
                    String::from("Save As..."),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::SaveAsInProgressThenNew),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::SaveSheetAs,
                            ..Default::default()
                        },
                    ],
                    false
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::DiscardChanges),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        }
                    ],
                    true
                ),
            ],
            // input: None
        }
    }

    pub fn make_save_as_and_open_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This sheet is unsaved, do you want to save before closing this sheet?"), None),
                (String::from("Existing files will be overwritten!"), Some(VMColor::AlertColor))
            ],
            buttons: vec![
                (
                    String::from("Save As..."),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::SaveAsInProgressThenOpen),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::SaveSheetAs,
                            ..Default::default()
                        },
                    ],
                    false
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::DiscardChanges),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::OpenExistingSheet,
                            ..Default::default()
                        }
                    ],
                    true
                ),
            ],
            // input: None
        }
    }

    pub fn make_save_as_and_quit_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This sheet is unsaved, do you want to save before closing this sheet?"), None),
                (String::from("Existing files will be overwritten!"), Some(VMColor::AlertColor))
            ],
            buttons: vec![
                (
                    String::from("Save As..."),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::SaveAsInProgressThenQuit),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::SaveSheetAs,
                            ..Default::default()
                        },
                    ],
                    false
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::DiscardChanges),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::QuitWithoutSaveGuard,
                            ..Default::default()
                        }
                    ],
                    true
                ),
            ],
            // input: None
        }
    }

    #[allow(dead_code)]
    pub fn make_overwrite_prompt_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompts: vec![
                (String::from("This file already exists. Do you want to overwrite?"), None)
            ],
            buttons: vec![
                (
                    String::from("Overwrite"),
                    vec![
                        ActionPayload {
                            action: Action::SaveSheetAsOverwrite,
                            ..Default::default()
                        },
                    ],
                    true
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }],
                    false
                ),
            ],
            // input: None
        }
    }

    pub fn make_delete_node_prompt_dialog_params(count: usize, remove_idx: u32) -> VMDialogParams {
        VMDialogParams {
            buttons: vec![
                (
                    String::from("Cancel"),
                    vec![
                        ActionPayload {
                            action: Action::NullAction,
                            ..Default::default()
                        }
                    ],
                    false
                ),
                (
                    format!("Delete {} nodes", count),
                    vec![
                        ActionPayload {
                            action: Action::DeleteNodeTree,
                            index: Some(remove_idx),
                            ..Default::default()
                        }
                    ],
                    true
                )
            ],
            prompts: vec![
                (
                    format!("Do you want to delete this node and {} descendants?", count-1),
                    Some(VMColor::AlertColor)
                )
            ],
        }
    }

    pub fn make_delete_tab_prompt_dialog_params() -> VMDialogParams {
        VMDialogParams {
            buttons: vec![
                (
                    String::from("Cancel"),
                    vec![
                        ActionPayload {
                            action: Action::NullAction,
                            ..Default::default()
                        }
                    ],
                    false
                ),
                (
                    format!("Delete tab"),
                    vec![
                        ActionPayload {
                            action: Action::DeleteTab,
                            ..Default::default()
                        }
                    ],
                    true
                )
            ],
            prompts: vec![
                (
                    format!("Do you want to permenantly delete this tab?"),
                    None,
                ),
                (
                    format!("WARNING: This action cannot be undone!"),
                    Some(VMColor::AlertColor)
                )
            ],
        }
    }

    pub fn make_new_tab_prompt_input_params() -> VMInputParams {
        VMInputParams {
            prompts: vec![(String::from("What do you want this tab to be called?"), None)],
            button: (String::from("Ok"), false),
            input_actions: vec![
                ActionPayload {
                    action: Action::CreateNewTab,
                    ..Default::default()
                }
            ],
        }
    }

    pub fn make_rename_tab_prompt_input_params() -> VMInputParams {
        VMInputParams {
            prompts: vec![(String::from("What do you want this tab to be called?"), None)],
            button: (String::from("Ok"), false),
            input_actions: vec![
                ActionPayload {
                    action: Action::RenameTab,
                    ..Default::default()
                }
            ],
        }
    }
}