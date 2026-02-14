/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::logger::log_v2;

#[napi]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warning = 4,
    Error = 5,
    Critical = 6,
    Off = 7,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "FATAL",
            LogLevel::Off => "OFF",
        }
    }
}

#[napi]
pub struct WorkbenchLogger {
    id: String,
    level: LogLevel,
}

#[napi]
impl WorkbenchLogger {
    #[napi(constructor)]
    pub fn new(id: String, level: Option<LogLevel>) -> Self {
        Self {
            id,
            level: level.unwrap_or(LogLevel::Info),
        }
    }

    #[napi]
    pub fn trace(&self, message: String) {
        if self.level <= LogLevel::Trace {
            log_v2("TRACE".into(), message, self.id.clone());
        }
    }

    #[napi]
    pub fn debug(&self, message: String) {
        if self.level <= LogLevel::Debug {
            log_v2("DEBUG".into(), message, self.id.clone());
        }
    }

    #[napi]
    pub fn info(&self, message: String) {
        if self.level <= LogLevel::Info {
            log_v2("INFO".into(), message, self.id.clone());
        }
    }

    #[napi]
    pub fn warn(&self, message: String) {
        if self.level <= LogLevel::Warning {
            log_v2("WARN".into(), message, self.id.clone());
        }
    }

    #[napi]
    pub fn error(&self, message: String) {
        if self.level <= LogLevel::Error {
            log_v2("ERROR".into(), message, self.id.clone());
        }
    }
}
