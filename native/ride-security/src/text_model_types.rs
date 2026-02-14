/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use crate::position::Position;
use crate::range::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum EndOfLineSequence {
    LF,
    CRLF,
}

#[derive(Debug)]
pub enum EditOperationType {
    Other,
    Typing,
    DeletingLeft,
    DeletingRight,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct PositionPod {
    pub line_number: u32,
    pub column: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct RangePod {
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct SingleEditOperation {
    pub range: RangePod,
    pub text: Option<String>,
    pub force_move_markers: Option<bool>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct TextChange {
    pub start_position: PositionPod,
    pub old_text: String,
    pub new_text: String,
}

#[napi(object)]
pub struct ModelContentChangedEvent {
    pub changes: Vec<TextChange>,
    pub eol: String,
    pub version_id: u32,
    pub is_undoing: bool,
    pub is_redoing: bool,
    pub is_flush: bool,
}
