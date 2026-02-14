/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use crate::position::Position;
use crate::range::Range;
use crate::selection::Selection;
use crate::text_model::TextModel;
use crate::cursor::Cursor;

// Placeholder for now
#[napi]
struct TextEdit {}

#[napi]
impl TextEdit {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}
