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

use std::ops::Range;

use druid::{EventCtx, LayoutCtx, piet::{PietTextLayout, TextLayout, Text, TextLayoutBuilder}, PaintCtx, RenderContext, Point, Rect, BoxConstraints, Size, text::{EditableText}, Color, FontFamily, Vec2, Affine};

use crate::{vminput::{ActionPayload, Action, KeybindMode, TextOperation, TextObj, TextMotion}, vmconfig::{VMConfigVersion4, VMColor}, constants::{NODE_LABEL_MAX_CONSTRAINTS, DEFUALT_TEXT_CURSOR_WIDTH}, vimmapper::VimMapper};

use unicode_segmentation::*;

#[allow(unused)]
pub struct VMTextInput {
    pub(crate) text: String,
    index: usize,
    visual_anchor: Option<usize>,
    unconfirmed_range: Option<Range<usize>>,
    pub(crate) text_layout: Option<PietTextLayout>,
    pub(crate) mode: KeybindMode,
}

pub trait VMTextSearch {
    fn next_word_start_offset(&self, index: usize) -> Option<usize>;
    fn prev_word_start_offset(&self, index: usize) -> Option<usize>;
    fn next_word_end_offset(&self, index: usize) -> Option<usize>;
    fn prev_word_end_offset(&self, index: usize) -> Option<usize>;
    fn current_word_bounds(&self, index: usize) -> Range<usize>;
    fn next_occurrence(&self, index: usize, grapheme: String) -> Option<usize>;
    fn prev_occurrence(&self, index: usize, grapheme: String) -> Option<usize>;
}

impl VMTextSearch for String {
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

    fn prev_word_end_offset(&self, index: usize) -> Option<usize> {
        if let Some(next_index) = self.next_grapheme_offset(index) {
            let mut words = self.split_word_bound_indices();
            while let Some((i, word)) = words.next_back() {
                if i+word.len() > next_index && !word.contains(char::is_whitespace) {
                    return self.prev_grapheme_offset(i);
                }
            }
        }
        return None;
    }

    fn current_word_bounds(&self, index: usize) -> Range<usize> {
        let mut words = self.split_word_bound_indices().peekable();
        while let Some((i, _)) = words.next() {
            if let Some((next, _)) = words.peek() {
                if i <= index && *next > index {
                    return i..*next;
                }
            } else {
                return i..self.len();
            }
        }
        return index..index;
    }

    fn next_occurrence(&self, index: usize, grapheme: String) -> Option<usize> {
        if let Some(next_index) = self.next_grapheme_offset(index) {
            if let Some(slice) = self.slice(next_index..self.len()) {
                let mut graphemes = slice.grapheme_indices(true);
                while let Some((i, graph)) = graphemes.next() {
                    if graph == grapheme { 
                        return Some(i+next_index); 
                    }
                }
            }
        }
        return None;
    }

    fn prev_occurrence(&self, index: usize, grapheme: String) -> Option<usize> {
        if let Some(slice) = self.slice(0..index) {
            let mut graphemes = slice.grapheme_indices(true);
            while let Some((i, graph)) = graphemes.next_back() {
                if graph == grapheme { 
                    return Some(i); 
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
            unconfirmed_range: None,
            text_layout: None,
            mode: KeybindMode::Edit,
        }
    }

    pub fn handle_action(&mut self, ctx: &mut EventCtx, payload: &ActionPayload) -> Option<KeybindMode> {
        // Some text to test vim actions
        let mut change_mode = None;
        match payload.action {
            Action::ExecuteTextAction => {
                if let Some(text_action) = &payload.text_action {
                    if text_action.operation == TextOperation::ChangeText {
                        change_mode = Some(KeybindMode::Insert);
                    }
                    match &text_action.operation {
                        TextOperation::ChangeText | TextOperation::DeleteText => {
                            if let Some(obj) = &text_action.text_obj {
                                match obj {
                                    TextObj::InnerWord => {
                                        let range = self.text.current_word_bounds(self.index);
                                        self.text.edit(range.clone(), "");
                                        self.index = range.start;
                                    },
                                    TextObj::OuterWord => {
                                        let mut range = self.text.current_word_bounds(self.index);
                                        let mut range_modified = false;
                                        if let Some(next_graph_index) = self.text.next_grapheme_offset(range.end) {
                                            if let Some(next_graph) = self.text.slice(range.end..next_graph_index) {
                                                if next_graph.contains(char::is_whitespace) {
                                                    range.end = next_graph_index;
                                                    range_modified = true;
                                                }
                                            }
                                        }
                                        if !range_modified {
                                            if let Some(prev_graph_index) = self.text.prev_grapheme_offset(range.start) {
                                                if let Some(prev_graph) = self.text.slice(prev_graph_index..range.start) {
                                                    if prev_graph.contains(char::is_whitespace) {
                                                        range.start = prev_graph_index;
                                                    }
                                                }
                                            }
                                        }
                                        self.text.edit(range.clone(), "");
                                        self.index = range.start;
                                    },
                                    TextObj::Inner(delimiters) => {
                                        change_mode = None;
                                        if let Some(opening_index) = self.text.prev_occurrence(self.index, delimiters.slice(0..delimiters.next_grapheme_offset(0).unwrap()).unwrap().to_string()) {
                                            if let Some(opening_index_next) = self.text.next_grapheme_offset(opening_index) {
                                                if let Some(closing_index) = self.text.next_occurrence(self.index, delimiters.slice(delimiters.prev_grapheme_offset(delimiters.len()).unwrap()..delimiters.len()).unwrap().to_string()) {
                                                    self.text.edit(opening_index_next..closing_index, "");
                                                    self.set_cursor(Some(opening_index_next));
                                                    change_mode = Some(KeybindMode::Insert);
                                                }
                                            }
                                        }
                                    },
                                    TextObj::Outer(delimiters) => {
                                        change_mode = None;
                                        if let Some(opening_index) = self.text.prev_occurrence(self.index, delimiters.slice(0..delimiters.next_grapheme_offset(0).unwrap()).unwrap().to_string()) {
                                            if let Some(closing_index) = self.text.next_occurrence(self.index, delimiters.slice(delimiters.prev_grapheme_offset(delimiters.len()).unwrap()..delimiters.len()).unwrap().to_string()) {
                                                if let Some(closing_index) = self.text.next_grapheme_offset(closing_index) {
                                                    self.text.edit(opening_index..closing_index, "");
                                                    self.set_cursor(Some(opening_index));
                                                    change_mode = Some(KeybindMode::Insert);
                                                }
                                            }
                                        }
                                    },
                                    _ => ()
                                }
                            } else if let Some(motion) = &text_action.text_motion {
                                match motion {
                                    TextMotion::ForwardCharacter => {
                                        if let Some(next_graph) = self.text.next_grapheme_offset(self.index) {
                                            self.text.edit(self.index..next_graph, "");
                                        }
                                    },
                                    TextMotion::BackwardCharacter => {
                                        if let Some(prev_graph) = self.text.prev_grapheme_offset(self.index) {
                                            self.text.edit(prev_graph..self.index, "");
                                            self.set_cursor(Some(prev_graph));
                                        }
                                    },
                                    TextMotion::ForwardWordStart | TextMotion::ForwardWordEnd => {
                                        if let Some(next_end) = self.text.next_word_end_offset(self.index) {
                                            if let Some(after) = self.text.next_grapheme_offset(next_end) {
                                                self.text.edit(self.index..after, "");
                                                self.index = self.text.len();
                                            }
                                        }
                                    },
                                    TextMotion::BackwardWordStart => {
                                        if let Some(prev_start) = self.text.prev_word_start_offset(self.index) {
                                            self.text.edit(prev_start..self.index, "");
                                            self.set_cursor(Some(prev_start));
                                        }
                                    },
                                    TextMotion::BackwardWordEnd => {
                                        if let Some(prev_end) = self.text.prev_word_end_offset(self.index) {
                                            self.text.edit(prev_end..self.index, "");
                                            self.set_cursor(Some(prev_end));
                                        }
                                    },
                                    TextMotion::ForwardToN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            if let Some(occurrence) = self.text.next_occurrence(self.index, grapheme) {
                                                self.text.edit(self.index..occurrence, "");
                                            }
                                        }
                                    },
                                    TextMotion::BackwardToN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            if let Some(occurrence) = self.text.prev_occurrence(self.index, grapheme) {
                                                if let Some(next_graph) = self.text.next_grapheme_offset(occurrence) {
                                                    self.text.edit(next_graph..self.index, "");
                                                    self.set_cursor(Some(next_graph));
                                                }
                                            }
                                        }
                                    },
                                    TextMotion::ForwardWithN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            if let Some(occurrence) = self.text.next_occurrence(self.index, grapheme) {
                                                if let Some(next_graph) = self.text.next_grapheme_offset(occurrence) {
                                                    self.text.edit(self.index..next_graph, "");
                                                }
                                            }
                                        }
                                    },
                                    TextMotion::BackwardWithN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            if let Some(occurrence) = self.text.prev_occurrence(self.index, grapheme) {
                                                self.text.edit(occurrence..self.index, "");
                                                self.set_cursor(Some(occurrence));
                                            }
                                        }
                                    },
                                    TextMotion::BeginningLine => {
                                        self.text.edit(0..self.index, "");
                                        self.cursor_to_start();
                                    },
                                    TextMotion::EndLine => {
                                        self.text.edit(self.index..self.text.len(), "");
                                    }
                                }
                            }
                        },
                        TextOperation::None => {
                            if let Some(motion) = &text_action.text_motion {
                                match motion {
                                    TextMotion::ForwardCharacter => {
                                        self.cursor_forward();
                                    },
                                    TextMotion::BackwardCharacter => {
                                        self.cursor_backward();
                                    },
                                    TextMotion::ForwardWordStart => {
                                        self.set_cursor(self.text.next_word_start_offset(self.index));
                                    },
                                    TextMotion::BackwardWordStart => {
                                        self.set_cursor(self.text.prev_word_start_offset(self.index));
                                    },
                                    TextMotion::ForwardWordEnd => {
                                        self.set_cursor(self.text.next_word_end_offset(self.index));
                                    },
                                    TextMotion::BackwardWordEnd => {
                                        self.set_cursor(self.text.prev_word_end_offset(self.index));
                                    },
                                    TextMotion::ForwardToN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            if let Some(occurrence) = self.text.next_occurrence(self.index, grapheme) {
                                                self.set_cursor(self.text.prev_grapheme_offset(occurrence));
                                            }
                                        }
                                    },
                                    TextMotion::BackwardToN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            if let Some(occurrence) = self.text.prev_occurrence(self.index, grapheme) {
                                                self.set_cursor(self.text.next_grapheme_offset(occurrence));
                                            }
                                        }
                                    },
                                    TextMotion::ForwardWithN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            self.set_cursor(self.text.next_occurrence(self.index, grapheme));
                                        }
                                    },
                                    TextMotion::BackwardWithN => {
                                        if let Some(grapheme) = text_action.character_string.clone() {
                                            self.set_cursor(self.text.prev_occurrence(self.index, grapheme));
                                        }
                                    },
                                    TextMotion::BeginningLine => {
                                        self.set_cursor(Some(0));
                                    },
                                    TextMotion::EndLine => {
                                        self.set_cursor(Some(self.text.len()));
                                    }
                                }
                            }
                        }
                        TextOperation::ReplaceText => {
                            if let Some(text) = &text_action.character_string {
                                if let Some(next_graph) = self.text.next_grapheme_offset(self.index) {
                                    self.text.edit(self.index..next_graph, text);
                                }
                            }
                        },
                    }
                }
            },
            Action::InsertCharacter => {
                self.text.insert_str(self.index, &payload.string.clone().unwrap());
                self.index = self.text.next_grapheme_offset(self.index).unwrap();
            },
            Action::InsertCharacterUnconfirmed => {
                self.text.insert_str(self.index, &payload.string.clone().unwrap());
                let next = self.text.next_grapheme_offset(self.index).unwrap();
                if let Some(range) = &mut self.unconfirmed_range {
                    range.end = next;
                } else {
                    self.unconfirmed_range = Some(self.index..next);
                }
                self.index = next;
            },
            Action::ConfirmInserts => {
                self.unconfirmed_range = None;
            },
            Action::RollBackInserts => {
                if let Some(range) = &self.unconfirmed_range {
                    self.text.edit(range.clone(), "");
                    self.set_cursor(Some(range.start));
                }
                self.unconfirmed_range = None;
            }
            _ => ()
        }
        if (self.mode == KeybindMode::Edit ||
            self.mode == KeybindMode::Visual) && self.index == self.text.len() && change_mode != Some(KeybindMode::Insert) {
                self.cursor_backward();
            } 
        ctx.request_layout();
        ctx.request_paint();
        return change_mode;
    }

    pub fn cursor_forward(&mut self) -> Result<(), ()> {
        if let Some(new_index) = self.text.next_grapheme_offset(self.index) { 
            if self.mode == KeybindMode::Edit || self.mode == KeybindMode::Visual {
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
            self.index = new_index; 
            return Ok(());
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

    pub fn get_block_cursor_bounds(&self, index: usize) -> Rect {
        if let Some(layout) = &self.text_layout {
            if self.text.is_empty() {
                let metric = layout.line_metric(0).unwrap();
                    return Rect::new(
                        0.,
                        0.,
                        6.,
                        metric.height,
                    );
            } else if let Some(next_index) = self.text.next_grapheme_offset(index) {
                let rects = layout.rects_for_range(index..next_index);
                return rects[0];
            } else {
                let rects = layout.rects_for_range(self.text.prev_grapheme_offset(index).unwrap()..index);
                    return Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+6.,
                        rects[0].y1,
                    );
            }
        } else {
            return Rect::new(0.,0.,0.,0.);
        }
    }

    pub fn get_line_cursor_bounds(&self, index: usize) -> Rect {
        if let Some(layout) = &self.text_layout {
            if self.text.is_empty() {
                let metric = layout.line_metric(0).unwrap();
                return Rect::new(
                        0.,
                        0.,
                        DEFUALT_TEXT_CURSOR_WIDTH,
                        metric.height,
                    );
            } else if let Some(next_index) = self.text.next_grapheme_offset(index) {
                let rects = layout.rects_for_range(index..next_index);
                return Rect::new(
                        rects[0].x0,
                        rects[0].y0,
                        rects[0].x0+DEFUALT_TEXT_CURSOR_WIDTH,
                        rects[0].y1,
                    );
            } else {
                let rects = layout.rects_for_range(self.text.prev_grapheme_offset(index).unwrap()..index);
                return Rect::new(
                        rects[0].x1,
                        rects[0].y0,
                        rects[0].x1+DEFUALT_TEXT_CURSOR_WIDTH,
                        rects[0].y1,
                    );
            }
        } else {
            return Rect::new(0.,0.,0.,0.);
        }
    }

    pub fn cursor_to_end(&mut self) {
        self.index = self.text.len();
    }

    pub fn cursor_to_start(&mut self) {
        self.index = 0;
    }

    pub fn set_keybind_mode(&mut self, mode: KeybindMode) {
        match mode {
            KeybindMode::Insert => {
                self.visual_anchor = None;
            }
            KeybindMode::Edit => {
                self.visual_anchor = None;
                if let Some(_) = self.text.next_grapheme_offset(self.index) {
                } else {
                    self.cursor_backward();
                }
            },
            KeybindMode::Visual => {
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

    pub fn get_cursor_index(&self) -> usize {
        return self.index;
    }

    pub fn get_index_at_point(&self, point: Point) -> Result<usize, ()> {
        let htp = self.text_layout.as_ref().unwrap().hit_test_point(point);
        if htp.is_inside {
            return Ok(htp.idx);
        }
        return Err(());
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
            let rect = match self.mode {
                KeybindMode::Insert => { self.get_line_cursor_bounds(self.index) },
                KeybindMode::Edit => { self.get_block_cursor_bounds(self.index) },
                KeybindMode::Visual => { self.get_block_cursor_bounds(self.index) },
                _ => { Rect::new(0.,0.,0.,0.) }
            };
            ctx.fill(
                rect,
                &config.get_color(VMColor::TextCursorColor).unwrap()
            );

            if debug {
                let debug_layout = ctx.text().new_text_layout(format!("{}", self.index))
                .text_color(Color::RED)
                .font(FontFamily::SANS_SERIF, 6.)
                .build()
                .unwrap();
                ctx.with_save(|ctx| {
                    ctx.transform(Affine::translate(Vec2::new(rect.x0, rect.y1)));
                    ctx.fill(debug_layout.size().to_rect(), &Color::BLUE);
                });
                ctx.draw_text(&debug_layout, Point::new(rect.x0, rect.y1));
            }
            ctx.draw_text(layout, Point {x: 0., y: 0.});
        }
    }
}