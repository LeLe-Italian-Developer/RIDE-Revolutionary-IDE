/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line_number: u32,
    pub column: u32,
}

impl Position {
    pub fn new(line_number: u32, column: u32) -> Self {
        Self {
            line_number,
            column,
        }
    }

    pub fn is_before(&self, other: &Position) -> bool {
        self.cmp_before(other)
    }

    pub fn is_before_or_equal(&self, other: &Position) -> bool {
        self.cmp_before_eq(other)
    }

    pub fn compare(a: &Position, b: &Position) -> i32 {
        Position::cmp(a, b)
    }

    pub fn with(&self, line_number: Option<u32>, column: Option<u32>) -> Self {
        Self {
            line_number: line_number.unwrap_or(self.line_number),
            column: column.unwrap_or(self.column),
        }
    }

    pub fn delta(&self, delta_line_number: Option<i32>, delta_column: Option<i32>) -> Self {
        let new_line = (self.line_number as i32 + delta_line_number.unwrap_or(0)).max(1) as u32;
        let new_col = (self.column as i32 + delta_column.unwrap_or(0)).max(1) as u32;
        Self {
            line_number: new_line,
            column: new_col,
        }
    }

    pub fn cmp_before(&self, other: &Position) -> bool {
         if self.line_number < other.line_number {
            true
        } else if self.line_number > other.line_number {
            false
        } else {
            self.column < other.column
        }
    }

    pub fn cmp_before_eq(&self, other: &Position) -> bool {
         if self.line_number < other.line_number {
            true
        } else if self.line_number > other.line_number {
            false
        } else {
            self.column <= other.column
        }
    }

    pub fn cmp(a: &Position, b: &Position) -> i32 {
        match a.line_number.cmp(&b.line_number) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => match a.column.cmp(&b.column) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Equal => 0,
            },
        }
    }
}
