/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi_derive::napi;
use std::collections::HashMap;
use regex::Regex;

#[napi(object)]
#[derive(Clone, Debug)]
pub struct Token {
    pub start_index: u32,
    pub length: u32,
    pub token_type: String,
    pub scopes: Vec<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct TokenizerRule {
    pub regex: String,
    pub token_type: String,
}

#[napi]
pub struct Tokenizer {
    rules: Vec<(Regex, String)>,
}

#[napi]
impl Tokenizer {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    #[napi]
    pub fn add_rule(&mut self, regex_str: String, token_type: String) -> napi::Result<()> {
        let re = Regex::new(&regex_str).map_err(|e| napi::Error::from_reason(e.to_string()))?;
        self.rules.push((re, token_type));
        Ok(())
    }

    #[napi]
    pub fn tokenize(&self, line_content: String) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let content_len = line_content.len();

        while pos < content_len {
            let mut matched = false;
            let slice = &line_content[pos..];

            for (re, token_type) in &self.rules {
                if let Some(m) = re.find(slice) {
                    if m.start() == 0 {
                        let len = m.end();
                        tokens.push(Token {
                            start_index: pos as u32,
                            length: len as u32,
                            token_type: token_type.clone(),
                            scopes: vec![format!("source.{}", token_type)],
                        });
                        pos += len;
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                let next_char_len = slice.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                tokens.push(Token {
                    start_index: pos as u32,
                    length: next_char_len as u32,
                    token_type: "text".to_string(),
                    scopes: vec!["source.text".to_string()],
                });
                pos += next_char_len;
            }
        }
        tokens
    }

    #[napi]
    pub fn tokenize_lines(&self, lines: Vec<String>) -> Vec<Vec<Token>> {
        lines.into_iter().map(|l| self.tokenize(l)).collect()
    }
}
