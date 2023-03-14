
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

use druid::{EventCtx, LayoutCtx, piet::{PietTextLayout, Text, TextLayoutBuilder}, Color, FontFamily, PaintCtx, RenderContext, Point};

use crate::{vminput::{ActionPayload, Action}, vmconfig::{VMConfigVersion4, VMColor}, constants::DEFAULT_LABEL_FONT_SIZE};

pub enum CursorStyle {
    None,
    Bar,
    Block
}
pub struct VMTextInput {
    pub(crate) text_buffer: String,
    pub(crate) index: usize,
    pub(crate) text_layout: Option<PietTextLayout>,
}

impl VMTextInput {
    pub fn new() -> Self {
        VMTextInput {
            text_buffer: String::new(),
            index: 0,
            text_layout: None,
        }
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, payload: &ActionPayload) -> Result<(), ()> {
        tracing::debug!("{:?}", payload);
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
                    self.text_buffer.remove(self.index);
                    self.index -= 1;
                }
            },
            Action::DeleteForward => {
                if self.text_buffer.len() < self.index {
                    self.text_buffer.remove(self.index);
                }
            }
            _ => ()
        }
        ctx.request_layout();
        ctx.request_paint();
        return Ok(());
    }

    pub fn insert_character(&mut self, char: &String) {
        self.text_buffer.insert_str(self.index, char);
        self.index += 1;
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, config: &VMConfigVersion4) {
        let layout = ctx.text().new_text_layout(self.text_buffer.clone())
        .text_color(config.get_color(VMColor::LabelTextColor).unwrap())
        .font(FontFamily::SANS_SERIF, DEFAULT_LABEL_FONT_SIZE)
        .build()
        .unwrap();

        self.text_layout = Some(layout);
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx) {
        if let Some(layout) = &self.text_layout {
            ctx.draw_text(layout, Point {x: 0., y: 0.});
        }
    }
}