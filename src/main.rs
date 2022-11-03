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
use druid::menu::MenuDesc;
use druid::widget::{prelude::*, Label, Flex, Button, MainAxisAlignment, SizedBox, ControllerHost};
use druid::{AppLauncher, WindowDesc, FileDialogOptions, Point, WindowState, Command, Target, WidgetPod, WidgetExt, LocalizedString, MenuItem, FileSpec, FontFamily, WindowId, Menu};
use druid::piet::{Text, TextLayout, TextLayoutBuilder};
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
            dialog: VMCanvas::make_dialog(&config),
            dialog_visible: true,
            path: None,
            config,
            input_manager: VMInputManager::new(),
            last_frame_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        }
    }

    pub fn set_path(&mut self, path: PathBuf) -> Result<PathBuf, String> {
        self.path = Some(path.clone());
        Ok(path.clone())
    }

    pub fn load_new_mapper(&mut self, mapper: VimMapper) {
        self.inner = Some(WidgetPod::new(mapper));
        self.dialog_visible = false;
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, payload: &Option<ActionPayload>) -> Result<(), ()> {
        if let Some(payload) = payload {
            if payload.action == Action::ToggleColorScheme {
                self.config.toggle_color_scheme();
                VMConfigSerde::save(&self.config);
                if let Some(vm) = &mut self.inner {
                    vm.widget_mut().set_config(self.config.clone());
                    ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                }
                self.dialog = VMCanvas::make_dialog(&self.config);
                ctx.children_changed();
                ctx.request_layout();
                ctx.request_paint();
                ctx.set_handled();
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
                            if let Some(_) = inner.widget_mut().add_node(idx, format!("New label"), None) {
                                ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                            }
                        }
                        return Ok(());
                    },
                    Action::CreateNewNodeAndEdit => {
                        if let Some(idx) = inner.widget().get_active_node_idx() {
                            if let Some(new_idx) = inner.widget_mut().add_node(idx, format!("New label"), None) {
                                self.input_manager.set_keybind_mode(KeybindMode::EditBrowse);
                                inner.widget_mut().open_editor(ctx, new_idx);
                                ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                                ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Auto));
                            }
                        }
                        return Ok(());
                    },
                    Action::EditActiveNodeSelectAll => {
                        if let Some(idx) = inner.widget().get_active_node_idx() {
                            self.input_manager.set_keybind_mode(KeybindMode::EditBrowse);
                            inner.widget_mut().open_editor(ctx, idx);
                            ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                            ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Auto));
                        }
                        return Ok(());
                    }
                    Action::ChangeModeWithTimeoutRevert => {
                        self.input_manager.set_timeout_revert_mode(Some(self.input_manager.get_keybind_mode()));
                        self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                        match payload.mode {
                            Some(KeybindMode::SearchBuild) | Some(KeybindMode::SearchedSheet) => {
                                if let Some(inner) = &mut self.inner {
                                    inner.widget_mut().set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                                }
                            }
                            _ => {
                                if let Some(inner) = &mut self.inner {
                                    inner.widget_mut().set_render_mode(NodeRenderMode::AllEnabled);
                                }
                            }
                        }
                        ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                        return Ok(());
                    },
                    Action::ChangeMode => {
                        match payload.mode {
                            Some(KeybindMode::Move) => {
                                if let Some(inner) = &self.inner {
                                    if let Some(active_idx) = inner.widget().get_active_node_idx() {
                                        if active_idx == 0 {
                                            ()
                                        } else {
                                            self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                        }
                                    }
                                }
                            }
                            Some(KeybindMode::SearchBuild) => {
                                self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                if let Some(inner) = &mut self.inner {
                                    inner.widget_mut().set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                                }
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
                                }
                            }
                            _ => {
                                self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());
                                if let Some(inner) = &mut self.inner {
                                    inner.widget_mut().set_render_mode(NodeRenderMode::AllEnabled);
                                }
                            }
                        }
                        ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                        return Ok(());
                    }
                    _ => {
                        if let Some(inner) = &self.inner {
                            if !inner.widget().is_editor_open() {
                                ctx.submit_command(EXECUTE_ACTION.with(payload.clone()));
                            }
                        }
                        return Ok(());
                    }
                } 
            } else {
                Err(())
            }
        } else {
            Ok(())
        }

    }

    pub fn make_dialog(config: &VMConfigVersion4) -> WidgetPod<(), Flex<()>> {
        let open_button = Button::new("Open...")
            .on_click(move |ctx, _, _| {
            ctx.submit_command(
                Command::new(
                    druid::commands::SHOW_OPEN_PANEL,
            FileDialogOptions::new()
                    .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])]),
                    Target::Auto
                )
            )
        });
        let new_button: ControllerHost<Button<()>, druid::widget::Click<_>> = Button::new("New")
            .on_click(move |ctx, _, _| {
            ctx.submit_command(
                Command::new(
                    druid::commands::NEW_FILE,
                    (),
                    Target::Auto
                )
            )
        });
        WidgetPod::new(
            Flex::column()
                .with_child(
                    SizedBox::new(
                        Flex::column()
                        .with_child(
                            Label::new(
                                "Do you want create a new sheet or load an existing one?"
                            )
                            .with_text_color(config.get_color(VMColor::LabelTextColor).expect("Couldn't get label text color from config"))
                            )
                        .with_child(SizedBox::empty().height(50.))
                        .with_child(
                            Flex::row().with_child(
                                new_button
                            ).with_default_spacer()
                            .with_child(
                                open_button
                            )   
                        ).main_axis_alignment(MainAxisAlignment::Center)
                    )
                    .padding(5.)
                    .border(config.get_color(VMColor::NodeBorderColor).expect("Couldn't get node border color from config")
                        , DEFAULT_BORDER_WIDTH)
                    .rounded(DEFAULT_BORDER_RADIUS)
                    .background(config.get_color(VMColor::NodeBackgroundColor).expect("Couldn't get node background color from config"))
                ).main_axis_alignment(MainAxisAlignment::Center)
        )
    }

    pub fn make_menu(_id: Option<WindowId>, _data: &(), _env: &Env) -> Menu<()> {
        let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])]);
        let save_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])])
        .default_type(FileSpec::new("VimMapper File", &["vmd"]))
        .default_name(DEFAULT_SAVE_NAME);

        let base = Menu::empty();
        let file_menu: Menu<()> = Menu::new(LocalizedString::new("file-menu").with_placeholder("File"))
        .entry(druid::platform_menus::win::file::new())
        .entry(
            MenuItem::new(
                LocalizedString::new("common-menu-file-open"),
                // druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options),
            )
            .command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options))
            .hotkey(druid::SysMods::Cmd, "o")
        )
        .entry(druid::platform_menus::win::file::save())
        .entry(
            MenuItem::new(
                LocalizedString::new("common-menu-file-save-as"),
                // druid::commands::SHOW_SAVE_PANEL.with(save_dialog_options),
            )
            .command(druid::commands::SHOW_SAVE_PANEL.with(save_dialog_options))
            .hotkey(druid::SysMods::CmdShift, "s")
        )
        .append_separator()
        .entry(druid::platform_menus::win::file::exit());

        base.entry(file_menu)
    }
}

#[allow(unused_must_use)]
impl Widget<()> for VMCanvas {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {
        ctx.request_layout();
        ctx.request_paint();
        if let Some(_) = &self.inner {
        } else {
            // if ctx.is_hot() {
                ctx.request_focus();
            // }
        }
        match event {
            Event::Command(command) if command.is(druid::commands::NEW_FILE) => {
                self.load_new_mapper(VimMapper::new(self.config.clone()));
                self.path = None;
                ctx.children_changed();
                ctx.request_layout();
            }
            Event::Command(command) if command.is(druid::commands::OPEN_FILE) => {
                let payload = command.get_unchecked(druid::commands::OPEN_FILE);
                if let Ok((save, path)) = VMSaveSerde::load(payload.path().to_str().unwrap().to_string()) {
                    let vm = VMSaveSerde::from_save(save, self.config.clone());
                    self.path = Some(path);
                    self.load_new_mapper(vm);
                    ctx.children_changed();
                    ctx.request_layout();
                }
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE) => {
                if let Some(inner) = &self.inner {
                    if let Some(path) = &self.path {
                        // self.save_file();
                        VMSaveSerde::save(&VMSaveSerde::to_save(&inner.widget()), (*path).clone());
                    } else {
                        ctx.submit_command(Command::new(
                            druid::commands::SHOW_SAVE_PANEL,
                            FileDialogOptions::new()
                                .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])])
                                .default_type(FileSpec::new("VimMapper File", &["vmd"]))
                                .default_name(DEFAULT_SAVE_NAME),
                            Target::Auto
                        ));
                    }
                }
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE_AS) => {
                if let Some(_) = self.inner {
                    let payload = command.get_unchecked(druid::commands::SAVE_FILE_AS);
                    let res = self.set_path(payload.path().to_path_buf());
                    if let Ok(path) = res {
                        // self.save_file();
                        VMSaveSerde::save(&VMSaveSerde::to_save(&self.inner.as_ref().unwrap().widget()), path);
                    } else if let Err(err) = res {
                        panic!("{}", err);
                    }
                }
            }
            Event::Command(command) if command.is(druid::commands::QUIT_APP) => {
                println!("QUIT_APP requested");
                ctx.set_handled();
            }
            Event::Command(command) if command.is(druid::commands::CLOSE_ALL_WINDOWS) => {
                println!("CLOSE_ALL_WINDOWS requested");
            }
            Event::Command(command) if command.is(druid::commands::CLOSE_WINDOW) => {
                let payload = command.get_unchecked(druid::commands::CLOSE_WINDOW);
                println!("CLOSE_WINDOW requested for {:?}", payload);
            }
            Event::Notification(note) if note.is(SUBMIT_CHANGES) => {
                if let Some(inner) = &mut self.inner {
                    inner.widget_mut().close_editor(ctx, true);
                    self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                    //Node has new label; invalidate layout
                    // self.nodes.get_mut(&self.get_active_node_idx().unwrap()).unwrap().container.layout = None;
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
                    if let Ok(_) = self.handle_action(ctx, payload) {

                    } else {
                        if let Some(inner) = &mut self.inner {
                            inner.event(ctx, event, data, env);
                        } else {
                            self.dialog.event(ctx, event, data, env);
                        }
                    }
                }
            }
            Event::Timer(token) => {
                if Some(*token) == self.input_manager.get_timout_token() {
                    self.input_manager.timeout();
                } 
            }
            _ => {
                if let Some(inner) = &mut self.inner {
                    inner.event(ctx, event, data, env);
                } else if self.dialog_visible {
                    self.dialog.event(ctx, event, data, env);
                }
            }
        }
        ctx.request_paint();
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(), env: &Env) {
        if let LifeCycle::WidgetAdded = event {
        }
        if self.dialog_visible {
            self.dialog.lifecycle(ctx, event, data, env);
        }
        if let Some(inner) = &mut self.inner {
            inner.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &(), data: &(), env: &Env) {
        if self.dialog_visible {
            self.dialog.update(ctx, data, env);
        } else if let Some(inner) = &mut self.inner {
            inner.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &(), env: &Env) -> Size {
        if self.dialog_visible {
            self.dialog.layout(ctx, bc, data, env);
            self.dialog.set_origin(ctx, data, env, Point::new(0., 0.));
        } 
        if let Some(inner) = &mut self.inner {
            inner.layout(ctx, bc, data, env);
            inner.set_origin(ctx, data, env, Point::new(0., 0.));
        }
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(), env: &Env) {
        let ctx_size = ctx.size();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if let Some(path) = &self.path {
            if path.display().to_string().starts_with(r"\\?") {
                ctx.window().set_title(format!("Vim-Mapper - {}", &(path.display().to_string())[4..]).as_str());
            } else {
                ctx.window().set_title(format!("Vim-Mapper - {}", path.display()).as_str());
            }
        }
        if self.dialog_visible {
            let rect = ctx.size().to_rect();
            ctx.fill(rect,
                 &self.config.get_color(VMColor::SheetBackgroundColor).expect("Couldn't get sheet background color from config")
                );
            self.dialog.paint(ctx, data, env);
        } else if let Some(inner) = &mut self.inner {
            inner.paint(ctx, data, env);
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
    }
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
                        println!("Launching with open sheet: {}.", path.display());
                    }
                }
            }
        }
    }

    let window = WindowDesc::new(canvas)
    .title("Vim-Mapper")
    .set_window_state(WindowState::Maximized);
    // .menu(VMCanvas::make_menu);
    #[cfg(debug_assertions)]
    AppLauncher::with_window(window)
    .log_to_console()
    .launch(())
    .expect("launch failed");
    #[cfg(not(debug_assertions))]
    AppLauncher::with_window(window)
    .use_simple_logger()
    .launch(())
    .expect("launch failed");
}