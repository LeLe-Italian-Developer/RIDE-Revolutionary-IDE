/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use crate::position::Position;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
}

impl Range {
    pub fn new(start_line_number: u32, start_column: u32, end_line_number: u32, end_column: u32) -> Self {
        if start_line_number > end_line_number || (start_line_number == end_line_number && start_column > end_column) {
            Self {
                start_line_number: end_line_number,
                start_column: end_column,
                end_line_number: start_line_number,
                end_column: start_column,
            }
        } else {
            Self {
                start_line_number,
                start_column,
                end_line_number,
                end_column,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start_line_number == self.end_line_number && self.start_column == self.end_column
    }

    pub fn contains_position(&self, position: &Position) -> bool {
        if position.line_number < self.start_line_number || position.line_number > self.end_line_number {
            false
        } else if position.line_number == self.start_line_number && position.column < self.start_column {
            false
        } else if position.line_number == self.end_line_number && position.column > self.end_column {
            false
        } else {
            true
        }
    }

    pub fn contains_range(&self, range: &Range) -> bool {
        if range.start_line_number < self.start_line_number || range.end_line_number > self.end_line_number {
            false
        } else if range.start_line_number == self.start_line_number && range.start_column < self.start_column {
            false
        } else if range.end_line_number == self.end_line_number && range.end_column > self.end_column {
            false
        } else {
            true
        }
    }

    pub fn plus_range(&self, range: &Range) -> Range {
        let start_line_number = std::cmp::min(self.start_line_number, range.start_line_number);
        let start_column = if self.start_line_number == range.start_line_number {
            std::cmp::min(self.start_column, range.start_column)
        } else if self.start_line_number < range.start_line_number {
            self.start_column
        } else {
            range.start_column
        };

        let end_line_number = std::cmp::max(self.end_line_number, range.end_line_number);
        let end_column = if self.end_line_number == range.end_line_number {
            std::cmp::max(self.end_column, range.end_column)
        } else if self.end_line_number > range.end_line_number {
            self.end_column
        } else {
            range.end_column
        };

        Range {
            start_line_number,
            start_column,
            end_line_number,
            end_column,
        }
    }

    pub fn get_start_position(&self) -> Position {
        Position {
            line_number: self.start_line_number,
            column: self.start_column,
        }
    }

    pub fn get_end_position(&self) -> Position {
        Position {
            line_number: self.end_line_number,
            column: self.end_column,
        }
    }
}
