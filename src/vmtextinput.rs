
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

use druid::{EventCtx, LayoutCtx, piet::{PietTextLayout, TextLayout}, PaintCtx, RenderContext, Point, Rect, BoxConstraints, Size, text::{EditableText}, Color};

use crate::{vminput::{ActionPayload, Action, KeybindMode}, vmconfig::{VMConfigVersion4, VMColor}, constants::{NODE_LABEL_MAX_CONSTRAINTS, DEFUALT_TEXT_CURSOR_WIDTH}, vimmapper::VimMapper};

use unicode_segmentation::*;

#[allow(unused)]
pub struct VMTextInput {
    pub(crate) text: String,
    index: usize,
    visual_anchor: Option<usize>,
    pub(crate) text_layout: Option<PietTextLayout>,
    pub(crate) mode: KeybindMode,
}

pub trait BackwardTextSearch {
    fn next_word_start_offset(&self, index: usize) -> Option<usize>;
    fn prev_word_start_offset(&self, index: usize) -> Option<usize>;
    fn next_word_end_offset(&self, index: usize) -> Option<usize>;
}

impl BackwardTextSearch for String {
    fn prev_word_start_offset(&self, index: usize) -> Option<usize> {
        let mut words = self.split_word_bound_indices().collect::<Vec<(usize, &str)>>();
        words.reverse();
        let mut iter = words.into_iter();
        while let Some((i, word)) = iter.next() {
            if i < index && !word.contains(char::is_whitespace) {
                return Some(i);
            }
        }
        return None;
    }

    fn next_word_start_offset(&self, index: usize) -> Option<usize> {
        let mut words = self.split_word_bound_indices();
        while let Some((i, word)) = words.next() {
            if i > index && !word.contains(char::is_whitespace) {
                return Some(i);
            }
        }
        return None;
    }

    fn next_word_end_offset(&self, index: usize) -> Option<usize> {
        if let Some(next_index) = self.next_grapheme_offset(index) {
            let mut words = self.split_word_bound_indices();
            while let Some((i, word)) = words.next() {
                if i+word.len() > next_index && !word.contains(char::is_whitespace) {
                    return self.prev_grapheme_offset(i+word.len());
                }
            }
        }
        return None;
    }
}

#[allow(unused_must_use)]
impl<'a> VMTextInput {
    pub fn new() -> Self {
        let text = String::new();
        VMTextInput {
            text,
            index: 0,
            visual_anchor: None,
            text_layout: None,
            mode: KeybindMode::EditBrowse,
        }
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, payload: &ActionPayload) -> Result<(), ()> {
        match payload.action {
            Action::InsertCharacter => {
                self.insert_character(payload.string.clone().unwrap());
            },
            Action::DeleteBackspace => {
                if let Ok(_) = self.cursor_backward() {
                    self.delete_character();
                }
            },
            Action::DeleteForward => {
                self.delete_character();
            },
            Action::CursorForward => {
                self.cursor_forward();
            },
            Action::CursorBackward => {
                self.cursor_backward();
            },
            Action::CursorForwardToEndOfWord => {
                self.set_cursor(self.text.next_word_end_offset(self.index));
            },
            Action::CursorForwardToBeginningOfWord => {
                self.set_cursor(self.text.next_word_start_offset(self.index));
            },
            Action::CursorBackwardToBeginningOfWord => {
                self.set_cursor(self.text.prev_word_start_offset(self.index));
            },
            Action::DeleteWord |
            Action::DeleteToNthCharacter |
            Action::DeleteWithNthCharacter |
            Action::ChangeWord |
            Action::ChangeToNthCharacter |
            Action::ChangeWithNthCharacter |
            Action::CursorToNthCharacter |
            Action::CursorBackwardToEndOfWord |
            _ => ()
        }
        if (self.mode == KeybindMode::EditBrowse ||
            self.mode == KeybindMode::EditVisual) && self.index == self.text.len() {
                self.cursor_backward();
            } 
        ctx.request_layout();
        ctx.request_paint();
        return Ok(());
    }

    pub fn cursor_forward(&mut self) -> Result<(), ()> {
        if let Some(new_index) = self.text.next_grapheme_offset(self.index) { 
            if self.mode == KeybindMode::EditBrowse || self.mode == KeybindMode::EditVisual {
                if let Some(_) = self.text.next_grapheme_offset(new_index) {
                    self.index = new_index; 
                    return Ok(());
                } else {
                    return Err(());
                }
            } else {
                self.index = new_index; 
                return Ok(());
            }
        } else {
            return Err(());
        }
    }

    pub fn cursor_backward(&mut self) -> Result<(), ()> {
        if let Some(new_index) = self.text.prev_grapheme_offset(self.index) { 
            self.index = new_index; return Ok(());
        } else {
            return Err(());
        }
    }

    pub fn set_cursor(&mut self, index: Option<usize>) -> Result<(), ()> {
        if let Some(i) = index {
            if let Some(_) = self.text.cursor(i) {
                self.index = i;
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn delete_character(&mut self) -> Result<(), ()> {
        if self.text.next_grapheme_offset(self.index).is_some() {
            self.text.edit(
                self.index..self.text.next_grapheme_offset(self.index).unwrap(),
                ""
            );
            return Ok(());
        } else {
            return Err(());
        }
    }

    pub fn insert_character(&mut self, string: String) {
        self.text.insert_str(self.index, string.as_str());
        self.index = self.text.next_grapheme_offset(self.index).unwrap();
    }

    pub fn get_block_cursor_bounds(&self, index: usize) -> Vec<Rect> {
        if let Some(layout) = &self.text_layout {
            if self.text.is_empty() {
                let metric = layout.line_metric(0).unwrap();
                return vec![
                    Rect::new(
                        0.,
                        0.,
                        6.,
                        metric.height,
                    )
                ]
            } else if let Some(next_index) = self.text.next_grapheme_offset(index) {
                let rects = layout.rects_for_range(index..next_index);
                return rects;
            } else {
                let rects = layout.rects_for_range(self.text.prev_grapheme_offset(index).unwrap()..index);
                return vec![
                    Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+6.,
                        rects[0].y1,
                    )
                ]
            }
        } else {
            return vec![Rect::new(0.,0.,0.,0.)];
        }
    }

    pub fn get_line_cursor_bounds(&self, index: usize) -> Vec<Rect> {
        if let Some(layout) = &self.text_layout {
            if self.text.is_empty() {
                let metric = layout.line_metric(0).unwrap();
                return vec![
                    Rect::new(
                        0.,
                        0.,
                        DEFUALT_TEXT_CURSOR_WIDTH,
                        metric.height,
                    )
                ]
            } else if let Some(next_index) = self.text.next_grapheme_offset(index) {
                let rects = layout.rects_for_range(index..next_index);
                return vec![
                    Rect::new(
                        rects[0].x0,
                        rects[0].y0,
                        rects[0].x0+DEFUALT_TEXT_CURSOR_WIDTH,
                        rects[0].y1,
                    ),
                ]
            } else {
                let rects = layout.rects_for_range(self.text.prev_grapheme_offset(index).unwrap()..index);
                return vec![
                    Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+DEFUALT_TEXT_CURSOR_WIDTH,
                        rects[0].y1,
                    )
                ]
            }
        } else {
            return vec![Rect::new(0.,0.,0.,0.)];
        }
    }

    pub fn cursor_to_end(&mut self) {
        self.index = self.text.len();
    }

    pub fn curosr_to_start(&mut self) {
        self.index = 0;
    }

    pub fn set_keybind_mode(&mut self, mode: KeybindMode) {
        match mode {
            KeybindMode::Edit => {
                self.visual_anchor = None;
            }
            KeybindMode::EditBrowse => {
                self.visual_anchor = None;
                if let Some(_) = self.text.next_grapheme_offset(self.index) {
                } else {
                    self.cursor_backward();
                }
            },
            KeybindMode::EditVisual => {
                self.visual_anchor = Some(self.index);
            }
            _ => ()
        }
        self.mode = mode;
    }

    #[allow(dead_code)]
    pub fn get_keybind_mode(&self) -> KeybindMode {
        return self.mode;
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, config: &VMConfigVersion4) {
        let layout = VimMapper::build_label_layout_for_constraints(
            ctx.text(), 
            self.text.clone(),
            BoxConstraints::new(
                        Size::new(0., 0.),
                        Size::new(NODE_LABEL_MAX_CONSTRAINTS.0, NODE_LABEL_MAX_CONSTRAINTS.1)),
            &config.get_color(VMColor::LabelTextColor).unwrap())
            .unwrap();

        self.text_layout = Some(layout);
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx, config: &VMConfigVersion4, debug: bool) {
        if let Some(layout) = &self.text_layout {
            for rect in 
                match self.mode {
                    KeybindMode::Edit => { self.get_line_cursor_bounds(self.index) },
                    KeybindMode::EditBrowse => { self.get_block_cursor_bounds(self.index) },
                    KeybindMode::EditVisual => { self.get_block_cursor_bounds(self.index) },
                    _ => { vec![] },
                } {
                ctx.fill(
                    rect,
                    &config.get_color(VMColor::TextCursorColor).unwrap()
                )
            }
            if debug {
                let mut index: usize = 0;
                loop {
                    if let Some(next_index) = self.text.next_word_offset(index) {
                        index = next_index;
                    } else {
                        break;
                    }
                    for rect in self.get_line_cursor_bounds(index) {
                        ctx.fill(
                            rect,
                            &Color::RED
                        );
                    }
                    if index == self.text.len() {
                        break;
                    }
                }

            }
            ctx.draw_text(layout, Point {x: 0., y: 0.});
        }
    }
}