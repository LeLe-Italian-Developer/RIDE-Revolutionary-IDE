/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use crate::text_model_types::SingleEditOperation;

#[napi]
#[derive(Clone, Default)]
pub struct EditStack {
    undo_stack: Vec<Vec<SingleEditOperation>>,
    redo_stack: Vec<Vec<SingleEditOperation>>,
}

#[napi]
impl EditStack {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    #[napi]
    pub fn push(&mut self, operations: Vec<SingleEditOperation>) {
        self.undo_stack.push(operations);
        self.redo_stack.clear();
    }

    pub fn pop_undo(&mut self) -> Option<Vec<SingleEditOperation>> {
        self.undo_stack.pop()
    }

    pub fn pop_redo(&mut self) -> Option<Vec<SingleEditOperation>> {
        self.redo_stack.pop()
    }

    pub fn push_redo(&mut self, operations: Vec<SingleEditOperation>) {
        self.redo_stack.push(operations);
    }
}
