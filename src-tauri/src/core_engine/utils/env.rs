use std::env;

pub fn get_cloud_function_url() -> String {
    env::var("PRETZL_CLOUD_BACKEND_URL").unwrap_or(
        "https://europe-west1-analyze-text-dev.cloudfunctions.net/analyze-code".to_string(),
    )
}

pub fn get_cloud_function_apikey() -> String {
    "".to_string()
}
