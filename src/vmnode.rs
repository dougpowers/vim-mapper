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
use druid::{Widget, WidgetExt, Vec2, WidgetPod, widget::{Container, Controller, TextBox}, EventCtx, Event, Env, keyboard_types::Key, text::Selection};
use force_graph::DefaultNodeIdx;
#[cfg(windows)]
use druid::piet::D2DTextLayout;
#[cfg(unix)]
use druid::piet::CairoTextLayout;


use crate::constants::*;

#[derive(Debug)]
pub struct VMNode {
    pub label: String,
    pub edges: Vec<u16>,
    pub index: u16,
    pub fg_index: Option<DefaultNodeIdx>,
    pub pos: Vec2,
    pub container: VMNodeLayoutContainer,
    pub is_active: bool,
    //The index to the internal 'edges' array that corresponds to the target edge. 
    // Reference the main edges HashMap filter out the non local node to determine target.
    pub targeted_internal_edge_idx: Option<usize>,
}

impl VMNode {
    pub fn cycle_target(&mut self) -> Option<u16> {
        if let Some(target) = self.targeted_internal_edge_idx {
            if self.edges.is_empty() {
                return None;
            } else if self.edges.len() == 1 {
                let edge_idx = self.edges[target];
                return Some(edge_idx);
            } else {
                if self.targeted_internal_edge_idx.unwrap() == self.edges.len()-1 {
                    self.targeted_internal_edge_idx = Some(0);
                    let edge_idx = self.edges[target];
                    return Some(edge_idx);
                } else {
                    self.targeted_internal_edge_idx = Some(target+1);
                    return Some(self.edges[target]);
                }
            }
        } else {
            if self.edges.is_empty() {
                None
            } else {
                self.targeted_internal_edge_idx = Some(0);
                return Some(self.edges[self.targeted_internal_edge_idx.unwrap()]);
            }
        }
    }
}

#[derive(Debug)]
pub struct VMEdge {
    pub label: Option<String>,
    pub from: u16,
    pub to: u16,
    pub index: u16,
}

pub struct VMNodeEditor {
    pub container: WidgetPod<String, Container<String>>,
    pub is_visible: bool,
    pub title_text: String,
}

impl VMNodeEditor {
    pub fn new() -> VMNodeEditor {
        let widget = Container::new(
            TextBox::<String>::new().controller(
                VMNodeEditorController::new()
            ).expand_width()
        );
            
        let nodeeditor = VMNodeEditor {
            container: WidgetPod::new(widget),
            is_visible: false,
            title_text: "".to_string(),
        };
        nodeeditor
    }
}

#[cfg(windows)]
#[derive(Debug)]
pub struct VMNodeLayoutContainer {
    pub layout: Option<D2DTextLayout>,
    pub index: u16,
}

#[cfg(unix)]
#[derive(Debug)]
pub struct VMNodeLayoutContainer {
    pub layout: Option<CairoTextLayout>,
    pub index: u16,
}

impl VMNodeLayoutContainer {
    pub fn new(_label: String, index: u16) -> VMNodeLayoutContainer {
        let new_layout = VMNodeLayoutContainer {
            layout: None,
            index,
        };
        new_layout
    }
}

pub struct VMNodeEditorController {}

impl VMNodeEditorController {
    pub fn new() -> VMNodeEditorController {
        VMNodeEditorController {}
    }
}

impl Controller<String, TextBox<String>> for VMNodeEditorController {
    fn event(&mut self, child: &mut TextBox<String>, ctx: &mut EventCtx, event: &Event, data: &mut String, env: &Env) {
        match event {
            Event::KeyDown(event) if event.key == Key::Enter => {
                ctx.submit_notification(SUBMIT_CHANGES);
                ctx.resign_focus();
                ctx.set_handled();
            }
            Event::KeyDown(event) if event.key == Key::Escape => {
                ctx.submit_notification(CANCEL_CHANGES);
                ctx.resign_focus();
                ctx.set_handled();
            }
            Event::Command(command) if command.is(TAKE_FOCUS) => {
                ctx.request_focus();
                ctx.set_handled();
                let selection = Selection::new(0,1000);
                // if let Some(text) = child.editor().layout().text() {
                //     selection = selection.constrained(text);
                //     selection.end = selection.max();
                // }
                child.set_selection(selection);
                child.event(ctx, event, data, env);
                child.set_text_alignment(druid::TextAlignment::Start);
            }
            Event::MouseDown(_event) => {
                ctx.submit_notification(TAKEN_FOCUS);
                child.event(ctx, event, data, env);
                ctx.set_handled();
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }
    fn lifecycle(
            &mut self,
            child: &mut TextBox<String>,
            ctx: &mut druid::LifeCycleCtx,
            event: &druid::LifeCycle,
            data: &String,
            env: &Env,
        ) {
        child.lifecycle(ctx, event, data, env);
    }
    fn update(&mut self, child: &mut TextBox<String>, ctx: &mut druid::UpdateCtx, old_data: &String, data: &String, env: &Env) {
        child.update(ctx, old_data, data, env);
    }
}