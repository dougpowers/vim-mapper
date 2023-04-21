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

use druid::{Size, Widget, piet::{PietTextLayout, Text, TextLayoutBuilder, TextLayout},Color, RenderContext, Rect, Point, FontFamily, Event, Command, Target, Menu, MenuItem};

use crate::{vmconfig::{VMConfigVersion4, VMColor}, vminput::{ActionPayload, Action}, AppState};
use crate::constants::*;

pub struct VMTabBar {
    tabs: Vec<(String, Option<PietTextLayout>, Option<Size>)>,
    active_tab: usize,
    tab_text_color: Color,
    tab_background_color_active: Color,
    tab_background_color_inactive: Color,
    tab_active_indicator_color: Color,
}

impl VMTabBar {
    pub fn new(config: &VMConfigVersion4, tab_names: &Vec<String>, active_tab: usize) -> VMTabBar {
        let mut tabs: Vec<(String, Option<PietTextLayout>, Option<Size>)> = vec![];
        for name in tab_names {
            tabs.push((
                name.clone(),
                None,
                None,
            ));
        }
        VMTabBar {
            tabs,
            active_tab,
            tab_text_color: config.get_color(VMColor::TabText).unwrap(),
            tab_background_color_active: config.get_color(VMColor::TabActive).unwrap(),
            tab_background_color_inactive: config.get_color(VMColor::TabInactive).unwrap(),
            tab_active_indicator_color: config.get_color(VMColor::TabIndicator).unwrap(),
        }
    }

    pub fn update_tabs(&mut self, tab_names: &Vec<String>, active_tab: usize) {
        self.tabs.clear();
        for (_index, name) in tab_names.iter().enumerate() {
            self.tabs.push((
                name.clone(),
                None,
                None,
            ));
        }
        self.active_tab = active_tab;
    }

    fn build_menu(&mut self, tab_index: usize) -> Menu<AppState> {
        let mut menu = Menu::<AppState>::empty();
        menu = menu.entry(
            MenuItem::new("New Tab").command(Command::new(EXECUTE_ACTION,
                ActionPayload {
                    action: Action::CreateNewTab,
                    tab_index: Some(tab_index),
                    ..Default::default()
                },
                Target::Global
            ))
        );
        menu = menu.entry(
            MenuItem::new("Rename Tab").command(Command::new(EXECUTE_ACTION,
                ActionPayload {
                    action: Action::OpenRenameTabInput,
                    tab_index: Some(tab_index),
                    ..Default::default()
                },
                Target::Global
            ))
        );
        if self.tabs.len() > 1 {
            menu = menu.separator();
            menu = menu.entry(
                MenuItem::new("Delete Tab").command(Command::new(EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::OpenDeleteTabPrompt,
                        tab_index: Some(tab_index),
                        ..Default::default()
                    },
                    Target::Global
                ))
            );
        }
        return menu;
    }
}

impl Widget<()> for VMTabBar {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, _data: &mut (), _env: &druid::Env) {
        match event {
            Event::MouseDown(mouse_event) if mouse_event.button.is_left() => {
                for index in 0..self.tabs.len() {
                    let x_min = *&self.tabs[0..index].iter().fold(0., |acc, v| {
                        return acc + v.2.unwrap().width + TAB_BAR_LABEL_PADDING_X*2.;
                    });
                    let x_max = x_min + self.tabs[index].2.unwrap().width + TAB_BAR_LABEL_PADDING_X*2.;
                    if mouse_event.pos.x > x_min && mouse_event.pos.x < x_max {
                        ctx.submit_command(Command::new(
                            EXECUTE_ACTION,
                            ActionPayload {
                                action: Action::GoToTab,
                                tab_index: Some(index),
                                ..Default::default()
                            },
                            Target::Global
                        ));
                    }
                }
            },
            Event::MouseDown(mouse_event) if mouse_event.button.is_right() => {
                for index in 0..self.tabs.len() {
                    let x_min = *&self.tabs[0..index].iter().fold(0., |acc, v| {
                        return acc + v.2.unwrap().width + TAB_BAR_LABEL_PADDING_X*2.;
                    });
                    let x_max = x_min + self.tabs[index].2.unwrap().width + TAB_BAR_LABEL_PADDING_X*2.;
                    if mouse_event.pos.x > x_min && mouse_event.pos.x < x_max {
                        ctx.show_context_menu(self.build_menu(index), mouse_event.pos);
                    }
                }
            },
            _ => {

            }
        }
    }

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, _data: &(), _env: &druid::Env) {
        match event {
            druid::LifeCycle::FocusChanged(focus) => {
                if *focus {
                    // tracing::debug!("Tab bar has gained focus");
                } else {
                    // tracing::debug!("Tab bar has lost focus");
                }
            }
            _ => ()
        }
    }

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &(), _data: &(), _env: &druid::Env) {
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, _bc: &druid::BoxConstraints, _data: &(), _env: &druid::Env) -> Size {
        for index in 0..self.tabs.len() {
            let mut tab = &mut self.tabs[index];
            if tab.1.is_none() {
                let label = ctx.text().new_text_layout(tab.0.clone())
                .text_color(self.tab_text_color)
                .font(FontFamily::SANS_SERIF, DEFAULT_TAB_LABEL_FONT_SIZE)
                .build()
                .unwrap();
                tab.2 = Some(label.size().ceil());
                tab.1 = Some(label);
            }
        }
        self.tabs.iter().fold(Size::ZERO, |acc, v| {
            return Size {
                height: TAB_BAR_HEIGHT,
                width: acc.width + v.2.unwrap().ceil().width + TAB_BAR_LABEL_PADDING_X*2.
        }
        }).ceil()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, _data: &(), _env: &druid::Env) {
        let size = ctx.size();
        let mut x = 0.0;
        for index in 0..self.tabs.len() {
            let cell_width = self.tabs[index].2.unwrap().ceil().width + TAB_BAR_LABEL_PADDING_X*2.; 
            if index == self.active_tab {
                ctx.fill(Rect::new(x, 0.0, x+cell_width, size.height), &self.tab_background_color_active);
                ctx.fill(Rect::new(
                    x, 
                    size.height-TAB_BAR_INDICATOR_HEIGHT, 
                    x + cell_width,
                    size.height,
                ), &self.tab_active_indicator_color);
            } else {
                ctx.fill(Rect::new(x, 0.0, x+cell_width, size.height), &self.tab_background_color_inactive);
            }
            if index != self.tabs.len()-1 {
                let color = Color::BLACK;
                ctx.paint_with_z_index(5, move |ctx| {
                    ctx.fill(Rect::new(x+cell_width, 0.0, x+cell_width+TAB_DIVIDER_WIDTH, size.height), &color);
                });
            }
            let layout = &self.tabs[index].1.as_mut().unwrap();
            ctx.draw_text(layout, Point::new(x+TAB_BAR_LABEL_PADDING_X, TAB_BAR_LABEL_PADDING_Y));
            x += cell_width;
        }
    }
}