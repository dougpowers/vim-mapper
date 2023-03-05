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

use druid::{widget::{Flex, SizedBox, Label, MainAxisAlignment, Controller}, WidgetExt, Command, Target, WidgetPod, Widget, RawMods};

use crate::{vmconfig::{VMConfigVersion4, VMColor}, vminput::{Action, ActionPayload}, vmsave::VMSaveState};

use crate::constants::*;

use crate::vmbutton::VMButton;

pub struct VMDialog {
    pub(crate) inner: WidgetPod<(), Flex<()>>
}

#[derive(Debug, Clone)]
pub struct VMDialogParams {
    pub buttons: Vec<(String, Vec<ActionPayload>, bool)>,
    pub prompts: Vec<(String, Option<VMColor>)>
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
        child.lifecycle(ctx, event, data, env);
    }
}

impl VMDialog {
    pub fn new(config: &VMConfigVersion4, params: VMDialogParams) -> VMDialog {
        let mut main_column = Flex::column();
        main_column.add_default_spacer();
        for (label, color) in &params.prompts {
            main_column.add_child(
                Label::new(
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
        let mut button_row = Flex::<()>::row();
        for i in 0..&params.buttons.len()-1 {
            let params = params.clone();
            let (label, payloads, is_alert) = params.buttons[i].clone();
            button_row.add_child(
                VMButton::new(
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
            VMButton::new(
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
            ]
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
            ]
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
            ]
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
            ]
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
            ]
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
            ]
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
            ]
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
            ]
        }
    }
}