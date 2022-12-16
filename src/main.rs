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

#![windows_subsystem = "windows"]
use druid::widget::{prelude::*, Flex};
use druid::{AppLauncher, WindowDesc, FileDialogOptions, Point, WindowState, Command, Target, WidgetPod, LocalizedString, MenuItem, FileSpec, FontFamily, WindowId, Menu, AppDelegate};
use druid::piet::{Text, TextLayout, TextLayoutBuilder};
use vmdialog::{VMDialogParams, VMDialog};
use std::fs;
use std::path::{PathBuf, Path};
use std::time::{SystemTime, UNIX_EPOCH};


mod vmnode;

mod vminput;
use vminput::*;

mod constants;
use constants::*;

mod vimmapper;
use vimmapper::*;

mod vmconfig;
use vmconfig::*;

mod vmsave;
use vmsave::*;

mod vmdialog;

mod vmbutton;

struct VMCanvas {
    inner: Option<WidgetPod<(), VimMapper>>,
    dialog: WidgetPod<(), Flex<()>>,
    dialog_visible: bool,
    path: Option<PathBuf>,
    config: VMConfigVersion4,
    input_manager: VMInputManager,
    last_frame_time: u128,
}

impl VMCanvas {
    pub fn new(config: VMConfigVersion4) -> VMCanvas {
        VMCanvas {
            inner: None,
            // dialog: VMCanvas::make_dialog(&config),
            dialog: VMCanvas::new_dialog(&config, VMDialog::make_start_dialog_params()),
            dialog_visible: true,
            path: None,
            config,
            input_manager: VMInputManager::new(),
            last_frame_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        }
    }

    pub fn set_path(&mut self, path: PathBuf) -> Result<PathBuf, String> {
        if path.is_file() {
            self.path = Some(path.clone());
            return Ok(path.clone());
        } else {
            self.path = Some(path.clone());
            if let Ok(_) = fs::write(&path, "test") {
                self.path = Some(path.clone());
                return Ok(path.clone());
            } else {
                return Err(String::from("Path is not accessible!"));
            }
        }
    }

    pub fn load_new_mapper(&mut self, mapper: VimMapper) {
        self.inner = Some(WidgetPod::new(mapper));
        self.hide_dialog();
    }

    fn new_dialog(config: &VMConfigVersion4, params: VMDialogParams) -> WidgetPod<(), Flex<()>> {
        let dialog = VMDialog::new(config, params);
        dialog.inner
    }

    fn hide_dialog(&mut self) {
        self.dialog_visible = false;
    }

    fn set_dialog(&mut self, ctx: &mut EventCtx, params: VMDialogParams, show: bool) {
        self.dialog = VMCanvas::new_dialog(&self.config, params);
        ctx.children_changed();
        ctx.request_layout();
        ctx.request_paint();
        ctx.set_handled();
        self.dialog_visible = show;
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, data: &mut AppState, payload: &Option<ActionPayload>) -> Result<(), ()> {
        if let Some(payload) = payload {
            if let Some(_) = self.path {
                match payload.action {
                    Action::CreateNewNode => data.save_state = VMSaveState::UnsavedChanges,
                    Action::CreateNewNodeAndEdit => data.save_state = VMSaveState::UnsavedChanges,
                    Action::ActivateTargetedNode => data.save_state = VMSaveState::UnsavedChanges,
                    Action::IncreaseActiveNodeMass => data.save_state = VMSaveState::UnsavedChanges,
                    Action::DecreaseActiveNodeMass => data.save_state = VMSaveState::UnsavedChanges,
                    Action::ResetActiveNodeMass => data.save_state = VMSaveState::UnsavedChanges,
                    Action::ToggleAnchorActiveNode => data.save_state = VMSaveState::UnsavedChanges,
                    Action::EditActiveNodeSelectAll => data.save_state = VMSaveState::UnsavedChanges,
                    Action::EditActiveNodeAppend => data.save_state = VMSaveState::UnsavedChanges,
                    Action::EditActiveNodeInsert => data.save_state = VMSaveState::UnsavedChanges,
                    Action::DeleteActiveNode => data.save_state = VMSaveState::UnsavedChanges,
                    Action::DeleteTargetNode => data.save_state = VMSaveState::UnsavedChanges,
                    Action::MarkActiveNode => data.save_state = VMSaveState::UnsavedChanges,
                    Action::MoveActiveNodeDown => data.save_state = VMSaveState::UnsavedChanges,
                    Action::MoveActiveNodeUp => data.save_state = VMSaveState::UnsavedChanges,
                    Action::MoveActiveNodeLeft => data.save_state = VMSaveState::UnsavedChanges,
                    Action::MoveActiveNodeRight => data.save_state = VMSaveState::UnsavedChanges,
                    Action::PanUp => data.save_state = VMSaveState::UnsavedChanges,
                    Action::PanDown => data.save_state = VMSaveState::UnsavedChanges,
                    Action::PanLeft => data.save_state = VMSaveState::UnsavedChanges,
                    Action::PanRight => data.save_state = VMSaveState::UnsavedChanges,
                    Action::ZoomOut => data.save_state = VMSaveState::UnsavedChanges,
                    Action::ZoomIn => data.save_state = VMSaveState::UnsavedChanges,
                    _ => ()
                }
            }

            match payload.action {
                Action::PrintToLogInfo => {
                    tracing::debug!("{:?}", payload.string);
                    ctx.set_handled();
                }
                Action::ToggleColorScheme => {
                    self.config.toggle_color_scheme();
                    let result = VMConfigSerde::save(&self.config);
                    if let Err(reason) = result {
                        panic!("Application panicked on config save: {}", reason);
                    } 
                    if let Some(vm) = &mut self.inner {
                        vm.widget_mut().set_config(self.config.clone());
                        ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                    }
                    // self.dialog = VMCanvas::make_dialog(&self.config);
                    // self.dialog = VMCanvas::new_dialog(&self.config, VMDialog::make_start_dialog_params());
                    // ctx.children_changed();
                    // ctx.request_layout();
                    // ctx.request_paint();
                    // ctx.set_handled();
                    self.set_dialog(ctx, VMDialog::make_start_dialog_params(), self.dialog_visible);
                    return Ok(());
                }
                Action::OpenExistingSheet => {
                    if data.save_state == VMSaveState::Saved || data.save_state == VMSaveState::NoSheetOpened || data.save_state == VMSaveState::DiscardChanges {
                        ctx.submit_command(
                            Command::new(
                                druid::commands::SHOW_OPEN_PANEL,
                                VMCanvas::make_open_panel_options(),
                                Target::Auto,
                            )
                        );
                        ctx.set_handled();
                        return Ok(());
                    } else if data.save_state == VMSaveState::NoSave {
                        // self.dialog = VMCanvas::new_dialog(&self.config, VMDialog::make_save_as_and_open_dialog_params());
                        // ctx.children_changed();
                        // self.dialog_visible = true;
                        // ctx.set_handled();
                        self.set_dialog(ctx, VMDialog::make_save_as_and_open_dialog_params(), true);
                        return Ok(());
                    } else {
                        // self.dialog = VMCanvas::new_dialog(&self.config, VMDialog::make_save_and_open_dialog_params());
                        // ctx.children_changed();
                        // self.dialog_visible = true;
                        // ctx.set_handled();
                        self.set_dialog(ctx, VMDialog::make_save_and_open_dialog_params(), true);
                        return Ok(());
                    }
                }
                Action::CreateNewSheet => {
                    if data.save_state == VMSaveState::Saved || data.save_state == VMSaveState::NoSheetOpened || data.save_state == VMSaveState::DiscardChanges {
                        self.load_new_mapper(VimMapper::new(self.config.clone()));
                        self.path = None;
                        data.save_state = VMSaveState::NoSave;
                        ctx.children_changed();
                        ctx.request_layout();
                        ctx.set_handled();
                        return Ok(());
                    } else if data.save_state == VMSaveState::NoSave {
                        // self.dialog = VMCanvas::new_dialog(&self.config, VMDialog::make_save_as_and_new_dialog_params());
                        // ctx.children_changed();
                        // self.dialog_visible = true;
                        // ctx.set_handled();
                        self.set_dialog(ctx, VMDialog::make_save_as_and_new_dialog_params(), true);
                        return Ok(());
                    } else {
                        // self.dialog = VMCanvas::new_dialog(&self.config, VMDialog::make_save_and_new_dialog_params());
                        // ctx.children_changed();
                        // self.dialog_visible = true;
                        // ctx.set_handled();
                        self.set_dialog(ctx, VMDialog::make_save_and_new_dialog_params(), true);
                        return Ok(());
                    }
                }
                Action::ToggleMenuVisible => {
                    data.menu_visible = !data.menu_visible;
                    ctx.set_handled();
                    return Ok(());
                }
                Action::CreateDialog => {
                    // self.dialog = VMDialog::new(&self.config, 
                    //     payload.dialog_params.clone().unwrap()
                    // ).inner;
                    // self.dialog_visible = true;
                    // ctx.set_handled();
                    self.set_dialog(ctx, payload.dialog_params.clone().unwrap(), true);
                    return Ok(());
                }
                Action::SaveSheet => { 
                    if let Some(inner) = &self.inner {
                        if let Some(path) = self.path.clone() {
                            if let Ok(_) = VMSaveSerde::save(
                                &VMSaveSerde::to_save(inner.widget()),
                                path,
                            ) {
                                    data.save_state = VMSaveState::Saved;
                            }
                        } else {
                            ctx.submit_command(
                                Command::new(
                                    EXECUTE_ACTION,
                                    ActionPayload {
                                        action: Action::SaveSheetAs,
                                        ..Default::default()
                                    },
                                    Target::Global,
                                )
                            );
                        }
                    }
                }
                Action::SaveSheetAs => {
                    match data.save_state {
                        VMSaveState::NoSheetOpened => (),
                        VMSaveState::NoSave | VMSaveState::UnsavedChanges => {
                            data.save_state = VMSaveState::SaveAsInProgress;
                            ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(VMCanvas::make_save_panel_options()));
                            ctx.set_handled();
                        },
                        _ => {
                            ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(VMCanvas::make_save_panel_options()));
                            ctx.set_handled();
                        },
                    }
                }
                // Action::SaveSheetAsOverwrite => {
                //     if let Some(inner) = &self.inner {
                //         if let Some(path) = self.path.clone() {
                //             if let Ok(_) = VMSaveSerde::save(
                //                 &VMSaveSerde::to_save(inner.widget()),
                //                 path,
                //             ) {
                //                     data.save_state = VMSaveState::Saved;
                //             }
                //         }
                //     }
                // }
                Action::QuitWithoutSaveGuard => {
                    ctx.submit_command(druid::commands::QUIT_APP);
                }
                Action::SetSaveState => {
                    data.save_state = payload.save_state.clone().unwrap();
                }
                _ => ()
            }
        }
        if let Some(inner) = &mut self.inner {
            if let Some(payload) = payload {
                if payload.action != Action::ChangeModeWithTimeoutRevert {
                    self.input_manager.clear_timeout();
                }
                match payload.action {
                    Action::CreateNewNode => {
                        if let Some(idx) = inner.widget().get_active_node_idx() {
                            if let Some(_) = inner.widget_mut().add_node(idx, format!("New Node"), None) {
                                ctx.submit_command(Command::new(REFRESH, (), Target::Widget(inner.id())));
                            }
                        }
                        return Ok(());
                    },
                    Action::CreateNewNodeAndEdit => {
                        if let Some(idx) = inner.widget().get_active_node_idx() {
                            if let Some(new_idx) = inner.widget_mut().add_node(idx, format!("New Node"), None) {
                                self.input_manager.set_keybind_mode(KeybindMode::EditBrowse);
                                inner.widget_mut().open_editor(ctx, new_idx);
                                ctx.submit_command(Command::new(REFRESH, (), Target::Widget(inner.id())));
                                ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Widget(inner.id())));
                            }
                        }
                        return Ok(());
                    },
                    Action::CreateNewExternalNode => {
                        if let Some(_) = inner.widget().get_active_node_idx() {
                            if let Some(new_idx) = inner.widget_mut().add_external_node(format!("New External Node")) {
                                ctx.submit_command(Command::new(REFRESH, (), Target::Widget(inner.id())));
                                inner.widget_mut().set_node_as_active(new_idx);
                                ctx.submit_command(Command::new(
                                    EXECUTE_ACTION,
                                    ActionPayload {
                                        action: Action::ChangeMode,
                                        mode: Some(KeybindMode::Move),
                                        ..Default::default()
                                    },
                                    Target::Global
                                ));
                            }
                        }
                        return Ok(());
                    },
                    Action::EditActiveNodeSelectAll => {
                        if let Some(idx) = inner.widget().get_active_node_idx() {
                            self.input_manager.set_keybind_mode(KeybindMode::EditBrowse);
                            inner.widget_mut().open_editor(ctx, idx);
                            ctx.submit_command(Command::new(REFRESH, (), Target::Widget(inner.id())));
                            ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Widget(inner.id())));
                        }
                        return Ok(());
                    }
                    Action::ChangeModeWithTimeoutRevert => {
                        self.input_manager.set_timeout_revert_mode(Some(self.input_manager.get_keybind_mode()));
                        self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                        match payload.mode {
                            Some(KeybindMode::SearchBuild) | Some(KeybindMode::SearchedSheet) => {
                                inner.widget_mut().set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                            }
                            _ => {
                                inner.widget_mut().set_render_mode(NodeRenderMode::AllEnabled);
                            }
                        }
                        ctx.submit_command(Command::new(REFRESH, (), Target::Widget(inner.id())));
                        return Ok(());
                    },
                    Action::ChangeMode => {
                        match payload.mode {
                            Some(KeybindMode::Move) => {
                                if let Some(active_idx) = inner.widget().get_active_node_idx() {
                                    if active_idx == 0 {
                                        ()
                                    } else {
                                        self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                    }
                                }
                            }
                            Some(KeybindMode::SearchBuild) => {
                                self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                inner.widget_mut().set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                            },
                            Some(KeybindMode::SearchedSheet) => {
                                self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                if inner.widget().get_target_list_length() == 1 {
                                    ctx.submit_command(EXECUTE_ACTION.with(
                                        ActionPayload {
                                            action: Action::ActivateTargetedNode,
                                            ..Default::default()
                                        }
                                    ));
                                    self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                                } else if inner.widget().get_target_list_length() == 0 {
                                    let idx = if let Some(idx) = inner.widget().get_active_node_idx() {
                                            idx
                                        } else {
                                            0
                                        };
                                    self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                                    inner.widget_mut().set_render_mode(NodeRenderMode::AllEnabled);
                                    inner.widget_mut().build_target_list_from_neighbors(idx);
                                    inner.widget_mut().cycle_target_forward();
                                }
                            }
                            _ => {
                                self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                // if let Some(inner) = &mut self.inner {
                                    inner.widget_mut().set_render_mode(NodeRenderMode::AllEnabled);
                                // }
                            }
                        }
                        ctx.submit_command(Command::new(REFRESH, (), Target::Widget(inner.id())));
                        return Ok(());
                    }
                    _ => {
                        if let Some(inner) = &self.inner {
                            if !inner.widget().is_editor_open() {
                                ctx.submit_command(Command::new(EXECUTE_ACTION, payload.clone(), Target::Widget(inner.id())));
                            }
                        }
                        return Ok(());
                    }
                } 
            } else {
                Err(())
            }
        } else {
            Err(())
        }

    }

    fn make_open_panel_options() -> FileDialogOptions {
        let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])]);
        open_dialog_options
    }

    fn make_save_panel_options() -> FileDialogOptions {
        let save_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])])
        .default_type(FileSpec::new("VimMapper File", &["vmd"]))
        .default_name(DEFAULT_SAVE_NAME);
        save_dialog_options
    }

    pub fn make_menu(_id: Option<WindowId>, data: &AppState, _env: &Env) -> Menu<AppState> {
        if data.menu_visible { 
            let base = Menu::<AppState>::empty();

            let file_menu = Menu::new(LocalizedString::new("file-menu").with_placeholder("File"))
            .entry(
                MenuItem::new(
                    String::from("New\tCtrl+N")
                )
                .command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::CreateNewSheet,
                        ..Default::default()
                    },
                    Target::Global
                ))
            )
            .entry(
                MenuItem::new(
                    String::from("Open File\tCtrl+O")
                )
                // .command(druid::commands::SHOW_OPEN_PANEL.with(VMCanvas::make_open_dialog_options()))
                .command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::OpenExistingSheet,
                        ..Default::default()
                    },
                    Target::Global,
                ))
            )
            .entry(
                MenuItem::new(
                    String::from("Save\tCtrl+S")
                )
                .command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::SaveSheet,
                        ..Default::default()
                    },
                    Target::Global,
                ))
            )
            .entry(
                MenuItem::new(
                    String::from("Save As...\tCtrl+Shift+S")
                )
                .command(druid::commands::SHOW_SAVE_PANEL.with(VMCanvas::make_save_panel_options()))
            )
            .separator()
            .entry(
                MenuItem::new(
                    String::from("Hide Menu\tAlt+F11")
                ).command(EXECUTE_ACTION.with(ActionPayload {
                    action: Action::ToggleMenuVisible,
                    ..Default::default()
                }))
            )
            .separator()
            .entry(
                MenuItem::new(
                    String::from("Exit\tAlt+F4")
                )
                .command(druid::commands::CLOSE_ALL_WINDOWS)
            );
            return base.entry(file_menu).rebuild_on(|old_data, data, _env| {
                old_data.menu_visible != data.menu_visible
            });
        } else {
            let base = Menu::<AppState>::empty();
            return base.rebuild_on(|old_data, data, _env| {
                old_data.menu_visible != data.menu_visible
            });
        }
    }
}

#[allow(unused_must_use)]
impl Widget<AppState> for VMCanvas {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        ctx.request_layout();
        ctx.request_paint();
        if let Some(_) = &self.inner {
        } else if !self.dialog_visible {
            // if ctx.is_hot() {
                ctx.request_focus();
            // }
        }
        match event {
            Event::Command(command) if command.is(druid::commands::NEW_FILE) => {
                tracing::error!("druid NEW_FILE called. this is an invalid command.");
                panic!();
            }
            Event::Command(command) if command.is(druid::commands::OPEN_FILE) => {
                let payload = command.get_unchecked(druid::commands::OPEN_FILE);
                if let Ok((save, path)) = VMSaveSerde::load(payload.path().to_str().unwrap().to_string()) {
                    let vm = VMSaveSerde::from_save(save, self.config.clone());
                    self.path = Some(path);
                    self.load_new_mapper(vm);
                    data.save_state = VMSaveState::Saved;
                    ctx.children_changed();
                }
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE) => {
                tracing::error!("druid::commands::SAVE_FILE command sent! This should not happen!");
                panic!();
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE_AS) => {
                if let Some(_) = self.inner {
                    let payload = command.get_unchecked(druid::commands::SAVE_FILE_AS);
                    let res = self.set_path(payload.path().to_path_buf());
                    if let Ok(path) = res {
                        match data.save_state {
                            VMSaveState::UnsavedChanges => {
                                data.save_state = VMSaveState::Saved;
                                VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                            },
                            VMSaveState::SaveAsInProgress => {
                                data.save_state = VMSaveState::Saved;
                                VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                            },
                            VMSaveState::SaveAsInProgressThenQuit => {
                                data.save_state = VMSaveState::Saved;
                                VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                                ctx.submit_command(Command::new(
                                    EXECUTE_ACTION,
                                    ActionPayload {
                                        action: Action::QuitWithoutSaveGuard,
                                        ..Default::default()
                                    },
                                    Target::Global
                                ))
                            },
                            VMSaveState::SaveAsInProgressThenNew => {
                                data.save_state = VMSaveState::Saved;
                                VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                                ctx.submit_command(Command::new(
                                    EXECUTE_ACTION,
                                    ActionPayload {
                                        action: Action::CreateNewSheet,
                                        ..Default::default()
                                    },
                                    Target::Global
                                ));
                            },
                            VMSaveState::SaveAsInProgressThenOpen => {
                                data.save_state = VMSaveState::Saved;
                                VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                                ctx.submit_command(Command::new(
                                    EXECUTE_ACTION,
                                    ActionPayload {
                                        action: Action::OpenExistingSheet,
                                        ..Default::default()
                                    },
                                    Target::Global
                                ));
                            },
                            VMSaveState::Saved => {
                                data.save_state = VMSaveState::Saved;
                                VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                            },
                            _ => {
                                tracing::error!("Tried to resolve SaveAs with an invalid save_state!");
                                panic!();
                            }
                        }
                    } else if let Err(err) = res {
                        panic!("{}", err);
                    }
                }
            }
            Event::Command(command) if command.is(druid::commands::OPEN_PANEL_CANCELLED) => {
                match data.save_state {
                    VMSaveState::NoSheetOpened => {
                        self.set_dialog(ctx, VMDialog::make_start_dialog_params(), true);
                    },
                    VMSaveState::DiscardChanges => {
                        if let Some(_) = self.path {
                            data.save_state = VMSaveState::UnsavedChanges;
                        } else {
                            data.save_state = VMSaveState::NoSave;
                        }
                    }
                    _ => ()
                }
            }
            Event::Command(command) if command.is(druid::commands::SAVE_PANEL_CANCELLED) => {
                match data.save_state {
                    VMSaveState::SaveAsInProgress => {
                        if self.path == None {
                            data.save_state = VMSaveState::NoSave;
                        } else {
                            data.save_state = VMSaveState::UnsavedChanges;
                        }
                    }
                    VMSaveState::SaveAsInProgressThenQuit => {
                        if self.path == None {
                            data.save_state = VMSaveState::NoSave;
                        } else {
                            data.save_state = VMSaveState::UnsavedChanges;
                        }
                    }
                    VMSaveState::SaveAsInProgressThenNew => {
                        if self.path == None {
                            data.save_state = VMSaveState::NoSave;
                        } else {
                            data.save_state = VMSaveState::UnsavedChanges;
                        }
                    }
                    _ => ()
                }
            }
            Event::WindowCloseRequested => {
                ctx.set_handled();
                match data.save_state {
                    VMSaveState::NoSheetOpened => ctx.submit_command(druid::commands::QUIT_APP),
                    VMSaveState::Saved => ctx.submit_command(druid::commands::QUIT_APP),
                    VMSaveState::DiscardChanges => ctx.submit_command(druid::commands::QUIT_APP),
                    VMSaveState::NoSave => {
                        // self.dialog = VMDialog::new(&self.config, VMDialog::make_save_as_and_quit_dialog_params()).inner;
                        // ctx.children_changed();
                        // self.dialog_visible = true;
                        self.set_dialog(ctx, VMDialog::make_save_as_and_quit_dialog_params(), true);
                    },
                    VMSaveState::UnsavedChanges => {
                        // self.dialog = VMCanvas::new_dialog(&self.config, VMDialog::make_save_and_quit_dialog_params());
                        // ctx.children_changed();
                        // self.dialog_visible = true;
                        self.set_dialog(ctx, VMDialog::make_save_and_quit_dialog_params(), true);
                    }
                    _ => ()
                }
            }
            Event::Command(command) if command.is(EXECUTE_ACTION) && !ctx.is_handled() => {
                let payload = command.get_unchecked(EXECUTE_ACTION);
                if let Ok(_) = self.handle_action(ctx, data, &Some(payload.to_owned())) {
                    ctx.set_handled();
                } else {
                    if let Some(inner) = &mut self.inner {
                        inner.event(ctx, event, &mut(), env);
                    } else {
                        self.dialog.event(ctx, event, &mut (), env);
                    }
                }
            }
            Event::Command(command) if command.is(DIALOG_EXECUTE_ACTIONS) && !ctx.is_handled() => {
                let payloads = command.get_unchecked(DIALOG_EXECUTE_ACTIONS);
                for payload in payloads {
                    self.handle_action(ctx, data, &Some((*payload).clone()));
                }
                self.hide_dialog();
            }
            Event::Notification(note) if note.is(SUBMIT_CHANGES) => {
                if let Some(inner) = &mut self.inner {
                    inner.widget_mut().close_editor(ctx, true);
                    self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                    inner.widget_mut().invalidate_node_layouts();
                    ctx.set_handled();
                    ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                }
            }
            Event::Notification(note) if note.is(CANCEL_CHANGES) => {
                if let Some(inner) = &mut self.inner {
                    inner.widget_mut().close_editor(ctx, false);
                    self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                    ctx.set_handled();
                    ctx.request_anim_frame();
                    ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                }
            }
            Event::KeyDown(key_event) => {
                let payloads = self.input_manager.accept_key(key_event.clone(), ctx);
                for payload in &payloads {
                    if self.dialog_visible 
                        && (key_event.key == druid::keyboard_types::Key::Tab ||
                        key_event.key == druid::keyboard_types::Key::Enter ||
                        key_event.key == druid::keyboard_types::Key::Character(String::from(" ")))
                        {
                        if key_event.key == druid::keyboard_types::Key::Tab {
                            ctx.focus_next();
                            ctx.set_handled();
                        }
                        self.dialog.event(ctx, event, &mut (), env);
                    } else {
                        if let Ok(_) = self.handle_action(ctx, data, payload) {

                        } else {
                            if let Some(inner) = &mut self.inner {
                                inner.event(ctx, event, &mut (), env);
                            }
                        }
                    } 
                }
            }
            Event::Timer(token) => {
                if Some(*token) == self.input_manager.get_timout_token() {
                    self.input_manager.timeout();
                } 
            }
            Event::WindowConnected => {
                ctx.request_focus();
            }
            _ => {
                if self.dialog_visible {
                    self.dialog.event(ctx, event, &mut (), env);
                } else if let Some(inner) = &mut self.inner {
                    inner.event(ctx, event, &mut (), env);
                }
            }
        }
        ctx.request_paint();
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &AppState, env: &Env) {
        if self.dialog_visible {
            self.dialog.lifecycle(ctx, event, &(), env);
        }
        if let Some(inner) = &mut self.inner {
            inner.lifecycle(ctx, event, &(), env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, env: &Env) {
        if self.dialog_visible {
            self.dialog.update(ctx, &(), env);
        } else if let Some(inner) = &mut self.inner {
            inner.update(ctx, &(), env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, env: &Env) -> Size {
        if self.dialog_visible {
            self.dialog.layout(ctx, bc, &(), env);
            self.dialog.set_origin(ctx, &(), env, Point::new(0., 0.));
        } 
        if let Some(inner) = &mut self.inner {
            inner.layout(ctx, bc, &(), env);
            inner.set_origin(ctx, &(), env, Point::new(0., 0.));
        }
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        let ctx_size = ctx.size();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if let Some(path) = &self.path {
                match data.save_state {
                    VMSaveState::UnsavedChanges => {
                        if path.display().to_string().starts_with(r"\\?") {
                            ctx.window().set_title(format!("Vim-Mapper - {}*", &(path.display().to_string())[4..]).as_str());
                        } else {
                            ctx.window().set_title(format!("Vim-Mapper - {}*", path.display()).as_str());
                        }
                    }
                    _ => {
                        if path.display().to_string().starts_with(r"\\?") {
                            ctx.window().set_title(format!("Vim-Mapper - {}", &(path.display().to_string())[4..]).as_str());
                        } else {
                            ctx.window().set_title(format!("Vim-Mapper - {}", path.display()).as_str());
                        }
                    }
                }
        } else {
            match data.save_state {
                VMSaveState::NoSheetOpened => {
                    ctx.window().set_title("Vim-Mapper");
                }
                VMSaveState::NoSave => {
                    ctx.window().set_title("Vim-Mapper - (unsaved sheet)");
                }
                _ => ()
            }
        }
        if let Some(inner) = &mut self.inner {
            inner.paint(ctx, &(), env);
            let layout = ctx.text().new_text_layout(self.input_manager.get_string())
                .font(FontFamily::SANS_SERIF, DEFAULT_COMPOSE_INDICATOR_FONT_SIZE)
                .text_color( self.config.get_color(VMColor::ComposeIndicatorTextColor).ok().expect("compose indicator text color not found in config"))
                .build().unwrap();
            ctx.paint_with_z_index(100, move |ctx| {
                ctx.draw_text(&layout, 
                    (Point::new(0., ctx_size.height-layout.size().height).to_vec2() + DEFAULT_COMPOSE_INDICATOR_INSET).to_point()
                    // (Point::new(0., 0.))
                );
            });
            if inner.widget().debug_data {
                let fps_layout = ctx.text().new_text_layout(
                    format!("FPS: {}", (1000./(now - self.last_frame_time) as f64).round())
                )
                .font(FontFamily::SANS_SERIF, 24.)
                .text_color(self.config.get_color(VMColor::ComposeIndicatorTextColor).unwrap())
                .build().unwrap();
                ctx.paint_with_z_index(100, move|ctx| {
                    ctx.draw_text(&fps_layout, 
                        Point::new(ctx_size.width-fps_layout.size().width, ctx_size.height-fps_layout.size().height)
                    );
                });
                self.last_frame_time = now;
            }
        }
        if self.dialog_visible {
            let rect = ctx.size().to_rect();
            if self.inner.is_none() {
                ctx.fill(rect,
                    &self.config.get_color(VMColor::SheetBackgroundColor).expect("Couldn't get sheet background color from config")
                    );
            }
            ctx.fill(rect,
                 &self.config.get_color(VMColor::DialogBackgroundColor).expect("Couldn't get sheet background color from config")
                );
            self.dialog.paint(ctx, &(), env);
        }
    }
}

#[derive(Data, Clone)]
struct AppState {
    menu_visible: bool,
    save_state: VMSaveState,
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn event(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _window_id: WindowId,
        event: Event,
        _data: &mut AppState,
        _env: &Env,
    ) -> Option<Event> {
        _ctx.submit_command(Command::new(
            EXECUTE_ACTION,
            ActionPayload {
                action: Action::PrintToLogInfo,
                string: Some(format!("{:?}", event)),
                ..Default::default()
            },
            Target::Global
        ));
        Some(event)
    }

    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> druid::Handled {
        if cmd.is(TOGGLE_MAIN_MENU) {
            data.menu_visible = !data.menu_visible;
            druid::Handled::Yes
        } else {
            druid::Handled::No
        }
    }

    fn window_added(
        &mut self,
        _id: WindowId,
        _handle: druid::WindowHandle,
        _data: &mut AppState,
        _env: &Env,
        _ctx: &mut druid::DelegateCtx,
    ) {
    }

    fn window_removed(&mut self, _id: WindowId, _data: &mut AppState, _env: &Env, _ctx: &mut druid::DelegateCtx) {}
}

#[allow(unused_must_use)]
pub fn main() {
    let mut canvas;
    match VMConfigSerde::load() {
        Ok(config) => {
            canvas = VMCanvas::new(config);
        }
        Err((err, config)) => {
            println!("{}", err);
            canvas = VMCanvas::new(config);
        }
    }

    let mut launch_with_file = false;
    let args: Vec<String> = std::env::args().collect();
    if let Some(str) = args.get(1) {
        let path = Path::new(str);
        if path.exists() {
            if let Some(ext) = path.extension() {
                if ext == "vmd" {
                    if let Ok((save, path)) = VMSaveSerde::load(path.display().to_string()) {
                        let vm = VMSaveSerde::from_save(save, canvas.config.clone());
                        canvas.path = Some(path.clone());
                        canvas.load_new_mapper(vm);
                        launch_with_file = true;
                        println!("Launching with open sheet: {}.", path.display());
                    }
                }
            }
        }
    }

    let window = WindowDesc::<AppState>::new(canvas)
    .title("Vim-Mapper")
    // .set_window_state(WindowState::Maximized)
    .set_window_state(WindowState::Restored)
    .menu(VMCanvas::make_menu);
    #[cfg(debug_assertions)]
    AppLauncher::with_window(window)
    .log_to_console()
    .launch(AppState {
        menu_visible: true,
        save_state: if launch_with_file {
            VMSaveState::Saved
        } else {
            VMSaveState::NoSheetOpened
        }
    })
    .expect("launch failed");
    #[cfg(not(debug_assertions))]
    {
        {
            use tracing_subscriber::prelude::*;
            let filter_layer = tracing_subscriber::filter::LevelFilter::ERROR;
            let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .init();
        }
    AppLauncher::with_window(window)
    .launch(AppState {
        menu_visible: true,
        save_state: if launch_with_file {
            VMSaveState::Saved
        } else {
            VMSaveState::NoSheetOpened
        }
    })
    .expect("launch failed");
    }
}