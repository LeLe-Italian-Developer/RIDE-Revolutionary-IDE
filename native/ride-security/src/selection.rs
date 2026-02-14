/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use crate::position::Position;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Selection {
    pub selection_start_line_number: u32,
    pub selection_start_column: u32,
    pub position_line_number: u32,
    pub position_column: u32,
}

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum SelectionDirection {
    LTR = 0,
    RTL = 1,
}

impl Selection {
    pub fn new(selection_start_line_number: u32, selection_start_column: u32, position_line_number: u32, position_column: u32) -> Self {
        Self {
            selection_start_line_number,
            selection_start_column,
            position_line_number,
            position_column,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.selection_start_line_number == self.position_line_number &&
        self.selection_start_column == self.position_column
    }

    pub fn get_start_position(&self) -> Position {
        if self.selection_start_line_number < self.position_line_number ||
           (self.selection_start_line_number == self.position_line_number && self.selection_start_column <= self.position_column) {
            Position::new(self.selection_start_line_number, self.selection_start_column)
        } else {
            Position::new(self.position_line_number, self.position_column)
        }
    }

    pub fn get_end_position(&self) -> Position {
        if self.selection_start_line_number < self.position_line_number ||
           (self.selection_start_line_number == self.position_line_number && self.selection_start_column <= self.position_column) {
            Position::new(self.position_line_number, self.position_column)
        } else {
            Position::new(self.selection_start_line_number, self.selection_start_column)
        }
    }

    pub fn get_direction(&self) -> SelectionDirection {
        if self.selection_start_line_number == self.position_line_number && self.selection_start_column == self.position_column {
            SelectionDirection::LTR
        } else if self.selection_start_line_number < self.position_line_number ||
           (self.selection_start_line_number == self.position_line_number && self.selection_start_column <= self.position_column) {
            SelectionDirection::LTR
        } else {
            SelectionDirection::RTL
        }
    }

    pub fn from_positions(start: Position, end: Position) -> Selection {
        Selection::new(start.line_number, start.column, end.line_number, end.column)
    }

    pub fn contains_position(&self, position: &Position) -> bool {
        let start = self.get_start_position();
        let end = self.get_end_position();

        if (position.line_number < start.line_number) || (position.line_number > end.line_number) {
            return false;
        }
        if (position.line_number == start.line_number) && (position.column < start.column) {
            return false;
        }
        if (position.line_number == end.line_number) && (position.column > end.column) {
            return false;
        }
        true
    }
}
