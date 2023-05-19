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

use druid::{Vec2, piet::{PietTextLayout, TextLayout, Text, TextLayoutBuilder}, Rect, PaintCtx, RenderContext, Affine, kurbo::TranslateScale, Point, FontFamily, FontWeight, Color};
use serde::{Serialize, Deserialize};
use vm_force_graph_rs::{DefaultNodeIdx, ForceGraph};

use crate::{constants::*, vmconfig::*, vmtextinput::VMTextInput};

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

#[derive(Serialize, Deserialize)]
#[serde(remote = "Rect")]
struct RectDef {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMNode {
    label: String,
    pub index: u32,
    pub fg_index: Option<DefaultNodeIdx>,
    pub is_active: bool,
    pub mark: Option<String>,
    // #[serde(skip)]
    // pub text_cursor_index: usize,
    #[serde(skip)]
    pub text_input: VMTextInput,
    //Cached rect of the node, transformed to screen coords. Used to scroll node into view.
    #[serde(with = "RectDef")]
    pub node_rect: Rect,
}

impl Default for VMNode {
    fn default() -> Self {
        let label = DEFAULT_ROOT_LABEL.to_string();
        let node = VMNode {
            label: label.clone(),
            index: 0,
            fg_index: None,
            is_active: false,
            mark: None,
            text_input: VMTextInput::new(label, None),
            // text_cursor_index: 0,
            node_rect: Rect::new(0.,0.,0.,0.),
        };
        node
    }
}

impl VMNode {
    #[allow(dead_code)]
    pub fn new(label: String) -> Self {
        let node = VMNode {
            label: label.clone(),
            index: 0,
            fg_index: None,
            is_active: false,
            mark: None,
            text_input: VMTextInput::new(label, None),
            // text_cursor_index: 0,
            node_rect: Rect::new(0.,0.,0.,0.),
        };
        node
    }

    pub fn with_fields(label: String, index: u32, fg_index: Option<DefaultNodeIdx>, mark: Option<String>, is_active: bool) -> Self {
        VMNode {
            label: label.clone(),
            index,
            fg_index,
            mark,
            is_active,
            text_input: VMTextInput::new(label, None),
            ..Default::default()
        }
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }

    pub fn save_text(&mut self) {
        self.label = self.text_input.get_text();
        self.text_input.push_history();
    }

    pub fn undo(&mut self) {
        self.text_input.undo();
        self.label = self.text_input.get_text();
    }

    pub fn redo(&mut self) {
        self.text_input.redo();
        self.label = self.text_input.get_text();
    }

    pub fn load_input_text(&mut self) {
        self.text_input.set_text(self.label.clone());
    }

    pub fn paint_node(
        &mut self, 
        ctx: &mut PaintCtx, 
        z_index: u32,
        graph: &ForceGraph<u32, u32>,
        enabled: bool,
        //enable screen-space rect caching (don't do this if drawing as a list member) to avoid polluting the cache with incorrect coords
        set_rect: bool,
        provided_layout: Option<&PietTextLayout>,
        config: &VMConfigVersion4, 
        target: Option<u32>,
        pos: Vec2,
        translate: &TranslateScale,
        scale: &TranslateScale,
        debug_data: bool,
    ) {
        ctx.with_save(|ctx| {
            // let label_size = self.enabled_layout.as_mut()
            // .expect("Node layout container was empty.").size();
            let layout: &PietTextLayout;
            if let Some(provided_layout) = provided_layout {
                layout = provided_layout;
            } else {
                layout = &self.text_input.text_layout.as_ref().unwrap();
            }
            let mut label_size = layout.size();
            if label_size.width < DEFAULT_MIN_NODE_WIDTH_DATA {
                label_size.width = DEFAULT_MIN_NODE_WIDTH_DATA;
            }
            ctx.transform(Affine::from(*translate));
            ctx.transform(Affine::from(*scale));
            ctx.transform(Affine::from(TranslateScale::new(-1.0*(label_size.to_vec2())/2.0, 1.0)));
            ctx.transform(Affine::from(TranslateScale::new(pos, 1.0)));
            let rect = label_size.to_rect().inflate(DEFAULT_BORDER_WIDTH, DEFAULT_BORDER_WIDTH);
            let border = druid::piet::kurbo::RoundedRect::from_rect(rect, DEFAULT_BORDER_RADIUS);
            //Cache this node's screen space-transformed rect (only if not drawn as a list member)
            if set_rect {
                self.node_rect = ctx.current_transform().transform_rect_bbox(rect).clone();
            }
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
            if !enabled {
                border_background = config.get_color(VMColor::DisabledNodeBackgroundColor).ok().expect("DIsabled node background color not found in config");
            } else {
                border_background = config.get_color(VMColor::NodeBackgroundColor).ok().expect("Node background color not found in config");
            }

            let badge_border_color = border_color.clone();

            // ctx.paint_with_z_index(z_index, move |ctx| {
                ctx.stroke(border, &border_color, border_width);
                ctx.fill(border, &border_background);
                ctx.draw_text(layout, Point::new(0.0, 0.0));
            // });

            if let Some(char) = self.mark.clone() {
                self.paint_node_badge(ctx, z_index, graph, enabled, config, &char, BadgePosition::TopRight, &rect, &badge_border_color);
            }

            if graph.get_graph()[self.fg_index.unwrap()].data.mass > DEFAULT_NODE_MASS {
                self.paint_node_badge(ctx, z_index, graph, enabled, config, &"+".to_string(), BadgePosition::BottomCenter, &rect, &badge_border_color);
            } else if graph.get_graph()[self.fg_index.unwrap()].data.mass < DEFAULT_NODE_MASS {
                self.paint_node_badge(ctx, z_index, graph, enabled, config, &"-".to_string(), BadgePosition::BottomCenter, &rect, &badge_border_color);
            }

            if graph.get_graph()[self.fg_index.unwrap()].data.is_anchor {
                self.paint_node_badge(ctx, z_index, graph, enabled, config, &"⚓".to_string(), BadgePosition::BottomLeft, &rect, &badge_border_color)
            }

            //Paint debug decals (node index)
            if debug_data {
                ctx.transform(Affine::from(TranslateScale::new(Vec2::new(-10., -10.), 1.)));
                let index_debug_decal = ctx.text()
                .new_text_layout(
                    format!("{} ({})", self.index, self.fg_index.unwrap().index())
                )
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
         _graph: &ForceGraph<u32, u32>,
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