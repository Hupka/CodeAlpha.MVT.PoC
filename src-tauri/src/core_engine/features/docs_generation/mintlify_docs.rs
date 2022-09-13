use std::env;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintlifyResponse {
    pub docstring: String,
    pub feedbackId: String,
    pub position: String,
    pub preview: String,
    pub shouldShowFeedback: bool,
    pub shouldShowShare: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintlifyRequest {
    apiKey: String,
    code: String,
    context: String,
}

pub fn _get_mintlify_documentation(
    code: &String,
    context: Option<String>,
) -> Option<MintlifyResponse> {
    let handle = mintlify_documentation(code, context);
    let mintlify_response: Result<MintlifyResponse, reqwest::Error> = block_on(handle);

    if let Ok(mintlify_response) = mintlify_response {
        Some(mintlify_response)
    } else {
        None
    }
}

pub async fn mintlify_documentation(
    code: &String,
    context: Option<String>,
) -> Result<MintlifyResponse, reqwest::Error> {
    let ctx_string = if let Some(context) = context {
        context
    } else {
        "".to_string()
    };

    let req_body = MintlifyRequest {
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code: code.clone(),
        context: ctx_string,
    };

    let url;
    let env_url = env::var("CODEALPHA_CLOUD_BACKEND_URL");
    if env_url.is_ok() {
        url = env_url.unwrap();
    } else {
        url = "https://europe-west1-analyze-text-dev.cloudfunctions.net/analyze-code".to_string();
    }

    let response = reqwest::Client::new()
        .post(url)
        .json(&req_body)
        .send()
        .await?;
    let parsed_response = response.json().await?;
    Ok(parsed_response)
}

#[cfg(test)]
mod tests_mintlify {

    use super::_get_mintlify_documentation;

    #[test]
    #[ignore]
    fn test_get_mintlify_documentation() {
        let resp = _get_mintlify_documentation(
            &"print(\"Hello World\")".to_string(),
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }

    #[test]
    #[ignore]
    fn test_get_mintlify_documentation_without_context() {
        let resp = _get_mintlify_documentation(&"print(\"Hello World\")".to_string(), None);
        assert!(resp.is_some());
    }
}