
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

use druid::{EventCtx, LayoutCtx, piet::{PietTextLayout, TextLayout}, PaintCtx, RenderContext, Point, Rect, BoxConstraints, Size};

use crate::{vminput::{ActionPayload, Action, KeybindMode}, vmconfig::{VMConfigVersion4, VMColor}, constants::{NODE_LABEL_MAX_CONSTRAINTS}, vimmapper::VimMapper};

#[allow(dead_code)]
pub enum CursorStyle {
    None,
    Line,
    Block
}
#[allow(unused)]
pub struct VMTextInput {
    pub(crate) text_buffer: String,
    pub(crate) index: usize,
    pub(crate) text_layout: Option<PietTextLayout>,
    pub(crate) cursor_style: CursorStyle,
}

impl VMTextInput {
    pub fn new() -> Self {
        VMTextInput {
            text_buffer: String::new(),
            index: 0,
            text_layout: None,
            cursor_style: CursorStyle::None,
        }
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, payload: &ActionPayload) -> Result<(), ()> {
        match payload.action {
            Action::InsertCharacter => {
                self.insert_character(&payload.string.as_ref().unwrap());
            },
            Action::DeleteBackspace => {
                if self.index == 0 {
                
                } else if self.index == self.text_buffer.len() {
                    self.text_buffer.pop();
                    self.index -= 1;
                } else {
                    self.text_buffer.remove(self.index-1);
                    self.index -= 1;
                }
            },
            Action::DeleteForward => {
                if self.index < self.text_buffer.len() {
                    self.text_buffer.remove(self.index);
                }
            },
            Action::CursorForward => {
                if self.index < self.text_buffer.len() {
                    self.index+=1;
                }
            },
            Action::CursorBackward => {
                if self.index > 0 {
                    self.index-=1;
                }
            },
            Action::CursorForwardToEndOfWord |
            Action::CursorForwardToBeginningOfWord |
            Action::CursorBackwardToEndOfWord |
            Action::CursorBackwardToBeginningOfWord |
            Action::CursorToNthCharacter |
            Action::DeleteWordWithWhitespace  |
            Action::DeleteWord |
            Action::DeleteToEndOfWord |
            Action::DeleteToNthCharacter |
            Action::DeleteWithNthCharacter |
            Action::ChangeWordWithWhitespace |
            Action::ChangeWord |
            Action::ChangeToEndOfWord |
            Action::ChangeToNthCharacter |
            Action::ChangeWithNthCharacter |
            _ => ()
        }
        ctx.request_layout();
        ctx.request_paint();
        return Ok(());
    }

    pub fn get_block_cursor_bounds(&self) -> Vec<Rect> {
        if let Some(layout) = &self.text_layout {
            if self.text_buffer.len() == 0 {
                let metric = layout.line_metric(0).unwrap();
                return vec![
                    Rect::new(
                        0.,
                        0.,
                        6.,
                        metric.height,
                    )
                ]
            } else if self.index < self.text_buffer.len() && self.index != 0 {
                return layout.rects_for_range(self.index..self.index+1);
            } else if self.index == 0 {
                let rects = layout.rects_for_range(self.index..self.index+1);
                return rects;
            } else if self.index == self.text_buffer.len() {
                let rects = layout.rects_for_range(self.index-1..self.index);
                return vec![
                    Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+6.,
                        rects[0].y1,
                    )
                ]
            } else {
                return vec![Rect::new(0.,0.,0.,0.)];
            }
        } else {
            return vec![Rect::new(0.,0.,0.,0.)];
        }
    }

    pub fn get_line_cursor_bounds(&self) -> Vec<Rect> {
        if let Some(layout) = &self.text_layout {
            if self.text_buffer.len() == 0 {
                let metric = layout.line_metric(0).unwrap();
                return vec![
                    Rect::new(
                        0.,
                        0.,
                        1.,
                        metric.height,
                    )
                ]
            } else if self.index < self.text_buffer.len() && self.index != 0 {
                let rects = layout.rects_for_range(self.index-1..self.index);
                return vec![
                    Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+1.,
                        rects[0].y1,
                    )
                ]
            } else if self.index == 0 {
                let rects = layout.rects_for_range(self.index..self.index+1);
                return vec![
                    Rect::new(
                        rects[0].x0,
                        rects[0].y0,
                        rects[0].x0+1.,
                        rects[0].y1,
                    )
                ]
            } else if self.index == self.text_buffer.len() {
                let rects = layout.rects_for_range(self.index-1..self.index);
                return vec![
                    Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+1.,
                        rects[0].y1,
                    )
                ];
            } else {
                return vec![Rect::new(0.,0.,0.,0.)];
            }
        } else {
            return vec![Rect::new(0.,0.,0.,0.)];
        }
    }

    pub fn insert_character(&mut self, char: &String) {
        self.text_buffer.insert_str(self.index, char);
        self.index += 1;
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, config: &VMConfigVersion4) {
        let mut text = self.text_buffer.clone();
        if self.index == self.text_buffer.len() {text.push_str(" ");}
        let layout = VimMapper::build_label_layout_for_constraints(
            ctx.text(), 
            text,
            BoxConstraints::new(
                        Size::new(0., 0.),
                        Size::new(NODE_LABEL_MAX_CONSTRAINTS.0, NODE_LABEL_MAX_CONSTRAINTS.1)),
            &config.get_color(VMColor::LabelTextColor).unwrap())
            .unwrap();

        self.text_layout = Some(layout);
    }

    pub fn cursor_to_end(&mut self) {
        self.index = self.text_buffer.len();
    }

    pub fn curosr_to_start(&mut self) {
        self.index = 0;
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx, mode: KeybindMode, config: &VMConfigVersion4) {
        if let Some(layout) = &self.text_layout {
            for rect in 
                if mode == KeybindMode::Edit { self.get_line_cursor_bounds() } else { self.get_block_cursor_bounds() } {
                ctx.fill(
                    rect,
                    &config.get_color(VMColor::ComposeIndicatorTextColor).unwrap()
                )
            }
            ctx.draw_text(layout, Point {x: 0., y: 0.});
        }
    }
}