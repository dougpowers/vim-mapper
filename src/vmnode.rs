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
use druid::{Widget, WidgetExt, Vec2, WidgetPod, widget::{Container, Controller, TextBox}, EventCtx, Event, Env, keyboard_types::Key, text::Selection, piet::{PietTextLayout, TextLayout, Text, TextLayoutBuilder}, Rect, PaintCtx, RenderContext, Affine, kurbo::{TranslateScale, RoundedRect}, Point, FontFamily, FontWeight, Color};
use force_graph::DefaultNodeIdx;

use crate::{constants::*, vimmapper::VimMapper, vmconfig::VMConfig};

//Position on the node to paint a badge. Format YPOS_XPOS. Only corners are guaranteed to have space 
// on the layout
pub enum BadgePosition {
    TOP_LEFT,
    TOP_CENTER,
    TOP_RIGHT,
    CENTER_RIGHT,
    BOTTOM_RIGHT,
    BOTTOM_CENTER,
    BOTTOM_LEFT,
    CENTER_LEFT,
}

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
    // Reference the main edges HashMap and filter out the non local node to determine target.
    pub targeted_internal_edge_idx: Option<usize>,
    pub mark: Option<String>,
    //Cached rect of the node, transformed to screen coords. Used to scroll node into view.
    pub node_rect: Option<Rect>,
    pub anchored: bool,
    pub mass: f32,
}

impl Default for VMNode {
    fn default() -> Self {
        let node = VMNode {
            label: DEFAULT_ROOT_LABEL.to_string(),
            edges: Vec::with_capacity(10),
            index: 0,
            fg_index: None,
            pos: Vec2::new(0.0, 0.0),
            container: VMNodeLayoutContainer::new(0),
            is_active: false,
            targeted_internal_edge_idx: None,
            mark: None,
            node_rect: None,
            anchored: false,
            mass: DEFAULT_NODE_MASS,
        };
        node
    }
}

impl VMNode {
    pub fn cycle_target(&mut self) -> Option<u16> {
        if let Some(target) = self.targeted_internal_edge_idx {
            if self.edges.is_empty() {
                // Is root node. No target available
                return None;
            } else if self.edges.len() == 1 {
                //There is only one edge. Cannot change it.
                let edge_idx = self.edges[target];
                return Some(edge_idx);
            } else {
                //There are targets to cycle through
                if self.targeted_internal_edge_idx.unwrap() == self.edges.len()-1 {
                    //Reached the end of the internal edge vector. Cycle through to index 0.
                    self.targeted_internal_edge_idx = Some(0);
                    let edge_idx = self.edges[0];
                    return Some(edge_idx);
                } else {
                    self.targeted_internal_edge_idx = Some(target+1);
                    return Some(self.edges[target+1]);
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

    pub fn set_target_edge_to_global_idx(&mut self, idx: u16) {
        if let Some(internal_target) = self.targeted_internal_edge_idx {
            if self.edges[internal_target] == idx {
                return
            }         
        } else {
            self.edges.iter().enumerate().for_each(|(i, edge_idx)| {
                if *edge_idx == idx {
                    self.targeted_internal_edge_idx = Some(i);
                }
            });
        }

    }

    pub fn set_mark(&mut self, mark: String) {
        if mark == " ".to_string() {
            self.mark = None;
        } else {
            self.mark = Some(mark);
        }
    }

    // pub fn paint_node(&mut self, ctx: &mut PaintCtx, vm: &VimMapper) {
    pub fn paint_node(&mut self, 
        ctx: &mut PaintCtx, 
        config: &VMConfig, 
        target: Option<u16>,
        translate: &TranslateScale,
        scale: &TranslateScale,
        debug_data: bool,
    ) {
        ctx.with_save(|ctx| {
            // let node = vm.nodes.get_mut(&node.data.user_data)
            // .expect("Attempted to retrieve a non-existent node.");
            let label_size = self.container.layout.as_mut()
            .expect("Node layout container was empty.").size();
            ctx.transform(Affine::from(*translate));
            ctx.transform(Affine::from(*scale));
            ctx.transform(Affine::from(TranslateScale::new(-1.0*(label_size.to_vec2())/2.0, 1.0)));
            ctx.transform(Affine::from(TranslateScale::new(self.pos, 1.0)));
            let rect = label_size.to_rect().inflate(DEFAULT_BORDER_WIDTH, DEFAULT_BORDER_WIDTH);
            let border = druid::piet::kurbo::RoundedRect::from_rect(rect, DEFAULT_BORDER_RADIUS);
            //Cache this node's screen space-transformed rect
            self.node_rect = Some(ctx.current_transform().transform_rect_bbox(rect).clone());
            let mut border_color = config.get_color("node-border-color".to_string()).ok().expect("node border color not found in config");
            if self.is_active {
                border_color = config.get_color("active-node-border-color".to_string()).ok().expect("active node border color not found in config");
            } else if let Some(idx) = target {
                if idx == self.index {
                    border_color = config.get_color("target-node-border-color".to_string()).ok().expect("target node border color not found in config");
                }
            }
            ctx.fill(border, &config.get_color("node-background-color".to_string()).ok().expect("node background color not found in config"));
            ctx.stroke(border, &border_color, DEFAULT_BORDER_WIDTH);
            ctx.draw_text(self.container.layout.as_mut().unwrap(), Point::new(0.0, 0.0));

            //Paint mark decals
            // let mark_point = border.origin().to_vec2() + Vec2::new(border.width(), 0.);
            // if let Some(char) = self.mark.clone() {
            //     let layout = ctx.text()
            //     .new_text_layout(char)
            //     .font(FontFamily::SANS_SERIF, 12.)
            //     .text_color(config.get_color("label-text-color".to_string()).ok().expect("label text color not found in config"))
            //     .build().unwrap();
            //     ctx.with_save(|ctx| {
            //         let circle = druid::piet::kurbo::Circle::new(mark_point.to_point(), layout.size().max_side()/1.8);
            //         ctx.with_save(|ctx| {
            //             ctx.fill(circle, &config.get_color("node-background-color".to_string()).ok().expect("node background color not found in config"));
            //             ctx.stroke(circle, &border_color, DEFAULT_MARK_BORDER_WIDTH);
            //         });
            //         ctx.transform(Affine::from(TranslateScale::new(-1.*layout.size().to_vec2()/2., 1.)));
            //         ctx.draw_text(&layout, mark_point.to_point());
            //     });
            // }
            if let Some(char) = self.mark.clone() {
                self.paint_node_badge(ctx, config, &char, BadgePosition::TOP_RIGHT, &border, &border_color);
            }

            if self.mass.clone() > DEFAULT_NODE_MASS {
                self.paint_node_badge(ctx, config, &"+".to_string(), BadgePosition::BOTTOM_CENTER, &border, &border_color);
            } else if self.mass.clone() < DEFAULT_NODE_MASS {
                self.paint_node_badge(ctx, config, &"-".to_string(), BadgePosition::BOTTOM_CENTER, &border, &border_color);
            }

            if self.anchored {
                self.paint_node_badge(ctx, config, &"@".to_string(), BadgePosition::BOTTOM_LEFT, &border, &border_color)
            }

            //Paint debug decals (node index)
            if debug_data {
                ctx.transform(Affine::from(TranslateScale::new(Vec2::new(-10., -10.), 1.)));
                let index_debug_decal = ctx.text()
                .new_text_layout(self.index.to_string())
                .font(FontFamily::SANS_SERIF, 12.)
                .default_attribute(
                    FontWeight::BOLD
                )
                .text_color(Color::RED)
                .build();
                ctx.draw_text(&index_debug_decal.unwrap(), Point::new(0., 0.));
            }
        });
    }

    //Paints a given character badge at a given BadgePosition.
    // NOTE: Expects to be executed within a context transformation to node-local position
    // TODO: End transformation and reexecute it within function
    pub fn paint_node_badge(&mut self,
         ctx: &mut PaintCtx,
         config: &VMConfig, 
         character: &String,
         position: BadgePosition, 
         border: &RoundedRect, 
         border_color: &Color,
        ) {
        let mark_point: Vec2;
        match position {
            BadgePosition::TOP_LEFT => {
                mark_point = border.origin().to_vec2();
            }
            BadgePosition::TOP_CENTER => {
                mark_point = border.origin().to_vec2() + Vec2::new(border.width()/2., 0.);
            }
            BadgePosition::TOP_RIGHT => {
                mark_point = border.origin().to_vec2() + Vec2::new(border.width(), 0.);
            }
            BadgePosition::CENTER_RIGHT => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(border.width(), 0.) +
                Vec2::new(0., border.height()/2.);
            }
            BadgePosition::BOTTOM_RIGHT => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(border.width(), 0.) +
                Vec2::new(0., border.height());
            }
            BadgePosition::BOTTOM_CENTER => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(border.width()/2., 0.) +
                Vec2::new(0., border.height());
            }
            BadgePosition::BOTTOM_LEFT => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(0., border.height());
            }
            BadgePosition::CENTER_LEFT => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(0., border.height()/2.);
            }
        }

        let layout = ctx.text()
        .new_text_layout(character.clone())
        .font(FontFamily::SANS_SERIF, 12.)
        .text_color(config.get_color("label-text-color".to_string()).ok().expect("label text color not found in config"))
        .build().unwrap();
        ctx.with_save(|ctx| {
            let circle = druid::piet::kurbo::Circle::new(mark_point.to_point(), layout.size().max_side()/1.8);
            ctx.with_save(|ctx| {
                ctx.fill(circle, &config.get_color("node-background-color".to_string()).ok().expect("node background color not found in config"));
                ctx.stroke(circle, border_color, DEFAULT_MARK_BORDER_WIDTH);
            });
            ctx.transform(Affine::from(TranslateScale::new(-1.*layout.size().to_vec2()/2., 1.)));
            ctx.draw_text(&layout, mark_point.to_point());
        });
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
    //Cached rect of the editor, transformed to screen coordinates. Used to scroll editor into view.
    pub editor_rect: Option<Rect>,
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
            editor_rect: None,
        };
        nodeeditor
    }
}

#[derive(Debug)]
pub struct VMNodeLayoutContainer {
    pub layout: Option<PietTextLayout>,
    pub index: u16,
}

impl VMNodeLayoutContainer {
    pub fn new(index: u16) -> VMNodeLayoutContainer {
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
                let selection = Selection::new(0,usize::MAX);
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