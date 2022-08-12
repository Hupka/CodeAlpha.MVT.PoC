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

pub fn get_mintlify_documentation(
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

    println!("{:?}", req_body);

    let response = reqwest::Client::new()
        .post("https://europe-west1-codealpha-analyze-text-dev.cloudfunctions.net/analyze-code")
        .json(&req_body)
        .send()
        .await?;
    println!("{:?}", response);
    let parsed_response = response.json().await?;
    println!("{:?}", parsed_response);
    Ok(parsed_response)
}

#[cfg(test)]
mod tests_mintlify {

    use super::get_mintlify_documentation;

    #[test]
    fn test_get_mintlify_documentation() {
        let resp = get_mintlify_documentation(
            &"print(\"Hello World\")".to_string(),
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }

    #[test]
    fn test_get_mintlify_documentation_without_context() {
        let resp = get_mintlify_documentation(&"print(\"Hello World\")".to_string(), None);
        assert!(resp.is_some());
    }
}