/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Snippet Parser — Rust port of `src/vs/editor/contrib/snippet/browser/snippetParser.ts`.
//! Parses TextMate-style snippets with placeholders, variables, and transformations.

use napi_derive::napi;
use napi::bindgen_prelude::*;

#[napi(string_enum)]
#[derive(PartialEq, Debug)]
pub enum SnippetNodeType {
    Text,
    Placeholder,
    Variable,
}

#[napi(object)]
#[derive(Clone)]
pub struct SnippetTransform {
    pub regex: String,
    pub format: String,
    pub options: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct SnippetNode {
    pub type_: SnippetNodeType,
    pub text: Option<String>,
    pub name: Option<String>,
    pub index: Option<u32>,
    pub children: Option<Vec<SnippetNode>>,
    pub transform: Option<SnippetTransform>,
}

#[napi(object)]
#[derive(Clone)]
pub struct Snippet {
    pub children: Vec<SnippetNode>,
}

pub struct SnippetParser {
    text: Vec<char>,
    pos: usize,
}

impl SnippetParser {
    pub fn new(text: &str) -> Self {
        Self { text: text.chars().collect(), pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Snippet> {
        let mut children = Vec::new();
        while self.pos < self.text.len() {
            if let Some(node) = self.parse_node()? {
                children.push(node);
            } else {
                // If parse_node returns None but we aren't at end, it means we hit a stop char like } or : or /
                // In top level, we shouldn't hit these. But if we do, treat as text?
                // Actually parse_node handles text.
                // If it returns None, it means EOF or special handling required by caller.
                // But at top level loop, we expect to consume everything.
                break;
            }
        }
        Ok(Snippet { children })
    }

    fn peek(&self) -> char {
        if self.pos >= self.text.len() { '\0' } else { self.text[self.pos] }
    }

    fn next(&mut self) -> char {
        let c = self.peek();
        if self.pos < self.text.len() { self.pos += 1; }
        c
    }

    fn parse_node(&mut self) -> Result<Option<SnippetNode>> {
        if self.pos >= self.text.len() { return Ok(None); }

        let char = self.peek();

        // Escape
        if char == '\\' {
            self.next();
            let escaped = self.next();
            return Ok(Some(SnippetNode {
                type_: SnippetNodeType::Text,
                text: Some(escaped.to_string()),
                name: None, index: None, children: None, transform: None
            }));
        }

        // Variable or Placeholder
        if char == '$' {
            self.next();
            return self.parse_var_or_placeholder().map(Some);
        }

        // Text
        let mut text = String::new();
        while self.pos < self.text.len() {
            let c = self.peek();
            if c == '$' || c == '\\' || c == '}' || c == ':' {
                // Determine if we should stop.
                // In standard parser, } : are stop chars only inside placeholders.
                // But here we might over-consume.
                // Let's stop at $ and \ always.
                // } and : are context dependent.
                // Simplified: Stop at $ and \.
                break;
            }
            text.push(self.next());
        }

        if text.is_empty() { return Ok(None); } // Hit a special char immediately

        Ok(Some(SnippetNode {
            type_: SnippetNodeType::Text,
            text: Some(text),
            name: None, index: None, children: None, transform: None
        }))
    }

    fn parse_var_or_placeholder(&mut self) -> Result<SnippetNode> {
        let is_bracket = if self.peek() == '{' { self.next(); true } else { false };

        let mut name: Option<String> = None;
        let mut index: Option<u32> = None;

        if self.peek().is_digit(10) {
            let mut num = String::new();
            while self.peek().is_digit(10) { num.push(self.next()); }
            index = num.parse().ok();
        } else {
            let mut ident = String::new();
            while self.peek().is_alphanumeric() || self.peek() == '_' { ident.push(self.next()); }
            name = Some(ident);
        }

        let mut children = None;
        let mut transform = None;

        if is_bracket {
            if self.peek() == ':' {
                 self.next();
                 // Parse default value
                 let mut defaults = Vec::new();
                 // Basic recursion for default value
                 // NOTE: This recursive parser is simplified
                 while self.peek() != '}' && self.pos < self.text.len() {
                     match self.parse_node()? {
                         Some(n) => defaults.push(n),
                         None => {
                            // Hit a char that parse_node doesn't prefer, like } or :
                            // Since we are inside default, assume } ends it.
                            if self.peek() == '}' { break; }
                            // Otherwise consume as text
                            let c = self.next();
                            defaults.push(SnippetNode {
                                type_: SnippetNodeType::Text,
                                text: Some(c.to_string()),
                                name: None, index: None, children: None, transform: None
                            });
                         }
                     }
                 }
                 children = Some(defaults);
            } else if self.peek() == '/' {
                self.next();
                // Regex transform
                let mut regex = String::new();
                while self.peek() != '/' && self.pos < self.text.len() { regex.push(self.next()); }
                self.next(); // /
                let mut format = String::new();
                while self.peek() != '/' && self.pos < self.text.len() { format.push(self.next()); }
                self.next(); // /
                let mut options = String::new();
                while self.peek() != '}' && self.pos < self.text.len() { options.push(self.next()); }

                transform = Some(SnippetTransform { regex, format, options: Some(options) });
            }

            if self.peek() == '}' { self.next(); }
        }

        if let Some(i) = index {
            Ok(SnippetNode {
                type_: SnippetNodeType::Placeholder,
                index: Some(i),
                name: None, text: None, children, transform
            })
        } else {
            Ok(SnippetNode {
                type_: SnippetNodeType::Variable,
                name: name,
                index: None, text: None, children, transform
            })
        }
    }
}

#[napi]
pub fn parse_snippet(template: String) -> Result<Snippet> {
    let mut parser = SnippetParser::new(&template);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let s = parse_snippet("hello".into()).unwrap();
        assert_eq!(s.children.len(), 1);
        assert_eq!(s.children[0].text.as_deref(), Some("hello"));
    }

    #[test]
    fn test_parse_placeholder() {
        let s = parse_snippet("${1:default}".into()).unwrap();
        assert_eq!(s.children.len(), 1);
        let n = &s.children[0];
        assert_eq!(n.type_, SnippetNodeType::Placeholder);
        assert_eq!(n.index, Some(1));
        assert!(n.children.is_some());
    }

    #[test]
    fn test_parse_variable() {
        let s = parse_snippet("$V".into()).unwrap();
        assert_eq!(s.children.len(), 1);
        assert_eq!(s.children[0].name.as_deref(), Some("V"));
    }
}
