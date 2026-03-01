use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub success: bool,
    pub data: Option<String>,
    pub message: Option<String>,
}

impl Response {
    pub fn ok(data: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn err(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

pub fn response_ok(data: &str) -> String {
    Response::ok(data.to_string()).to_json()
}

pub fn response_err(message: &str) -> String {
    Response::err(message.to_string()).to_json()
}

pub struct JsonResponse {
    pub body: String,
}

impl JsonResponse {
    pub fn new(body: &str) -> Self {
        Self {
            body: body.to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        self.body.clone()
    }
}
