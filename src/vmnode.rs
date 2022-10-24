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
use druid::{Widget, WidgetExt, Vec2, WidgetPod, widget::{Container, Controller, TextBox}, EventCtx, Event, Env, keyboard_types::Key, text::{Selection}, piet::{PietTextLayout, TextLayout, Text, TextLayoutBuilder}, Rect, PaintCtx, RenderContext, Affine, kurbo::TranslateScale, Point, FontFamily, FontWeight, Color};
use force_graph::DefaultNodeIdx;

use crate::{constants::*, vmconfig::*};

//Position on the node to paint a badge. Format YposXpos. Only corners are guaranteed to have space 
// on the layout
#[allow(dead_code)]
pub enum BadgePosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterRight,
    BottomRight,
    BottomCenter,
    BottomLeft,
    CenterLeft,
}

#[derive(Debug)]
pub struct VMNode {
    pub label: String,
    pub edges: Vec<u16>,
    pub index: u16,
    pub fg_index: Option<DefaultNodeIdx>,
    // pub pos: Vec2,
    pub enabled_layout: Option<PietTextLayout>,
    pub disabled_layout: Option<PietTextLayout>,
    pub is_active: bool,
    pub mark: Option<String>,
    //Cached rect of the node, transformed to screen coords. Used to scroll node into view.
    pub node_rect: Option<Rect>,
    pub anchored: bool,
    pub mass: f64,
}

impl Default for VMNode {
    fn default() -> Self {
        let label = DEFAULT_ROOT_LABEL.to_string();
        let node = VMNode {
            label,
            edges: Vec::with_capacity(10),
            index: 0,
            fg_index: None,
            // pos: Vec2::new(0.0, 0.0),
            // container: VMNodeLayoutContainer::new(0),
            enabled_layout: None,
            disabled_layout: None,
            is_active: false,
            mark: None,
            node_rect: None,
            anchored: false,
            mass: DEFAULT_NODE_MASS,
        };
        node
    }
}

impl VMNode {
    pub fn set_mark(&mut self, mark: String) {
        if mark == " ".to_string() {
            self.mark = None;
        } else {
            self.mark = Some(mark);
        }
    }

    // pub fn paint_node(&mut self, ctx: &mut PaintCtx, vm: &VimMapper) {
    pub fn paint_node(
        &mut self, 
        ctx: &mut PaintCtx, 
        z_index: u32,
        enabled: bool,
        config: &VMConfigVersion4, 
        target: Option<u16>,
        pos: Vec2,
        translate: &TranslateScale,
        scale: &TranslateScale,
        debug_data: bool,
    ) {
        ctx.with_save(|ctx| {
            // let node = vm.nodes.get_mut(&node.data.user_data)
            // .expect("Attempted to retrieve a non-existent node.");
            let label_size = self.enabled_layout.as_mut()
            .expect("Node layout container was empty.").size();
            ctx.transform(Affine::from(*translate));
            ctx.transform(Affine::from(*scale));
            ctx.transform(Affine::from(TranslateScale::new(-1.0*(label_size.to_vec2())/2.0, 1.0)));
            ctx.transform(Affine::from(TranslateScale::new(pos, 1.0)));
            let rect = label_size.to_rect().inflate(DEFAULT_BORDER_WIDTH, DEFAULT_BORDER_WIDTH);
            let border = druid::piet::kurbo::RoundedRect::from_rect(rect, DEFAULT_BORDER_RADIUS);
            //Cache this node's screen space-transformed rect
            self.node_rect = Some(ctx.current_transform().transform_rect_bbox(rect).clone());
            let mut border_color = config.get_color(VMColor::NodeBorderColor).ok().expect("node border color not found in config");
            let mut border_width = DEFAULT_BORDER_WIDTH;
            if self.is_active {
                border_color = config.get_color(VMColor::ActiveNodeBorderColor).ok().expect("active node border color not found in config");
                border_width = DEFAULT_ACTIVE_BORDER_WIDTH;
            } else if let Some(idx) = target {
                if idx == self.index {
                    border_color = config.get_color(VMColor::TargetNodeBorderColor).ok().expect("target node border color not found in config");
                    border_width = DEFAULT_TARGET_BORDER_WIDTH;
                } 
            }

            let border_background;
            let mut container;
            if !enabled {
                container = self.disabled_layout.clone();
                border_background = config.get_color(VMColor::DisabledNodeBackgroundColor).ok().expect("DIsabled node background color not found in config");
            } else {
                container = self.enabled_layout.clone();
                border_background = config.get_color(VMColor::NodeBackgroundColor).ok().expect("Node background color not found in config");
            }

            let badge_border_color = border_color.clone();

            // ctx.paint_with_z_index(z_index, move |ctx| {
                ctx.stroke(border, &border_color, border_width);
                ctx.fill(border, &border_background);
                ctx.draw_text(container.as_mut().unwrap(), Point::new(0.0, 0.0));
            // });

            if let Some(char) = self.mark.clone() {
                self.paint_node_badge(ctx, z_index, enabled, config, &char, BadgePosition::TopRight, &rect, &badge_border_color);
            }

            if self.mass.clone() > DEFAULT_NODE_MASS {
                self.paint_node_badge(ctx, z_index, enabled, config, &"+".to_string(), BadgePosition::BottomCenter, &rect, &badge_border_color);
            } else if self.mass.clone() < DEFAULT_NODE_MASS {
                self.paint_node_badge(ctx, z_index, enabled, config, &"-".to_string(), BadgePosition::BottomCenter, &rect, &badge_border_color);
            }

            if self.anchored {
                self.paint_node_badge(ctx, z_index, enabled, config, &"@".to_string(), BadgePosition::BottomLeft, &rect, &badge_border_color)
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
         _z_index: u32,
         enabled: bool,
         config: &VMConfigVersion4, 
         character: &String,
         position: BadgePosition, 
         border: &Rect, 
         border_color: &Color,
    ) {
        let border = border.inflate(
            BADGE_BORDER_INFLATION_AMOUNT,
            BADGE_BORDER_INFLATION_AMOUNT,
        );
        let mark_point: Vec2;
        match position {
            BadgePosition::TopLeft => {
                mark_point = border.origin().to_vec2();
            }
            BadgePosition::TopCenter => {
                mark_point = border.origin().to_vec2() + Vec2::new(border.width()/2., 0.);
            }
            BadgePosition::TopRight => {
                mark_point = border.origin().to_vec2() + Vec2::new(border.width(), 0.);
            }
            BadgePosition::CenterRight => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(border.width(), 0.) +
                Vec2::new(0., border.height()/2.);
            }
            BadgePosition::BottomRight => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(border.width(), 0.) +
                Vec2::new(0., border.height());
            }
            BadgePosition::BottomCenter => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(border.width()/2., 0.) +
                Vec2::new(0., border.height());
            }
            BadgePosition::BottomLeft => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(0., border.height());
            }
            BadgePosition::CenterLeft => {
                mark_point = border.origin().to_vec2() + 
                Vec2::new(0., border.height()/2.);
            }
        }

        let layout = ctx.text()
        .new_text_layout(character.clone())
        .font(FontFamily::SANS_SERIF, 12.)
        .text_color(
            if enabled {
                config.get_color(VMColor::LabelTextColor).ok().expect("label text color not found in config")
            } else {
                config.get_color(VMColor::DisabledLabelTextColor).ok().expect("label text color not found in config")
            })
        .build().unwrap();
        ctx.with_save(move |ctx| {
            let circle = druid::piet::kurbo::Circle::new(mark_point.to_point().clone(), layout.size().max_side()/1.8);
            let background_color = if enabled {
                config.get_color(VMColor::NodeBackgroundColor).ok().expect("badge background color not found in config")
            } else {
                config.get_color(VMColor::DisabledNodeBackgroundColor).ok().expect("badge background color not found in config")
            };
            let badge_border_color = border_color.clone();
            ctx.with_save(|ctx| {
                // ctx.paint_with_z_index(z_index, move |ctx| {
                    ctx.fill(circle, &background_color);
                    ctx.stroke(circle, &badge_border_color, DEFAULT_MARK_BORDER_WIDTH);
                // });
            });
            ctx.transform(Affine::from(TranslateScale::new(-1.*layout.size().to_vec2()/2., 1.)));
            // ctx.paint_with_z_index(z_index, move |ctx| {
                ctx.draw_text(&layout, mark_point.to_point());
            // });
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
        let textbox = TextBox::<String>::multiline().with_text_size(DEFAULT_LABEL_FONT_SIZE)
        .controller(
            VMNodeEditorController::new()
        ).expand_width();
        let widget = Container::new(
            textbox
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