use std::fs;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Documentation extracted from source code comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDoc {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub responses: Vec<Response>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status_code: String,
    pub description: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub name: String,
    pub description: Option<String>,
    pub value: String,
}

/// Extracts documentation from Rust source files
pub struct DocExtractor {
    comment_regex: Regex,
    doc_comment_regex: Regex,
    tag_regex: Regex,
    param_regex: Regex,
    response_regex: Regex,
    example_regex: Regex,
}

impl DocExtractor {
    pub fn new() -> Self {
        Self {
            // Matches both /// and /** */ style comments
            comment_regex: Regex::new(r"(?m)^\s*///\s*(.*)$|/\*\*\s*(.*?)\s*\*/").unwrap(),
            // Matches doc comments with special tags
            doc_comment_regex: Regex::new(r"(?m)^\s*///\s*(.*)$").unwrap(),
            // Matches @tag annotations
            tag_regex: Regex::new(r"@(\w+)(?:\s+(.*))?").unwrap(),
            // Matches @param annotations
            param_regex: Regex::new(r"@param\s+(\w+)\s*\{([^}]+)\}\s*(.*)").unwrap(),
            // Matches @response annotations
            response_regex: Regex::new(r"@response\s+(\d+)\s*(.*)").unwrap(),
            // Matches @example annotations
            example_regex: Regex::new(r"@example\s*(?:(\w+))?\s*(.*)").unwrap(),
        }
    }

    /// Extract documentation from a controller function
    pub fn extract_from_function(&self, source: &str, function_name: &str) -> Option<ExtractedDoc> {
        // Find the function definition
        let function_regex = Regex::new(&format!(r"(?s)((?:///.*?\n\s*)*)\s*pub\s+async\s+fn\s+{}\s*\(", function_name)).unwrap();

        if let Some(captures) = function_regex.captures(source) {
            let comments = captures.get(1)?.as_str();
            return Some(self.parse_comments(comments));
        }
        None
    }

    /// Extract documentation from a struct (for models and requests)
    pub fn extract_from_struct(&self, source: &str, struct_name: &str) -> Option<ExtractedDoc> {
        // Find the struct definition
        let struct_regex = Regex::new(&format!(r"(?s)((?:///.*?\n\s*)*)\s*(?:#\[.*?\]\s*)*\s*pub\s+struct\s+{}", struct_name)).unwrap();

        if let Some(captures) = struct_regex.captures(source) {
            let comments = captures.get(1)?.as_str();
            return Some(self.parse_comments(comments));
        }
        None
    }

    /// Extract field documentation from a struct
    pub fn extract_struct_fields(&self, source: &str, struct_name: &str) -> Vec<Parameter> {
        let mut fields = Vec::new();

        // Find the struct definition and its fields
        let struct_regex = Regex::new(&format!(r"(?s)pub\s+struct\s+{}\s*\{{(.*?)\}}", struct_name)).unwrap();

        if let Some(captures) = struct_regex.captures(source) {
            let struct_body = captures.get(1).unwrap().as_str();

            // Extract field definitions with their comments
            let field_regex = Regex::new(r"(?s)((?:///.*?\n\s*)*)\s*pub\s+(\w+):\s*([^,\n]+)").unwrap();

            for field_match in field_regex.captures_iter(struct_body) {
                let comments = field_match.get(1).map(|m| m.as_str()).unwrap_or("");
                let field_name = field_match.get(2).unwrap().as_str();
                let field_type = field_match.get(3).unwrap().as_str().trim_end_matches(',').trim();

                let doc = self.parse_comments(comments);

                fields.push(Parameter {
                    name: field_name.to_string(),
                    param_type: field_type.to_string(),
                    description: doc.description.or(doc.summary),
                    required: !field_type.starts_with("Option<"),
                    example: None,
                });
            }
        }

        fields
    }

    /// Parse comment blocks and extract structured documentation
    fn parse_comments(&self, comments: &str) -> ExtractedDoc {
        let mut doc = ExtractedDoc {
            summary: None,
            description: None,
            tags: Vec::new(),
            parameters: Vec::new(),
            responses: Vec::new(),
            examples: Vec::new(),
        };

        let mut lines = Vec::new();

        // Extract all comment lines
        for line in comments.lines() {
            if let Some(captures) = self.doc_comment_regex.captures(line) {
                if let Some(content) = captures.get(1) {
                    lines.push(content.as_str().trim());
                }
            }
        }

        let _comment_text = lines.join("\n");

        // Parse special annotations
        for line in &lines {
            // Parse @tag annotations
            if let Some(captures) = self.tag_regex.captures(line) {
                let tag = captures.get(1).unwrap().as_str();
                match tag {
                    "param" => {
                        if let Some(param_captures) = self.param_regex.captures(line) {
                            doc.parameters.push(Parameter {
                                name: param_captures.get(1).unwrap().as_str().to_string(),
                                param_type: param_captures.get(2).unwrap().as_str().to_string(),
                                description: Some(param_captures.get(3).unwrap().as_str().to_string()),
                                required: true,
                                example: None,
                            });
                        }
                    },
                    "response" => {
                        if let Some(response_captures) = self.response_regex.captures(line) {
                            doc.responses.push(Response {
                                status_code: response_captures.get(1).unwrap().as_str().to_string(),
                                description: response_captures.get(2).unwrap().as_str().to_string(),
                                example: None,
                            });
                        }
                    },
                    "example" => {
                        if let Some(example_captures) = self.example_regex.captures(line) {
                            doc.examples.push(Example {
                                name: example_captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| "default".to_string()),
                                description: None,
                                value: example_captures.get(2).unwrap().as_str().to_string(),
                            });
                        }
                    },
                    _ => {
                        if let Some(tag_content) = captures.get(2) {
                            doc.tags.push(format!("{}: {}", tag, tag_content.as_str()));
                        } else {
                            doc.tags.push(tag.to_string());
                        }
                    }
                }
            }
        }

        // Extract summary and description from non-annotated lines
        let mut description_lines = Vec::new();
        let mut found_annotations = false;

        for line in &lines {
            if line.starts_with('@') {
                found_annotations = true;
                continue;
            }

            if !found_annotations && !line.is_empty() {
                description_lines.push(*line);
            }
        }

        if !description_lines.is_empty() {
            if description_lines.len() == 1 {
                doc.summary = Some(description_lines[0].to_string());
            } else {
                doc.summary = Some(description_lines[0].to_string());
                if description_lines.len() > 1 {
                    doc.description = Some(description_lines[1..].join("\n"));
                }
            }
        }

        doc
    }

    /// Extract documentation from all files in a directory
    pub fn extract_from_directory(&self, dir_path: &str) -> Result<Vec<(String, ExtractedDoc)>, Box<dyn std::error::Error>> {
        let mut docs = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path)?;
                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

                // Try to extract documentation from the main struct/function
                if let Some(doc) = self.extract_from_struct(&content, filename) {
                    docs.push((filename.to_string(), doc));
                }
            }
        }

        Ok(docs)
    }
}

impl Default for DocExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_doc() {
        let source = r#"
/// Get all countries with optional filtering and pagination
/// This endpoint supports various query parameters for filtering results.
/// @param page {number} Page number for pagination
/// @param limit {number} Number of items per page
/// @response 200 List of countries retrieved successfully
/// @response 500 Internal server error
/// @example {"page": 1, "limit": 10}
pub async fn index() -> impl IntoResponse {
    // function body
}
"#;

        let extractor = DocExtractor::new();
        let doc = extractor.extract_from_function(source, "index").unwrap();

        assert_eq!(doc.summary, Some("Get all countries with optional filtering and pagination".to_string()));
        assert!(doc.description.is_some());
        assert_eq!(doc.parameters.len(), 2);
        assert_eq!(doc.responses.len(), 2);
        assert_eq!(doc.examples.len(), 1);
    }

    #[test]
    fn test_extract_struct_doc() {
        let source = r#"
/// Request payload for creating a new country
/// Contains all required and optional fields for country creation.
/// @example {"name": "United States", "iso_code": "US", "phone_code": "+1"}
#[derive(Deserialize, Serialize)]
pub struct CreateCountryRequest {
    /// Country name (required)
    pub name: String,
    /// ISO country code (required)
    pub iso_code: String,
    /// Optional phone country code
    pub phone_code: Option<String>,
}
"#;

        let extractor = DocExtractor::new();
        let doc = extractor.extract_from_struct(source, "CreateCountryRequest").unwrap();

        assert_eq!(doc.summary, Some("Request payload for creating a new country".to_string()));
        assert!(doc.description.is_some());
        assert_eq!(doc.examples.len(), 1);

        let fields = extractor.extract_struct_fields(source, "CreateCountryRequest");
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name, "name");
        assert!(fields[0].required);
        assert_eq!(fields[2].name, "phone_code");
        assert!(!fields[2].required);
    }
}