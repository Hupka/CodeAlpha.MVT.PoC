use serde::{Deserialize, Serialize};
use textwrap::{wrap, Options};
use tracing::error;
use ts_rs::TS;

use cached::proc_macro::cached;

use crate::core_engine::{
    get_cloud_function_apikey, get_cloud_function_url,
    syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
    XcodeText,
};

use super::node_annotation::{AnnotationCodeBlock, NODE_EXPLANATION_CURRENT_DOCSTRING};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/node_explanation/")]
pub struct FunctionParameterWithExplanation {
    pub name: String,
    pub explanation: String,
    pub param_type: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParameterDto {
    pub name: String,
    pub explanation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/node_explanation/")]
pub struct NodeExplanation {
    pub summary: String,
    pub kind: SwiftCodeBlockKind,
    pub parameters: Option<Vec<FunctionParameterWithExplanation>>,
    pub name_suggestion: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NodeExplanationResponse {
    pub summary: String,
    pub kind: SwiftCodeBlockKind,
    pub parameters: Option<Vec<FunctionParameterDto>>,
    pub name_suggestion: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExplainResponse {
    data: NodeExplanationResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeExplanationRequest {
    apiKey: String,
    version: String,
    kind: SwiftCodeBlockKind,
    code: String,
    context: Option<String>,
    method: String,
    parameter_names: Option<Vec<String>>,
    code_block_name: Option<String>,
}

pub async fn fetch_node_explanation(
    code_block: AnnotationCodeBlock,
) -> Result<NodeExplanation, reqwest::Error> {
    let context = if let Some(context) = code_block.context {
        Some(String::from_utf16_lossy(&context))
    } else {
        None
    };

    let result = cached_fetch_node_explanation(
        code_block.text,
        code_block.kind,
        code_block.func_parameters_todo,
        code_block.name,
        context,
    )
    .await;
    if let Ok(node_explanation) = result.as_ref() {
        let node_docstring = explanation_to_docstring(&node_explanation);
        *NODE_EXPLANATION_CURRENT_DOCSTRING.lock() = node_docstring;
    }
    result
}

#[cached(result = true, size = 100)]
async fn cached_fetch_node_explanation(
    text: XcodeText,
    kind: SwiftCodeBlockKind,
    func_parameters: Option<Vec<FunctionParameter>>,
    name: Option<String>,
    context: Option<String>,
) -> Result<NodeExplanation, reqwest::Error> {
    let url = get_cloud_function_url();

    let codeblock_text_string = String::from_utf16_lossy(&text);

    let req_body = NodeExplanationRequest {
        code_block_name: name,
        version: "v1".to_string(),
        method: "explain".to_string(),
        apiKey: get_cloud_function_apikey(),
        code: codeblock_text_string,
        kind: kind.clone(),
        context,
        parameter_names: func_parameters
            .as_ref()
            .map(map_function_parameters_to_names),
    };

    let response = reqwest::Client::new()
        .post(url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| {
            error!(?e, "Error while sending explain request to cloud backend");
            e
        })?
        .json::<ExplainResponse>()
        .await
        .map_err(|e| {
            error!(
                ?e,
                "Error while parsing explain response from cloud backend"
            );
            e
        })?;

    let node_explanation =
        map_node_explanation_response_to_node_explanation(response.data, func_parameters.as_ref());

    Ok(node_explanation)
}

fn map_function_parameters_to_names(params: &Vec<FunctionParameter>) -> Vec<String> {
    params.iter().map(|p| p.name.clone()).collect()
}

fn map_node_explanation_response_to_node_explanation(
    response: NodeExplanationResponse,
    function_parameters: Option<&Vec<FunctionParameter>>,
) -> NodeExplanation {
    let parameters = if let (Some(function_parameters), Some(response_parameters)) =
        (function_parameters, response.parameters)
    {
        let mut parameters_with_explanations: Vec<FunctionParameterWithExplanation> = [].to_vec();
        for param in function_parameters {
            let response_param = response_parameters.iter().find(|p| p.name == param.name);
            if let Some(response_param) = response_param {
                parameters_with_explanations.push(FunctionParameterWithExplanation {
                    name: param.name.clone(),
                    explanation: response_param.explanation.clone(),
                    param_type: param.param_type.clone(),
                });
            }
        }
        Some(parameters_with_explanations)
    } else {
        None
    };
    NodeExplanation {
        summary: response.summary,
        kind: response.kind,
        parameters: parameters,
        name_suggestion: response.name_suggestion,
    }
}

fn explanation_to_docstring(explanation: &NodeExplanation) -> String {
    let line_length = 80;
    let mut docstring = String::new();
    docstring.push_str(&wrap_str(
        &format!("/// {}", explanation.summary),
        line_length,
        "/// ",
    ));
    if let Some(parameters) = &explanation.parameters {
        if parameters.len() > 0 {
            docstring.push_str("\n");
            for param in parameters {
                docstring.push_str(&wrap_str(
                    &format!(
                        "/// - parameter {}: `{}` {}\n",
                        param.name, param.param_type, param.explanation
                    ),
                    line_length,
                    "/// ",
                ));
            }
            docstring.pop();
        }
    }
    docstring
}

fn wrap_str(text: &str, n: usize, insert_str: &str) -> String {
    let options = Options::new(n)
        .initial_indent("")
        .subsequent_indent(insert_str);
    let lines = wrap(text, &options);
    lines.join("\n")
}

#[cfg(test)]
mod tests {

    mod map_node_explanation_response_to_node_explanation {
        use crate::core_engine::{
            features::{
                docs_generation::{
                    node_explanation::map_node_explanation_response_to_node_explanation,
                    FunctionParameterDto, FunctionParameterWithExplanation,
                    NodeExplanationResponse,
                },
                NodeExplanation,
            },
            syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
        };

        #[test]
        fn no_parameters() {
            let response = NodeExplanationResponse {
                name_suggestion: None,

                summary: "summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: None,
            };
            assert_eq!(
                map_node_explanation_response_to_node_explanation(response, None),
                NodeExplanation {
                    summary: "summary".to_string(),
                    name_suggestion: None,
                    kind: SwiftCodeBlockKind::Function,
                    parameters: None,
                }
            );
        }

        #[test]
        fn correct_parameters() {
            let response = NodeExplanationResponse {
                name_suggestion: None,

                summary: "summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![
                    FunctionParameterDto {
                        name: "param1".to_string(),
                        explanation: "It's a param".to_string(),
                    },
                    FunctionParameterDto {
                        name: "param2".to_string(),
                        explanation: "Another one".to_string(),
                    },
                ]),
            };
            let input_parameters = Some(vec![
                FunctionParameter {
                    name: "param1".to_string(),
                    param_type: "Int".to_string(),
                },
                FunctionParameter {
                    name: "param2".to_string(),
                    param_type: "String".to_string(),
                },
            ]);

            assert_eq!(
                map_node_explanation_response_to_node_explanation(
                    response,
                    input_parameters.as_ref()
                ),
                NodeExplanation {
                    name_suggestion: None,
                    summary: "summary".to_string(),
                    kind: SwiftCodeBlockKind::Function,
                    parameters: Some(vec![
                        FunctionParameterWithExplanation {
                            name: "param1".to_string(),
                            explanation: "It's a param".to_string(),
                            param_type: "Int".to_string(),
                        },
                        FunctionParameterWithExplanation {
                            name: "param2".to_string(),
                            explanation: "Another one".to_string(),
                            param_type: "String".to_string(),
                        },
                    ]),
                }
            );
        }

        #[test]
        fn filters_out_wrong_parameters() {
            let response = NodeExplanationResponse {
                name_suggestion: None,

                summary: "summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![
                    FunctionParameterDto {
                        name: "crazywrongparam".to_string(),
                        explanation: "{a{ADSSfci3 xc,v.je}}".to_string(),
                    },
                    FunctionParameterDto {
                        name: "param1".to_string(),
                        explanation: "It's a param".to_string(),
                    },
                    FunctionParameterDto {
                        name: "param1".to_string(),
                        explanation: "It's a param again???".to_string(),
                    },
                    FunctionParameterDto {
                        name: "crazywrongparasdfasdam".to_string(),
                        explanation: "{a{ADSSf133qrwfasdfci3 xc,v.je}}".to_string(),
                    },
                ]),
            };
            let input_parameters = Some(vec![
                FunctionParameter {
                    name: "param1".to_string(),
                    param_type: "Int".to_string(),
                },
                FunctionParameter {
                    name: "param2".to_string(),
                    param_type: "String".to_string(),
                },
            ]);

            assert_eq!(
                map_node_explanation_response_to_node_explanation(
                    response,
                    input_parameters.as_ref()
                ),
                NodeExplanation {
                    name_suggestion: None,

                    summary: "summary".to_string(),
                    kind: SwiftCodeBlockKind::Function,
                    parameters: Some(vec![FunctionParameterWithExplanation {
                        name: "param1".to_string(),
                        explanation: "It's a param".to_string(),
                        param_type: "Int".to_string(),
                    }]),
                }
            );
        }
    }

    mod wrap_str {
        use crate::core_engine::features::docs_generation::node_explanation::wrap_str;

        #[test]
        fn no_line_breaks() {
            assert_eq!(
                wrap_str(
                    "textwrap: an efficient and powerful library for wrapping text.",
                    80,
                    "///"
                ),
                "textwrap: an efficient and powerful library for wrapping text."
            );
        }

        #[test]
        fn multiple_line_break() {
            assert_eq!(
                wrap_str(
                    "textwrap: an efficient and powerful library for wrapping text.",
                    28,
                    "///"
                ),
                "textwrap: an efficient and\n///powerful library for\n///wrapping text."
            );
        }
    }

    mod explanation_to_docstring {
        use crate::core_engine::features::docs_generation::FunctionParameterWithExplanation;

        use super::super::explanation_to_docstring;
        use super::super::NodeExplanation;

        use super::super::SwiftCodeBlockKind;

        #[test]
        fn only_summary() {
            let explanation = NodeExplanation {
                summary: "This is a summary".to_string(),
                kind: SwiftCodeBlockKind::Class,
                parameters: None,
                name_suggestion: None,
            };
            let docstring = explanation_to_docstring(&explanation);
            assert_eq!(docstring, "/// This is a summary");
        }
        #[test]
        fn function_with_parameter_and_name() {
            let explanation = NodeExplanation {
                name_suggestion: Some("calculateFibonacci".to_string()),
                summary: "This is a summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![FunctionParameterWithExplanation {
                    name: "param1".to_string(),
                    explanation: "This is a UIElement".to_string(),
                    param_type: "UIElement".to_string(),
                }]),
            };
            let docstring = explanation_to_docstring(&explanation);
            assert_eq!(
                docstring,
                r#"/// This is a summary
/// - parameter param1: `UIElement` This is a UIElement"#
            );
        }

        #[test]
        fn function_with_two_parameters() {
            let explanation = NodeExplanation {
                name_suggestion: None,
                summary: "This is a summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![
                    FunctionParameterWithExplanation {
                        name: "param1".to_string(),
                        explanation: "This is a UIElement".to_string(),
                        param_type: "UIElement".to_string(),
                    },
                    FunctionParameterWithExplanation {
                        name: "param2".to_string(),
                        explanation: "This is a string".to_string(),
                        param_type: "String".to_string(),
                    },
                ]),
            };
            let docstring = explanation_to_docstring(&explanation);
            assert_eq!(
                docstring,
                r#"/// This is a summary
/// - parameter param1: `UIElement` This is a UIElement
/// - parameter param2: `String` This is a string"#
            );
        }
    }
    mod fetch_node_explanation {

        use tauri::async_runtime::block_on;

        use crate::core_engine::{
            features::docs_generation::node_annotation::AnnotationCodeBlock,
            syntax_tree::SwiftCodeBlockKind, TextPosition, XcodeText,
        };

        use super::super::{fetch_node_explanation, NodeExplanation};

        fn _fetch_node_explanation(codeblock: AnnotationCodeBlock) -> Option<NodeExplanation> {
            let handle = fetch_node_explanation(codeblock);
            block_on(handle).ok()
        }

        #[test]
        fn with_context() {
            let resp = _fetch_node_explanation(AnnotationCodeBlock {
                text: XcodeText::from_str("print(\"Hello World\")"),
                name: Some("my_fun".to_string()),
                first_char_pos: TextPosition { row: 0, column: 0 },
                last_char_pos: TextPosition { row: 0, column: 0 },
                kind: SwiftCodeBlockKind::Function,
                func_complexity_todo: None,
                func_parameters_todo: None,
                context: Some(XcodeText::from_str("print(\"Hello World\")")),
            });
            assert!(resp.is_some());
        }

        #[test]
        fn without_context() {
            let resp = _fetch_node_explanation(AnnotationCodeBlock {
                text: XcodeText::from_str("print(\"Hello World\")"),
                name: Some("my_fun".to_string()),
                func_parameters_todo: None,
                first_char_pos: TextPosition { row: 0, column: 0 },
                last_char_pos: TextPosition { row: 0, column: 0 },
                kind: SwiftCodeBlockKind::Function,
                func_complexity_todo: None,
                context: None,
            });
            assert!(resp.is_some());
        }
    }
}
