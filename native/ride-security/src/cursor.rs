/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use crate::position::Position;
use crate::selection::Selection;
use crate::text_model::TextModel;
use crate::word_ops;

#[napi]
#[derive(Clone)]
pub struct Cursor {
    pub id: String,
    pub(crate) position: Position,
    pub(crate) selection: Selection,
    pub(crate) preferred_column: u32,
}

#[napi]
impl Cursor {
    #[napi(constructor)]
    pub fn new(position: Position) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            position: position,
            selection: Selection::from_positions(position, position),
            preferred_column: position.column,
        }
    }

    #[napi]
    pub fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }

    #[napi(getter)]
    pub fn position(&self) -> Position {
        self.position
    }

    #[napi(getter)]
    pub fn selection(&self) -> Selection {
        self.selection
    }

    #[napi]
    pub fn set_position(&mut self, pos: Position, keep_selection: bool) {
        self.position = pos;
        self.preferred_column = pos.column;
        if !keep_selection {
            self.selection = Selection::from_positions(pos, pos);
        } else {
            self.selection = Selection::from_positions(self.selection.get_start_position(), pos);
        }
    }

    // ─── Simple Movements ──────────────────────────────────────────────────

    #[napi]
    pub fn move_left(&mut self, model: &TextModel, count: u32, keep_selection: bool) {
        for _ in 0..count {
            if self.position.column > 1 {
                self.position.column -= 1;
            } else if self.position.line_number > 1 {
                self.position.line_number -= 1;
                self.position.column = model.get_line_content(self.position.line_number - 1).chars().count() as u32 + 1;
            }
        }
        self.preferred_column = self.position.column;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_right(&mut self, model: &TextModel, count: u32, keep_selection: bool) {
        for _ in 0..count {
            let line_content = model.get_line_content(self.position.line_number - 1);
            let line_len = line_content.chars().count() as u32;
            if self.position.column <= line_len {
                self.position.column += 1;
            } else if self.position.line_number < model.line_count() {
                self.position.line_number += 1;
                self.position.column = 1;
            }
        }
        self.preferred_column = self.position.column;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_up(&mut self, model: &TextModel, count: u32, keep_selection: bool) {
        for _ in 0..count {
            if self.position.line_number > 1 {
                self.position.line_number -= 1;
                let line_content = model.get_line_content(self.position.line_number - 1);
                let line_len = line_content.chars().count() as u32;
                self.position.column = std::cmp::min(self.preferred_column, line_len + 1);
            }
        }
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_down(&mut self, model: &TextModel, count: u32, keep_selection: bool) {
        for _ in 0..count {
            if self.position.line_number < model.line_count() {
                self.position.line_number += 1;
                let line_content = model.get_line_content(self.position.line_number - 1);
                let line_len = line_content.chars().count() as u32;
                self.position.column = std::cmp::min(self.preferred_column, line_len + 1);
            }
        }
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    // ─── Word Movements ────────────────────────────────────────────────────

    #[napi]
    pub fn move_word_left(&mut self, model: &TextModel, keep_selection: bool) {
        let content = model.get_line_content(self.position.line_number - 1);
        let new_col = word_ops::find_previous_word_start(&content, self.position.column);
        if new_col < self.position.column {
            self.position.column = new_col;
        } else if self.position.line_number > 1 {
            self.position.line_number -= 1;
            let prev_content = model.get_line_content(self.position.line_number - 1);
            self.position.column = prev_content.chars().count() as u32 + 1;
        }
        self.preferred_column = self.position.column;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_word_right(&mut self, model: &TextModel, keep_selection: bool) {
        let content = model.get_line_content(self.position.line_number - 1);
        let new_col = word_ops::find_next_word_end(&content, self.position.column);
        if new_col > self.position.column {
            self.position.column = new_col;
        } else if self.position.line_number < model.line_count() {
            self.position.line_number += 1;
            self.position.column = 1;
        }
        self.preferred_column = self.position.column;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    // ─── Boundary Movements ────────────────────────────────────────────────

    #[napi]
    pub fn move_to_line_start(&mut self, keep_selection: bool) {
        self.position.column = 1;
        self.preferred_column = 1;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_to_line_end(&mut self, model: &TextModel, keep_selection: bool) {
        let content = model.get_line_content(self.position.line_number - 1);
        self.position.column = content.chars().count() as u32 + 1;
        self.preferred_column = self.position.column;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_to_buffer_start(&mut self, keep_selection: bool) {
        self.position = Position::new(1, 1);
        self.preferred_column = 1;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }

    #[napi]
    pub fn move_to_buffer_end(&mut self, model: &TextModel, keep_selection: bool) {
        let line_count = model.line_count();
        let content = model.get_line_content(line_count - 1);
        self.position = Position::new(line_count, content.chars().count() as u32 + 1);
        self.preferred_column = self.position.column;
        if !keep_selection {
            self.selection = Selection::from_positions(self.position, self.position);
        }
    }
}
