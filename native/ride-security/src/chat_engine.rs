/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Agentic Chat Engine v2
//!
//! Features:
//! - Multi-session state management with persistent history
//! - Rich message types (Markdown, Tool Calls, Variable References)
//! - Variable resolution engine for IDE context (files, selections, symbols)
//! - Participant-based routing (Copilot, specialized agents)
//! - Streaming protocol baseline
//! - Token usage and latency telemetry

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user", "assistant", "system", "tool"
    pub content: String,
    pub timestamp: f64,
    pub tool_calls: Option<Vec<ChatToolCall>>,
    pub variables: Option<Vec<ChatVariableReference>>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatVariableReference {
    pub name: String,
    pub value: String,
    pub range_json: Option<String>, // Range of text where variable is used
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub participant_id: String,
    pub messages: Vec<ChatMessage>,
    pub metadata: HashMap<String, String>,
    pub stats: ChatSessionStats,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSessionStats {
    pub token_count: u32,
    pub turn_count: u32,
    pub last_turn_latency_ms: f64,
}

#[napi]
pub struct ChatEngine {
    sessions: Mutex<HashMap<String, ChatSession>>,
    participants: Mutex<HashMap<String, String>>, // ID -> Name
    context_variables: Mutex<HashMap<String, String>>, // Name -> Value (global context)
}

#[napi]
impl ChatEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            participants: Mutex::new(HashMap::new()),
            context_variables: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn register_participant(&self, id: String, name: String) {
        self.participants.lock().unwrap().insert(id, name);
    }

    #[napi]
    pub fn create_session(&self, session_id: String, participant_id: String) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), ChatSession {
            id: session_id,
            participant_id,
            messages: Vec::new(),
            metadata: HashMap::new(),
            stats: ChatSessionStats {
                token_count: 0,
                turn_count: 0,
                last_turn_latency_ms: 0.0,
            },
        });
    }

    #[napi]
    pub fn add_message(&self, session_id: String, message: ChatMessage) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.stats.turn_count += 1;
            // Simplified token estimating
            session.stats.token_count += (message.content.len() / 4) as u32;
            session.messages.push(message);
            return true;
        }
        false
    }

    #[napi]
    pub fn set_context_variable(&self, name: String, value: String) {
        self.context_variables.lock().unwrap().insert(name, value);
    }

    #[napi]
    pub fn resolve_variables(&self, text: String) -> Vec<ChatVariableReference> {
        let context = self.context_variables.lock().unwrap();
        let mut refs = Vec::new();
        for (name, value) in context.iter() {
            let var_token = format!("#{}", name);
            if text.contains(&var_token) {
                refs.push(ChatVariableReference {
                    name: name.clone(),
                    value: value.clone(),
                    range_json: None,
                });
            }
        }
        refs
    }

    #[napi]
    pub fn get_session(&self, session_id: String) -> Option<ChatSession> {
        self.sessions.lock().unwrap().get(&session_id).cloned()
    }

    #[napi]
    pub fn delete_session(&self, session_id: String) -> bool {
        self.sessions.lock().unwrap().remove(&session_id).is_some()
    }

    #[napi]
    pub fn update_telemetry(&self, session_id: String, latency: f64) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.stats.last_turn_latency_ms = latency;
        }
    }
}
