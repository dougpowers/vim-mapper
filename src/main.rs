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
use vmdialog::{VMDialogParams, VMDialog, VMInputParams};
use vmtabbar::VMTabBar;
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf, Path};
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_family = "windows")]
use windows::{
    core::{PCWSTR},
    Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE},
    },
};

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

mod vmgraphclip;
use vmgraphclip::VMGraphClip;

mod vmdialog;

mod vmbutton;

mod vmtabbar;

mod vmtextinput;

struct VMCanvas {
    tabs: Vec<VMTab>,
    active_tab: usize,
    tab_bar: WidgetPod<(), VMTabBar>,
    dialog: WidgetPod<String, Flex<String>>,
    dialog_visible: bool,
    path: Option<PathBuf>,
    config: VMConfigVersion4,
    last_frame_time: u128,
    take_focus: bool,
    start_input_manager: VMInputManager,
    graph_clip_registers: HashMap<String, VMGraphClip>,
    debug_data: bool,
}

pub struct VMTab {
    vm: WidgetPod<(), VimMapper>,
    tab_name: String,
}

impl VMCanvas {
    pub fn new(config: VMConfigVersion4) -> VMCanvas {
        VMCanvas {
            tabs: vec![],
            active_tab: 0,
            tab_bar: WidgetPod::new(VMTabBar::new(&config, &vec![], 0)),
            dialog: VMCanvas::new_dialog(&config, VMDialog::make_start_dialog_params()),
            dialog_visible: true,
            path: None,
            config,
            last_frame_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            take_focus: true,
            start_input_manager: VMInputManager::new(),
            graph_clip_registers: HashMap::new(),
            debug_data: false,
        }
    }

    #[allow(dead_code)]
    pub fn get_register(&mut self, register: String) -> Option<&VMGraphClip> {
        if let Some(clip) = self.graph_clip_registers.get_mut(&register) {
            return Some(clip);
        } else {
            return None;
        }
    }

    pub fn set_path(&mut self, path: PathBuf) -> Result<PathBuf, String> {
        if path.is_file() {
            self.path = Some(path.clone());
            return Ok(path.clone());
        } else {
            self.path = Some(path.clone());
            if let Ok(_) = fs::write(&path, "test") {
                #[allow(unused_must_use)]
                {
                    fs::remove_file(path.clone());
                }
                self.path = Some(path.clone());
                return Ok(path.clone());
            } else {
                return Err(String::from("Path is not accessible!"));
            }
        }
    }

    pub fn load_new_tabs(&mut self, tabs: Vec<VMTab>, active_tab: Option<usize>) {
        let tab_names = &tabs.iter().map(|v| {return v.tab_name.clone();}).collect();
        self.tabs = vec![];
        for tab in tabs {
            self.tabs.push(tab);
        }
        if let Some(active_tab) = active_tab {
            self.active_tab = active_tab; 
        }
        self.tab_bar.widget_mut().update_tabs(
            tab_names,
            self.active_tab
        );
        self.hide_dialog();
    }
    
    fn load_tab(&mut self, ctx: &mut EventCtx, tab: VMTab) {
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        ctx.children_changed();
        ctx.request_layout();
        ctx.set_handled();
        let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
        self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
    }

    fn new_tab(&mut self, ctx: &mut EventCtx, tab_name: String) {
        self.tabs.push(VMTab {
            vm: WidgetPod::new(VimMapper::new(self.config.clone())),
            tab_name,
        });
        self.active_tab = self.tabs.len() - 1;
        self.tabs[self.active_tab].vm.widget_mut().set_node_as_active(0);
        self.tabs[self.active_tab].vm.widget_mut().debug_data = self.debug_data;
        ctx.children_changed();
        ctx.request_layout();
        ctx.set_handled();
        let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
        self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
    }

    fn delete_tab(&mut self, ctx: &mut EventCtx, tab_id: usize) {
        if let Some(_) = self.tabs.get(tab_id) {
            self.tabs.remove(tab_id);
            if tab_id == self.active_tab {
                if self.active_tab > self.tabs.len() - 1 {
                    self.active_tab = self.active_tab - 1;
                }
            } else {
                if self.active_tab > tab_id {
                    self.active_tab -= 1;
                }
            }
            ctx.children_changed();
            ctx.request_layout();
            ctx.set_handled();
            let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
            self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
        }
    }

    fn new_dialog(config: &VMConfigVersion4, params: VMDialogParams) -> WidgetPod<String, Flex<String>> {
        let dialog = VMDialog::as_dialog(config, params);
        dialog.inner
    }

    fn new_input(config: &VMConfigVersion4, params: VMInputParams) -> WidgetPod<String, Flex<String>> {
        let dialog = VMDialog::as_input(config, params);
        dialog.inner
    }

    fn hide_dialog(&mut self) {
        self.tabs[self.active_tab].vm.widget_mut().input_manager.set_keybind_mode(KeybindMode::Sheet);
        self.dialog_visible = false;
        self.take_focus = true;
    }

    fn set_input_dialog(&mut self, ctx: &mut EventCtx, data: &mut AppState, params: VMInputParams, default_value: String, show: bool) {
        data.dialog_input_text = default_value;
        self.dialog = VMCanvas::new_input(&self.config, params);
        ctx.children_changed();
        ctx.request_layout();
        ctx.request_paint();
        ctx.set_handled();
        self.dialog_visible = show;
        if show {
            if data.save_state != VMSaveState::NoSheetOpened {
                self.tabs[self.active_tab].vm.widget_mut().input_manager.set_keybind_mode(KeybindMode::Dialog);
            } else {
                self.tabs[self.active_tab].vm.widget_mut().input_manager.set_keybind_mode(KeybindMode::Sheet);
            }
            ctx.focus_next();
        }
    }

    fn set_dialog(&mut self, ctx: &mut EventCtx, data: &mut AppState, params: VMDialogParams, show: bool) {
        self.dialog = VMCanvas::new_dialog(&self.config, params);
        ctx.children_changed();
        ctx.request_layout();
        ctx.request_paint();
        ctx.set_handled();
        self.dialog_visible = show;
        if show {
            if let Some(tab)  = self.tabs.get_mut(self.active_tab) {
                if data.save_state != VMSaveState::NoSheetOpened {
                    tab.vm.widget_mut().input_manager.set_keybind_mode(KeybindMode::Dialog);
                } else {
                    tab.vm.widget_mut().input_manager.set_keybind_mode(KeybindMode::Sheet);
                }
            } else {
                if data.save_state != VMSaveState::NoSheetOpened {
                    self.start_input_manager.set_keybind_mode(KeybindMode::Dialog);
                } else {
                    self.start_input_manager.set_keybind_mode(KeybindMode::Sheet);
                }
            }
            ctx.focus_next();
        }
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, data: &mut AppState, payload: &Option<ActionPayload>) -> Result<(), ()> {
        if let Some(payload) = payload {
            if let Some(_) = self.path {
                match payload.action {
                    Action::CreateNewNode |
                    Action::CreateNewNodeAndEdit |
                    Action::CreateNewTab |
                    Action::DeleteActiveTab |
                    Action::RenameTab |
                    Action::GoToNextTab |
                    Action::GoToPreviousTab |
                    Action::ActivateTargetedNode |
                    Action::IncreaseNodeMass |
                    Action::DecreaseNodeMass |
                    Action::ResetNodeMass |
                    Action::ToggleNodeAnchor |
                    Action::EditActiveNodeSelectAll |
                    Action::EditActiveNodeAppend |
                    Action::EditActiveNodeInsert |
                    Action::AttemptNodeDeletion |
                    Action::CutNode |
                    Action::CutTargetNode |
                    Action::CutNodeTree |
                    Action::PasteNodeTree |
                    Action::PasteNodeTreeAsTab |
                    Action::PasteNodeTreeExternal |
                    Action::MarkActiveNode |
                    Action::MoveActiveNodeDown |
                    Action::MoveActiveNodeUp |
                    Action::MoveActiveNodeLeft |
                    Action::MoveActiveNodeRight |
                    Action::AcceptNodeText |
                    Action::PanUp |
                    Action::PanDown |
                    Action::PanLeft |
                    Action::PanRight |
                    Action::ZoomOut |
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
                    if self.tabs.get(self.active_tab).is_some() {
                        for tab in self.tabs.iter_mut() {
                            tab.vm.widget_mut().set_config(self.config.clone());
                        }

                        self.tab_bar = WidgetPod::new(VMTabBar::new(&self.config, &self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect(), self.active_tab));
                        ctx.children_changed();

                        ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                        if self.dialog_visible {
                            ctx.submit_command(Command::new(
                                DIALOG_EXECUTE_ACTIONS,
                                vec![
                                    ActionPayload {
                                        action: Action::NullAction,
                                        ..Default::default()
                                    }
                                ],
                                Target::Global
                            ));
                        }
                    } else {
                        self.set_dialog(ctx, data, VMDialog::make_start_dialog_params(), self.dialog_visible);
                    }
                    ctx.set_focus(ctx.widget_id());
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
                        self.set_dialog(ctx, data, VMDialog::make_save_as_and_open_dialog_params(), true);
                        return Ok(());
                    } else {
                        self.set_dialog(ctx, data, VMDialog::make_save_and_open_dialog_params(), true);
                        return Ok(());
                    }
                }
                Action::CreateNewSheet => {
                    if data.save_state == VMSaveState::Saved || data.save_state == VMSaveState::NoSheetOpened || data.save_state == VMSaveState::DiscardChanges {
                        // self.load_new_mapper(VimMapper::new(self.config.clone()));
                        self.load_new_tabs(vec![VMTab { tab_name: String::from("Tab 1"), vm: WidgetPod::new(VimMapper::new(self.config.clone()))}], Some(0));
                        self.path = None;
                        data.save_state = VMSaveState::NoSave;
                        ctx.children_changed();
                        ctx.request_layout();
                        ctx.set_handled();
                        return Ok(());
                    } else if data.save_state == VMSaveState::NoSave {
                        self.set_dialog(ctx, data, VMDialog::make_save_as_and_new_dialog_params(), true);
                        return Ok(());
                    } else {
                        self.set_dialog(ctx, data, VMDialog::make_save_and_new_dialog_params(), true);
                        return Ok(());
                    }
                }
                Action::ToggleMenuVisible => {
                    data.menu_visible = !data.menu_visible;
                    self.config.menu_shown = Some(data.menu_visible);
                    #[allow(unused_must_use)]
                    {
                        VMConfigSerde::save(&self.config);
                    }
                    ctx.set_handled();
                    return Ok(());
                }
                Action::CreateDialog => {
                    self.set_dialog(ctx, data, payload.dialog_params.clone().unwrap(), true);
                    return Ok(());
                }
                Action::SaveSheet => { 
                    let tab = &mut self.tabs.get_mut(self.active_tab);
                    if let Some(_) = tab {
                        if let Some(path) = self.path.clone() {
                            if let Ok(_) = VMSaveSerde::save(
                                &VMSaveSerde::to_save(&self.tabs, self.active_tab),
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
                Action::QuitWithoutSaveGuard => {
                    ctx.submit_command(druid::commands::QUIT_APP);
                }
                Action::SetSaveState => {
                    data.save_state = payload.save_state.clone().unwrap();
                }
                _ => ()
            }
        }
        let tab = &mut self.tabs.get_mut(self.active_tab);
        if let Some(tab) = tab {
            if let Some(payload) = payload {
                if payload.action != Action::ChangeModeWithTimeoutRevert {
                    // inner.widget_mut().input_manager.clear_timeout();
                }
                match payload.action {
                    Action::ToggleDebug => {
                        #[cfg(debug_assertions)]
                        {
                            self.debug_data = !self.debug_data;
                            for tab in &mut self.tabs {
                                tab.vm.widget_mut().debug_data = self.debug_data;
                            }
                            return Ok(());
                        }
                        #[cfg(not(debug_assertions))]
                        {
                            return Ok(());
                        }
                    },
                    Action::GoToNextTab => {
                        if self.tabs.len() > 1 {
                            if let Some(_) = self.tabs.get_mut(self.active_tab + 1) {
                                self.active_tab = self.active_tab + 1;
                                ctx.children_changed();
                                ctx.request_layout();
                                ctx.set_handled();
                                let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
                                self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
                                return Ok(());
                            } else {
                                self.active_tab = 0;
                                ctx.children_changed();
                                ctx.request_layout();
                                ctx.set_handled();
                                let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
                                self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
                                return Ok(());
                            }
                        } else {
                            return Ok(());
                        }
                    },
                    Action::GoToPreviousTab => {
                        let tabs_len = self.tabs.len();
                        if tabs_len > 1 {
                            if self.active_tab == 0 {
                                self.active_tab = tabs_len-1;
                                ctx.children_changed();
                                ctx.request_layout();
                                ctx.set_handled();
                                let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
                                self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
                                return Ok(());
                            } else {
                                self.active_tab = self.active_tab-1;
                                ctx.children_changed();
                                ctx.request_layout();
                                ctx.set_handled();
                                let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
                                self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
                                return Ok(());
                            }
                        } else {
                            return Ok(());
                        }
                    },
                    Action::GoToTab => {
                        let tab_index = payload.tab_index.unwrap();
                        if let Some(_) = self.tabs.get_mut(tab_index) {
                            self.active_tab = tab_index;
                            ctx.children_changed();
                            ctx.request_layout();
                            ctx.set_handled();
                            let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
                            self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
                            return Ok(());
                        } else {
                            return Ok(());
                        }
                    },
                    Action::OpenNewTabInput => {
                        self.set_input_dialog(ctx, data, VMDialog::make_new_tab_prompt_input_params(),
                            String::from(format!("Tab {}", self.tabs.len() + 1)), 
                            true
                        );
                        return Ok(());
                    },
                    Action::OpenRenameTabInput => {
                        self.set_input_dialog(ctx, data, VMDialog::make_rename_tab_prompt_input_params(),
                            String::from(format!("Tab {}", self.active_tab + 1)), 
                            true
                        );
                        return Ok(());
                    },
                    Action::RenameTab => {
                        tab.tab_name = payload.string.clone().unwrap();
                        let tab_names: Vec<String> = self.tabs.iter().map(|v| -> String {return v.tab_name.clone()}).collect();
                        self.tab_bar.widget_mut().update_tabs(&tab_names, self.active_tab);
                        return Ok(());
                    },
                    Action::CreateNewTab => {
                        let name = payload.string.clone();
                        if name.is_none() {
                            self.new_tab(ctx, format!("Tab {}", self.tabs.len() + 1))
                        } else {
                            self.new_tab(ctx, payload.string.clone().unwrap());
                        }
                        return Ok(());
                    },
                    Action::OpenDeleteTabPrompt => {
                        if self.tabs.len() > 1 {
                            if let Some(tab_index) = payload.tab_index {
                                self.set_dialog(ctx, data, VMDialog::make_delete_tab_prompt_dialog_params(tab_index), true);
                            } else {
                                self.set_dialog(ctx, data, VMDialog::make_delete_tab_prompt_dialog_params(self.active_tab), true);
                            }
                        }
                        return Ok(());
                    },
                    Action::DeleteActiveTab => {
                        self.delete_tab(ctx, self.active_tab);
                        return Ok(());
                    },
                    Action::DeleteTab => {
                        if let Some(tab_index) = payload.tab_index {
                            self.delete_tab(ctx, tab_index);
                        }
                        return Ok(());
                    },
                    Action::PasteNodeTreeAsTab => {
                        if let Some(graph_clip) = self.graph_clip_registers.get("0") {
                            self.load_tab(ctx, graph_clip.init_tab_with_clip(self.config.clone()));
                            ctx.children_changed();
                            ctx.request_layout();
                            ctx.set_handled();
                        }
                        return Ok(());
                    },
                    _ => {
                        let tab = &mut self.tabs.get_mut(self.active_tab);
                        if let Some(tab) = tab {
                            let inner = &mut tab.vm;
                            ctx.submit_command(Command::new(EXECUTE_ACTION, payload.clone(), Target::Widget(inner.id())));
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
        if self.take_focus {
            ctx.request_focus();
            self.take_focus = false;
        }
        match event {
            Event::Command(command) if command.is(druid::commands::NEW_FILE) => {
                tracing::error!("druid NEW_FILE called. this is an invalid command.");
                panic!();
            }
            Event::Command(command) if command.is(druid::commands::OPEN_FILE) => {
                let payload = command.get_unchecked(druid::commands::OPEN_FILE);
                if let Ok((save, path)) = VMSaveSerde::load(payload.path().to_str().unwrap().to_string()) {
                    let (tabs, active_tab) = VMSaveSerde::from_save(save, self.config.clone());
                    self.path = Some(path);
                    self.load_new_tabs(tabs, Some(active_tab));
                    data.save_state = VMSaveState::Saved;
                    ctx.children_changed();
                }
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE) => {
                tracing::error!("druid::commands::SAVE_FILE command sent! This should not happen!");
                panic!();
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE_AS) => {
                if self.tabs.get(self.active_tab).is_some() {
                    let payload = command.get_unchecked(druid::commands::SAVE_FILE_AS);
                    let res = self.set_path(payload.path().to_path_buf());
                    // let inner = &self.tabs.get(self.active_tab).unwrap().vm;
                    if let Ok(path) = res {
                        match data.save_state {
                            VMSaveState::UnsavedChanges => {
                                data.save_state = VMSaveState::Saved;
                                // VMSaveSerde::save(&VMSaveSerde::to_save(inner.widget()), path);
                                VMSaveSerde::save(
                                    &VMSaveSerde::to_save(&self.tabs, self.active_tab),
                                    path
                                );
                            },
                            VMSaveState::SaveAsInProgress => {
                                data.save_state = VMSaveState::Saved;
                                // VMSaveSerde::save(&VMSaveSerde::to_save(inner.widget()), path);
                                VMSaveSerde::save(
                                    &VMSaveSerde::to_save(&self.tabs, self.active_tab),
                                    path
                                );
                            },
                            VMSaveState::SaveAsInProgressThenQuit => {
                                data.save_state = VMSaveState::Saved;
                                // VMSaveSerde::save(&VMSaveSerde::to_save(inner.widget()), path);
                                VMSaveSerde::save(
                                    &VMSaveSerde::to_save(&self.tabs, self.active_tab),
                                    path
                                );
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
                                // VMSaveSerde::save(&VMSaveSerde::to_save(inner.widget()), path);
                                VMSaveSerde::save(
                                    &VMSaveSerde::to_save(&self.tabs, self.active_tab),
                                    path
                                );
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
                                // VMSaveSerde::save(&VMSaveSerde::to_save(inner.widget()), path);
                                VMSaveSerde::save(
                                    &VMSaveSerde::to_save(&self.tabs, self.active_tab),
                                    path
                                );
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
                                // VMSaveSerde::save(&VMSaveSerde::to_save(inner.widget()), path);
                                VMSaveSerde::save(
                                    &VMSaveSerde::to_save(&self.tabs, self.active_tab),
                                    path
                                );
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
                        self.take_focus = true;
                        self.set_dialog(ctx, data, VMDialog::make_start_dialog_params(), true);
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
                    VMSaveState::DiscardChanges => {
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
                if self.dialog_visible && data.save_state != VMSaveState::NoSheetOpened {
                } else {
                    match data.save_state {
                        VMSaveState::NoSheetOpened => ctx.submit_command(druid::commands::QUIT_APP),
                        VMSaveState::Saved => ctx.submit_command(druid::commands::QUIT_APP),
                        VMSaveState::DiscardChanges => ctx.submit_command(druid::commands::QUIT_APP),
                        VMSaveState::NoSave => {
                            self.set_dialog(ctx, data, VMDialog::make_save_as_and_quit_dialog_params(), true);
                        },
                        VMSaveState::UnsavedChanges => {
                            self.set_dialog(ctx, data, VMDialog::make_save_and_quit_dialog_params(), true);
                        }
                        _ => ()
                    }
                }
            }
            Event::Command(command) if command.is(EXECUTE_ACTION) && !ctx.is_handled() => {
                let payload = command.get_unchecked(EXECUTE_ACTION);
                if let Ok(_) = self.handle_action(ctx, data, &Some(payload.to_owned())) {
                    ctx.set_handled();
                } else {
                    let tab = &mut self.tabs.get_mut(self.active_tab);
                    if let Some(tab) = tab {
                        let inner = &mut tab.vm;
                        inner.event(ctx, event, &mut(), env);
                    } else {
                        self.dialog.event(ctx, event, &mut data.dialog_input_text, env);
                    }
                }
            }
            Event::Command(command) if command.is(DIALOG_EXECUTE_ACTIONS) && !ctx.is_handled() => {
                let payloads = command.get_unchecked(DIALOG_EXECUTE_ACTIONS);
                if data.save_state != VMSaveState::NoSheetOpened {
                    self.hide_dialog();
                }
                for payload in payloads {
                    self.handle_action(ctx, data, &Some((*payload).clone()));
                }
            }
            Event::Command(command) if command.is(SET_REGISTER) => {
                let (register_name, graph_clip) = command.get_unchecked(SET_REGISTER);
                self.graph_clip_registers.insert(register_name.clone(), graph_clip.clone());
            }
            Event::Command(command) if command.is(GET_REGISTER) => {
                let (register_name, paste_external, external_point) = command.get_unchecked(GET_REGISTER);
                if let Some(graph_clip) = self.graph_clip_registers.get(register_name) {
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        ctx.submit_command(Command::new(OFFER_REGISTER,
                            (register_name.clone(), graph_clip.clone(), *paste_external, *external_point),
                            Target::Widget(tab.vm.id())
                        ));
                    }
                }
            }
            Event::Notification(note) if note.is(SUBMIT_CHANGES) => {
                let tab = &mut self.tabs.get_mut(self.active_tab);
                if let Some(tab) = tab {
                    let inner = &mut tab.vm;
                    // self.input_managers[self.active_tab].set_keybind_mode(KeybindMode::Sheet);
                    inner.widget_mut().input_manager.set_keybind_mode(KeybindMode::Sheet);
                    inner.widget_mut().invalidate_node_layouts();
                    ctx.set_handled();
                    inner.widget_mut().restart_simulation();
                    ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                    ctx.request_focus();
                }
            }
            Event::Notification(note) if note.is(CANCEL_CHANGES) => {
                let tab = &mut self.tabs.get_mut(self.active_tab);
                if let Some(tab) = tab {
                    tab.vm.widget_mut().input_manager.set_keybind_mode(KeybindMode::Sheet);
                    ctx.set_handled();
                    ctx.request_anim_frame();
                    ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                    ctx.request_focus();
                }
            }
            Event::KeyDown(key_event) => {
                let mut im = &mut self.start_input_manager;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    im = &mut tab.vm.widget_mut().input_manager;
                }
                let payloads = im.accept_key(key_event.clone(), ctx);
                for payload in &payloads {
                    if self.dialog_visible 
                        && ((key_event.key == druid::keyboard_types::Key::Tab ||
                        key_event.key == druid::keyboard_types::Key::Enter ||
                        key_event.key == druid::keyboard_types::Key::Character(String::from(" "))) ||
                        key_event.key == druid::keyboard_types::Key::Escape)
                        {
                        if key_event.key == druid::keyboard_types::Key::Tab {
                            ctx.focus_next();
                            ctx.set_handled();
                        }
                        self.dialog.event(ctx, event, &mut data.dialog_input_text, env);
                    } else {
                        if let Ok(_) = self.handle_action(ctx, data, payload) {

                        } else {
                            let tab = &mut self.tabs.get_mut(self.active_tab);
                            if let Some(tab) = tab {
                                let inner = &mut tab.vm;
                                inner.event(ctx, event, &mut (), env);
                            }
                        }
                    } 
                }
            }
            Event::Timer(token) => {
                let mut im = &mut self.start_input_manager;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    im = &mut tab.vm.widget_mut().input_manager;
                }
                if Some(*token) == im.get_timeout_revert_token() {
                    im.revert_timeout(ctx);
                } else if Some(*token) == im.get_timeout_build_token() {
                    im.build_timeout(ctx);
                }
            }
            Event::WindowConnected => {
                ctx.request_focus();
                if let Some(menu_visible) = self.config.menu_shown {
                    data.menu_visible = menu_visible;
                }
            }
            Event::MouseDown(mouse_event) => {
                if self.dialog_visible {
                    self.dialog.event(ctx, event, &mut data.dialog_input_text, env);
                } else if self.tab_bar.layout_rect().contains(mouse_event.pos) {
                    self.tab_bar.event(ctx, event, &mut (), env);
                } else if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let inner = &mut tab.vm;
                    if let Some(register) = self.graph_clip_registers.get("0") {
                        inner.widget_mut().default_paste_register_count = register.get_graph().node_count();
                    }
                    inner.event(ctx, event, &mut (), env);
                }
            }
            _ => {
                if self.dialog_visible {
                    self.dialog.event(ctx, event, &mut data.dialog_input_text, env);
                } else if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let inner = &mut tab.vm;
                    inner.event(ctx, event, &mut (), env);
                }
            }
        }
        ctx.request_paint();
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &AppState, env: &Env) {
        if let LifeCycle::WidgetAdded = event{
            // VMInputManager::validate_keybinds();
        } else if let LifeCycle::FocusChanged(focused) = event {
            if *focused {
                // tracing::debug!("Main window gained focus");
            } else {
                // tracing::debug!("Main window lost focus");
            }
        } else if let LifeCycle::BuildFocusChain = event {
            ctx.register_for_focus();
        }
        if self.dialog_visible {
            self.dialog.lifecycle(ctx, event, &_data.dialog_input_text, env);
        }
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let inner = &mut tab.vm;
            inner.lifecycle(ctx, event, &(), env);
            self.tab_bar.lifecycle(ctx, event, &(), env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, env: &Env) {
        if self.dialog_visible {
            self.dialog.update(ctx, &_data.dialog_input_text, env);
        } else if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let inner = &mut tab.vm;
            inner.update(ctx, &(), env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, env: &Env) -> Size {
        if self.dialog_visible {
            self.dialog.layout(ctx, bc, &_data.dialog_input_text, env);
            self.dialog.set_origin(ctx, Point::new(0., 0.));
        } 
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let inner = &mut tab.vm;
            inner.layout(ctx, bc, &(), env);
            inner.set_origin(ctx, Point::new(0., 0.));
            self.tab_bar.layout(ctx, bc, &(), env);
            self.tab_bar.set_origin(ctx, Point::new(0., 0.));
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
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let inner = &mut tab.vm;
            inner.paint(ctx, &(), env);
            if !inner.widget().input_manager.get_keybind_mode().contains(KeybindMode::SearchEntry) {
                let input_layout = ctx.text()
                    .new_text_layout(
                        // self.input_managers[self.active_tab].get_string()
                        format!("{}{}", inner.widget().input_manager.get_mode_prompt(), inner.widget().input_manager.get_string())
                    )
                    .font(FontFamily::SANS_SERIF, DEFAULT_COMPOSE_INDICATOR_FONT_SIZE)
                    .text_color( self.config.get_color(VMColor::ComposeIndicatorTextColor).ok().expect("compose indicator text color not found in config"))
                    .build().unwrap();
                ctx.paint_with_z_index(100, move |ctx| {
                    ctx.draw_text(&input_layout, 
                        (Point::new(0., ctx_size.height-input_layout.size().height).to_vec2() + DEFAULT_COMPOSE_INDICATOR_INSET).to_point()
                    );
                });
            }
            let mode_label = ctx.text()
                .new_text_layout(
                    // self.input_managers[self.active_tab].get_string()
                    // format!("{}{}", inner.widget().input_manager.get_mode_prompt(), inner.widget().input_manager.get_string())
                    inner.widget().input_manager.get_mode_label().to_string()
                )
                .font(FontFamily::SANS_SERIF, DEFAULT_COMPOSE_INDICATOR_FONT_SIZE)
                .text_color( self.config.get_color(VMColor::ComposeIndicatorTextColor).ok().expect("compose indicator text color not found in config"))
                .build().unwrap();
            ctx.paint_with_z_index(100, move |ctx| {
                ctx.draw_text(&mode_label, 
                    (Point::new(ctx_size.width-mode_label.size().width, ctx_size.height-mode_label.size().height).to_vec2()).to_point()
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
            self.tab_bar.paint_always(ctx, &(), env);
        }
        if self.dialog_visible {
            let rect = ctx.size().to_rect();
            let tab = self.tabs.get_mut(self.active_tab);
            if tab.is_none() {
                ctx.fill(rect,
                    &self.config.get_color(VMColor::SheetBackgroundColor).expect("Couldn't get sheet background color from config")
                    );
            }
            ctx.fill(rect,
                 &self.config.get_color(VMColor::DialogBackgroundColor).expect("Couldn't get sheet background color from config")
                );
            self.dialog.paint(ctx, &data.dialog_input_text, env);
        }
    }
}

#[derive(Data, Clone)]
pub struct AppState {
    menu_visible: bool,
    save_state: VMSaveState,
    dialog_input_text: String,
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
    #[cfg(target_family = "windows")]
    let _icon = unsafe {
        LoadImageW(
            GetModuleHandleW(None).unwrap(),
            PCWSTR(1 as _),
            IMAGE_ICON,
            0,
            0,  
            LR_DEFAULTSIZE,
        )
    };

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
    let mut launch_with_unsaved_path = false;
    let args: Vec<String> = std::env::args().collect();
    if let Some(str) = args.get(1) {
        let path = Path::new(str);
        if path.exists() {
            if let Some(ext) = path.extension() {
                if ext == "vmd" {
                    if let Ok((save, path)) = VMSaveSerde::load(path.display().to_string()) {
                        let (tabs, active_tab) = VMSaveSerde::from_save(save, canvas.config.clone());
                        canvas.path = Some(path.clone());
                        // canvas.load_new_mapper(vm);
                        canvas.load_new_tabs(tabs, Some(active_tab));
                        launch_with_file = true;
                        println!("Launching with open sheet: {}...", path.display());
                    }
                }
            }
        } else if let Some(ext) = path.extension() {
            if ext == "vmd" {
                if let Ok(path) = canvas.set_path(path.to_path_buf()) {
                    let (tabs, active_tab) = (vec![VMTab {tab_name: String::from("Tab 1"), vm: WidgetPod::new(VimMapper::new(canvas.config.clone()))}], 0);
                    launch_with_unsaved_path = true;
                    // canvas.load_new_mapper(vm);
                    canvas.load_new_tabs(tabs, Some(active_tab));
                    println!("Launching new file with path: {}...", path.display());
                }
            } else {
                println!("Cannot create a new file without a .vmd extension!");
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
        } else if launch_with_unsaved_path {
            VMSaveState::UnsavedChanges
        } else {
            VMSaveState::NoSheetOpened
        },
        dialog_input_text: String::from("")
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
        } else if launch_with_unsaved_path {
            VMSaveState::UnsavedChanges
        } else {
            VMSaveState::NoSheetOpened
        },
        dialog_input_text: String::from("")
    })
    .expect("launch failed");
    }
}