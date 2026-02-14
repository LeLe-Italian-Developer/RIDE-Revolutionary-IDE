use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct RemoteAuthority {
    pub scheme: String,
    pub authority: String,
    pub path: String,
}

#[napi]
pub struct RemoteService {
    connection_token: String,
}

#[napi]
impl RemoteService {
    #[napi(constructor)]
    pub fn new(token: String) -> Self {
        Self {
            connection_token: token,
        }
    }

    #[napi]
    pub fn parse_authority(&self, authority: String) -> RemoteAuthority {
        let parts: Vec<&str> = authority.split('+').collect();
        if parts.len() == 2 {
            RemoteAuthority {
                scheme: parts[0].to_string(),
                authority: parts[1].to_string(),
                path: "/".to_string(),
            }
        } else {
            RemoteAuthority {
                scheme: "ssh".to_string(),
                authority,
                path: "/".to_string(),
            }
        }
    }
}
