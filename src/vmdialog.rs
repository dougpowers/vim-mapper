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

use druid::{widget::{Button, ControllerHost, Flex, SizedBox, Label, MainAxisAlignment}, WidgetExt, Command, FileDialogOptions, Target, WidgetPod, Notification};

use crate::{vmconfig::{VMConfigVersion4, VMColor}, vminput::ActionPayload};

use crate::constants::*;

pub struct VMDialog {
    pub(crate) inner: WidgetPod<(), Flex<()>>
}

#[derive(Debug, Clone)]
pub struct VMDialogParams {
    pub buttons: Vec<(String, Vec<ActionPayload>)>,
    pub prompt: String,
}

impl VMDialog {
    pub fn new(config: &VMConfigVersion4, params: VMDialogParams) -> VMDialog {
        // let open_button = Button::new("Open...")
        //     .on_click(move |ctx, _, _| {
        //     ctx.submit_command(
        //         Command::new(
        //             druid::commands::SHOW_OPEN_PANEL,
        //             FileDialogOptions::new(),
        //             Target::Auto
        //         )
        //     )
        // });
        // let new_button: ControllerHost<Button<()>, druid::widget::Click<_>> = Button::new("New")
        //     .on_click(move |ctx, _, _| {
        //     ctx.submit_command(
        //         Command::new(
        //             druid::commands::NEW_FILE,
        //             (),
        //             Target::Auto
        //         )
        //     )
        // });

        let mut main_column = Flex::column()
            .with_child(
                Label::new(
                    "Do you want create a new sheet or load an existing one?"
                )
                .with_text_color(config.get_color(VMColor::LabelTextColor).expect("Couldn't get label text color from config"))
            ).with_child(SizedBox::empty().height(50.));
            let mut button_row = Flex::<()>::row();
            for i in 0..params.buttons.len()-1 {
                let params = params.clone();
                let (label, payloads) = params.buttons[i].clone();
                button_row.add_child(Button::new(label.clone())
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
                        // Flex::column()
                        // .with_child(
                        //     Label::new(
                        //         "Do you want create a new sheet or load an existing one?"
                        //     )
                        //     .with_text_color(config.get_color(VMColor::LabelTextColor).expect("Couldn't get label text color from config"))
                        //     )
                        // .with_child(SizedBox::empty().height(50.))
                        // .with_child(
                        //     Flex::row().with_child(
                        //         new_button
                        //     ).with_default_spacer()
                        //     .with_child(
                        //         open_button
                        //     )   
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
}