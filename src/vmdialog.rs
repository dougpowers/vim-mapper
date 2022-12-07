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

use std::borrow::Borrow;

use druid::{widget::{Button, Flex, SizedBox, Label, MainAxisAlignment, Controller}, WidgetExt, Command, Target, WidgetPod, Widget};

use crate::{vmconfig::{VMConfigVersion4, VMColor}, vminput::{Action, ActionPayload}, vmsave::VMSaveState};

use crate::constants::*;

pub struct VMDialog {
    pub(crate) inner: WidgetPod<(), Flex<()>>
}

#[derive(Debug, Clone)]
pub struct VMDialogParams {
    pub buttons: Vec<(String, Vec<ActionPayload>)>,
    pub prompt: String,
}

struct VMDialogController;

impl<T, W: Widget<T>> Controller<T, W> for VMDialogController {
    fn event(&mut self, child: &mut W, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut T, env: &druid::Env) {
        // if let druid::Event::KeyDown(key_event) = event {
        if ctx.has_focus() {
            // tracing::debug!("{:?} - {:?}", ctx.widget_id(), event);
        }
        // }
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
        match event.borrow() {
            druid::LifeCycle::BuildFocusChain => {
                ctx.register_for_focus();
            }
            druid::LifeCycle::FocusChanged(is_focused) => {
                #[cfg(debug_assertions)]
                if *is_focused {
                    tracing::debug!("{:?} is now focused", ctx.widget_id());
                    self.lifecycle(child, ctx, 
                    &druid::LifeCycle::HotChanged {
                        0: true,
                    }, 
                    data, env);
                    ctx.request_paint();
                } else {
                    tracing::debug!("{:?} is now NOT focused", ctx.widget_id());
                    self.lifecycle(child, ctx, 
                    &druid::LifeCycle::HotChanged {
                        0: false,
                    }, 
                    data, env);
                    ctx.request_paint();
                }
            }
            druid::LifeCycle::HotChanged(is_hot) => {
                #[cfg(debug_assertions)]
                if *is_hot {
                    tracing::debug!("{:?} is now hot", ctx.widget_id());
                } else {
                    tracing::debug!("{:?} is now NOT hot", ctx.widget_id());
                }
                ctx.request_paint();
            }
            _ => ()
        }
        child.lifecycle(ctx, event, data, env);
    }

}

impl VMDialog {
    pub fn new(config: &VMConfigVersion4, params: VMDialogParams) -> VMDialog {
        let mut main_column = Flex::column()
            .with_child(
                Label::new(
                    params.prompt.clone()
                )
                .with_text_color(config.get_color(VMColor::LabelTextColor).expect("Couldn't get label text color from config"))
            ).with_child(SizedBox::empty().height(50.));
            let mut button_row = Flex::<()>::row();
            for i in 0..params.buttons.len()-1 {
                let params = params.clone();
                let (label, payloads) = params.buttons[i].clone();
                button_row.add_child(Button::new(label.clone())
                    .controller(VMDialogController)
                    .on_click(move |ctx, _, _| {
                        ctx.submit_command(
                            Command::new(
                                DIALOG_EXECUTE_ACTIONS,
                                payloads.clone(),
                                Target::Auto
                            )
                        )
                    })
                );
                button_row.add_default_spacer();
            }
            let idx = params.buttons.len()-1;
            let (label, payloads) = params.buttons[idx].clone();
            button_row.add_child(Button::new(label.clone())
                .controller(VMDialogController)
                .on_click(move |ctx, _, _| {
                    ctx.submit_command(
                        Command::new(
                            DIALOG_EXECUTE_ACTIONS,
                            payloads.clone(),
                            Target::Auto
                        )
                    )
                })
            );

            main_column.add_child(button_row);

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
            prompt: "Do you want create a new sheet or load an existing one?".to_string(),
            buttons: vec![
                (
                    String::from("New"),
                    vec![ActionPayload {
                        action: Action::CreateNewSheet,
                        ..Default::default()
                    }]
                ),
                (
                    String::from("Open"),
                    vec![ActionPayload {
                        action: Action::OpenExistingSheet,
                        ..Default::default()
                    }]
                )
            ]
        }
    }

    pub fn make_save_and_quit_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompt: String::from("This sheet has unsaved changes, do you want to save before closing this sheet?"),
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
                    ]
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }]
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::QuitWithoutSaveGuard,
                            ..Default::default()
                        }
                    ]
                ),
            ]
        }
    }

    pub fn make_save_and_new_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompt: String::from("This sheet has unsaved changes, do you want to save before closing this sheet?"),
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
                    ]
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }]
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::Saved),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        }
                    ]
                ),
            ]
        }
    }

    pub fn make_save_as_and_new_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompt: String::from("This sheet is unsaved, do you want to save before closing this sheet?"),
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
                    ]
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }]
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::Saved),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::CreateNewSheet,
                            ..Default::default()
                        }
                    ]
                ),
            ]
        }
    }

    pub fn make_save_as_and_quit_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompt: String::from("This sheet is unsaved, do you want to save before closing this sheet?"),
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
                    ]
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }]
                ),
                (
                    String::from("Discard Changes"),
                    vec![
                        ActionPayload {
                            action: Action::SetSaveState,
                            save_state: Some(VMSaveState::Saved),
                            ..Default::default()
                        },
                        ActionPayload {
                            action: Action::QuitWithoutSaveGuard,
                            ..Default::default()
                        }
                    ]
                ),
            ]
        }
    }

    pub fn make_overwrite_prompt_dialog_params() -> VMDialogParams {
        VMDialogParams {
            prompt: String::from("This file already exists. Do you want to overwrite?"),
            buttons: vec![
                (
                    String::from("Overwrite"),
                    vec![
                        ActionPayload {
                            action: Action::SaveSheetAsOverwrite,
                            ..Default::default()
                        },
                    ]
                ),
                (
                    String::from("Cancel"),
                    vec![ActionPayload { action: Action::NullAction, ..Default::default() }]
                ),
            ]
        }
    }
}