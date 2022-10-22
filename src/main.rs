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
use druid::widget::{prelude::*, Label, Flex, Button, MainAxisAlignment, SizedBox, ControllerHost};
use druid::{AppLauncher, WindowDesc, FileDialogOptions, Point, WindowState, Command, Target, WidgetPod, WidgetExt, MenuDesc, LocalizedString, MenuItem, FileSpec, FontFamily};
use druid::piet::{Text, TextLayout, TextLayoutBuilder};
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

struct VMCanvas {
    inner: Option<WidgetPod<(), VimMapper>>,
    dialog: WidgetPod<(), Flex<()>>,
    dialog_visible: bool,
    path: Option<PathBuf>,
    config: VMConfig,
    input_manager: VMInputManager,
    last_frame_time: u128,
}

impl VMCanvas {
    pub fn new(config: VMConfig) -> VMCanvas {
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

    pub fn open_file(&mut self, path: String) -> Result<(), String> {
        if let Ok(string) = fs::read_to_string(path.clone()) {
            if let Ok(save) = serde_json::from_str::<VMSave>(string.as_str()) {
                if let Ok(path) = Path::new(&path.clone()).canonicalize() {
                    self.path = Some(path);
                    self.load_new_mapper(VimMapper::from_save(save, self.config.clone()));
                    Ok(())
                } else {
                    Err("Not a valid path.".to_string())
                }
            } else {
                Err("Not a valid path.".to_string())
            }
        } else {
        Err("Couldn't load file.".to_string())
        }
    }

    pub fn save_file(&mut self) -> Result<String, String> {
        if let Some(mapper_pod) = &self.inner {
            match &self.path {
                Some(path) => {
                    if let Ok(string) = serde_json::to_string(&mapper_pod.widget().to_save()) {
                        if let Ok(_) = fs::write(path, string) {
                            Ok("File saved".to_string())
                        } else {
                            Err("Could not save to file.".to_string())
                        }
                    } else {
                        Err("Could not serialize map".to_string())
                    }
                }
                None => {
                    Err("No path set.".to_string())
                }
            }
        } else {
            Err("No sheet was openend.".to_string())
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

    pub fn make_dialog(config: &VMConfig) -> WidgetPod<(), Flex<()>> {
        let open_button = Button::new("Open...")
            .on_click(move |ctx, _, _| {
            ctx.submit_command(
                Command::new(
                    druid::commands::SHOW_OPEN_PANEL,
                    FileDialogOptions::new(),
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
}

#[allow(unused_must_use)]
impl Widget<()> for VMCanvas {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {
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
                if let Ok(_) = self.open_file(payload.path().to_str().unwrap().to_string()) {
                    ctx.children_changed();
                    ctx.request_layout();
                }
            }
            Event::Command(command) if command.is(druid::commands::SAVE_FILE) => {
                if let Some(_) = self.inner {
                    if let Some(_) = self.path {
                        self.save_file();
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
                    if let Ok(_path) = res {
                        self.save_file();
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
                    if let Some(payload) = payload {
                        if payload.action == Action::ToggleColorScheme {
                            println!("{:?}", payload.action);
                            self.config.toggle_color_scheme();
                            self.config.save();
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
                                },
                                Action::EditActiveNodeSelectAll => {
                                    if let Some(idx) = inner.widget().get_active_node_idx() {
                                        self.input_manager.set_keybind_mode(KeybindMode::EditBrowse);
                                        inner.widget_mut().open_editor(ctx, idx);
                                        ctx.submit_command(Command::new(REFRESH, (), Target::Auto));
                                        ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Auto));
                                    }
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
                                                println!("Execute doubleaction.");
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
                                }
                                _ => {
                                    if let Some(inner) = &self.inner {
                                        if !inner.widget().is_editor_open() {
                                            ctx.submit_command(EXECUTE_ACTION.with(payload.clone()));
                                        }
                                    }
                                }
                            } 
                        } else {
                            inner.event(ctx, event, data, env);
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
        let fps_layout = ctx.text().new_text_layout(
        format!("FPS: {}", (1000./(now - self.last_frame_time) as f64).round()))
        .font(FontFamily::SANS_SERIF, 24.)
        .text_color(self.config.get_color(VMColor::ComposeIndicatorTextColor).unwrap())
        .build().unwrap();
        ctx.paint_with_z_index(100, move|ctx| {
            ctx.draw_text(&fps_layout, 
                Point::new(ctx_size.width-fps_layout.size().width, ctx_size.height-fps_layout.size().height)
            );
        });
        self.last_frame_time = now;
        if let Some(path) = &self.path {
            ctx.window().set_title(format!("VimMapper - {}", path.display()).as_str());
        }
        if self.dialog_visible {
            let rect = ctx.size().to_rect();
            ctx.fill(rect,
                 &self.config.get_color(VMColor::SheetBackgroundColor).expect("Couldn't get sheet background color from config")
                );
            self.dialog.paint(ctx, data, env);
        } else if let Some(inner) = &mut self.inner {
            inner.paint(ctx, data, env);
        }
        //Paint VMInputManager indicator
        if let Some(_) = self.inner {
            let layout = ctx.text().new_text_layout(self.input_manager.get_string())
                .font(FontFamily::SANS_SERIF, DEFAULT_COMPOSE_INDICATOR_FONT_SIZE)
                .text_color( self.config.get_color(VMColor::ComposeIndicatorTextColor).ok().expect("compose indicator text color not found in config"))
                .build().unwrap();
            ctx.paint_with_z_index(100, move |ctx| {
                ctx.draw_text(&layout, 
                    (Point::new(0., ctx_size.height-layout.size().height).to_vec2() + DEFAULT_COMPOSE_INDICATOR_INSET).to_point()
                );
            });
        }
    }
}

pub fn main() {

    let open_dialog_options = FileDialogOptions::new()
    .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])]);
    let save_dialog_options = FileDialogOptions::new()
    .allowed_types(vec![FileSpec::new("VimMapper File", &["vmd"])])
    .default_type(FileSpec::new("VimMapper File", &["vmd"]))
    .default_name(DEFAULT_SAVE_NAME);

    let file_menu: MenuDesc<()> = MenuDesc::new(LocalizedString::new("file-menu").with_placeholder("File"))
    .append(druid::platform_menus::win::file::new())
    .append(
        MenuItem::new(
            LocalizedString::new("common-menu-file-open"),
            druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options),
        )
        .hotkey(druid::SysMods::Cmd, "o")
    )
    .append(druid::platform_menus::win::file::save())
    .append(
        MenuItem::new(
            LocalizedString::new("common-menu-file-save-as"),
            druid::commands::SHOW_SAVE_PANEL.with(save_dialog_options),
        )
        .hotkey(druid::SysMods::CmdShift, "s")
    )
    .append_separator()
    .append(druid::platform_menus::win::file::exit());

    let mut canvas: VMCanvas;
    match VMConfig::load() {
        Ok(config) => {
            canvas = VMCanvas::new(config);
        }
        Err(error) => {
            println!("Failed to load config with error: {}", error);
            canvas = VMCanvas::new(VMConfig::default());
        }
    }

    let args: Vec<String> = std::env::args().collect();
    if let Some(str) = args.get(1) {
        let path = Path::new(str);
        if path.exists() {
            if let Some(ext) = path.extension() {
                if ext == "vmd" {
                    if let Ok(_) = canvas.open_file(path.display().to_string()) {
                        println!("Launching with open sheet: {}.", path.display());
                    }
                }
            }
        }
    }

    let window = WindowDesc::new(|| canvas)
    .title("VimMapper")
    .set_window_state(WindowState::MAXIMIZED)
    .menu(MenuDesc::empty().append(file_menu));
    #[cfg(debug_assertions)]
    AppLauncher::with_window(window)
    .use_simple_logger()
    .launch(())
    .expect("launch failed");
    #[cfg(not(debug_assertions))]
    AppLauncher::with_window(window)
    .use_simple_logger()
    .launch(())
    .expect("launch failed");
}