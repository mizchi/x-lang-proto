//! Intent parsing from natural language
//! 
//! This module interprets user requests and converts them into structured intents.

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Represents the user's intent for code generation
#[derive(Debug, Clone)]
pub struct CodeIntent {
    pub action: IntentAction,
    pub target: IntentTarget,
    pub constraints: Vec<Constraint>,
    pub examples: Vec<Example>,
}

/// The primary action the user wants to perform
#[derive(Debug, Clone)]
pub enum IntentAction {
    Create,
    Modify,
    Refactor,
    Implement,
    Fix,
    Optimize,
    Test,
    Document,
}

/// What the user wants to act upon
#[derive(Debug, Clone)]
pub enum IntentTarget {
    Function {
        name: String,
        parameters: Vec<ParameterIntent>,
        return_type: Option<String>,
        description: String,
    },
    DataType {
        name: String,
        kind: DataTypeKind,
        fields: Vec<FieldIntent>,
    },
    Module {
        name: String,
        exports: Vec<String>,
    },
    Algorithm {
        name: String,
        complexity: Option<String>,
    },
    Interface {
        name: String,
        methods: Vec<MethodIntent>,
    },
    Effect {
        name: String,
        operations: Vec<OperationIntent>,
    },
}

#[derive(Debug, Clone)]
pub enum DataTypeKind {
    Record,
    Variant,
    Alias,
}

#[derive(Debug, Clone)]
pub struct ParameterIntent {
    pub name: String,
    pub typ: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldIntent {
    pub name: String,
    pub typ: String,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct MethodIntent {
    pub name: String,
    pub parameters: Vec<ParameterIntent>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OperationIntent {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: String,
}

/// Constraints on the generated code
#[derive(Debug, Clone)]
pub enum Constraint {
    Performance(PerformanceConstraint),
    Style(StyleConstraint),
    Compatibility(String),
    UseLibrary(String),
    AvoidFeature(String),
}

#[derive(Debug, Clone)]
pub enum PerformanceConstraint {
    TimeComplexity(String),
    SpaceComplexity(String),
    Tailrecursive,
}

#[derive(Debug, Clone)]
pub enum StyleConstraint {
    Functional,
    Imperative,
    PointFree,
    Verbose,
    Concise,
}

/// Example input/output pairs
#[derive(Debug, Clone)]
pub struct Example {
    pub input: String,
    pub output: String,
    pub description: Option<String>,
}

/// Intent for refining existing code
#[derive(Debug, Clone)]
pub struct RefinementIntent {
    pub action: RefinementAction,
    pub target: Option<String>,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum RefinementAction {
    ChangeType,
    AddParameter,
    RemoveParameter,
    RenameItem,
    AddCase,
    FixError,
    ImprovePerformance,
    Clarify,
}

/// Intent parser
pub struct IntentParser {
    patterns: HashMap<String, IntentPattern>,
}

struct IntentPattern {
    regex: Regex,
    action: IntentAction,
}

impl IntentParser {
    pub fn new() -> Self {
        let patterns = HashMap::new();
        Self { patterns }
    }
    
    /// Parse a natural language request into a code intent
    pub fn parse(&self, request: &str) -> Result<CodeIntent> {
        // Normalize the request
        let normalized = request.trim().to_lowercase();
        
        // Define regex patterns
        let create_function = Regex::new(
            r"(?i)(create|write|make|implement)\s+(?:a\s+)?function\s+(?:called\s+)?(\w+)"
        ).unwrap();
        let create_type = Regex::new(
            r"(?i)(create|define|make)\s+(?:a\s+)?(?:data\s+)?type\s+(?:called\s+)?(\w+)"
        ).unwrap();
        let implement_algorithm = Regex::new(
            r"(?i)implement\s+(?:the\s+)?(\w+)\s+algorithm"
        ).unwrap();
        
        // Try to match against known patterns
        if let Some(captures) = create_function.captures(&normalized) {
            self.parse_function_creation(request, captures)
        } else if let Some(captures) = create_type.captures(&normalized) {
            self.parse_type_creation(request, captures)
        } else if let Some(captures) = implement_algorithm.captures(&normalized) {
            self.parse_algorithm_implementation(request, captures)
        } else {
            // Fallback to more general parsing
            self.parse_general_intent(request)
        }
    }
    
    /// Parse refinement feedback
    pub fn parse_refinement(&self, feedback: &str) -> Result<RefinementIntent> {
        let normalized = feedback.trim().to_lowercase();
        
        let action = if normalized.contains("add") && normalized.contains("parameter") {
            RefinementAction::AddParameter
        } else if normalized.contains("change") && normalized.contains("type") {
            RefinementAction::ChangeType
        } else if normalized.contains("rename") {
            RefinementAction::RenameItem
        } else if normalized.contains("fix") {
            RefinementAction::FixError
        } else if normalized.contains("performance") || normalized.contains("faster") {
            RefinementAction::ImprovePerformance
        } else {
            RefinementAction::Clarify
        };
        
        Ok(RefinementIntent {
            action,
            target: self.extract_target(feedback),
            details: feedback.to_string(),
        })
    }
    
    fn parse_function_creation(&self, request: &str, captures: regex::Captures) -> Result<CodeIntent> {
        let function_name = captures.get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        
        // Extract parameters from phrases like "that takes x and y"
        let parameters = self.extract_parameters(request);
        
        // Extract return type from phrases like "returns a string"
        let return_type = self.extract_return_type(request);
        
        Ok(CodeIntent {
            action: IntentAction::Create,
            target: IntentTarget::Function {
                name: function_name,
                parameters,
                return_type,
                description: request.to_string(),
            },
            constraints: self.extract_constraints(request),
            examples: self.extract_examples(request),
        })
    }
    
    fn parse_type_creation(&self, request: &str, captures: regex::Captures) -> Result<CodeIntent> {
        let type_name = captures.get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "UnnamedType".to_string());
        
        let kind = if request.contains("variant") || request.contains("enum") {
            DataTypeKind::Variant
        } else if request.contains("record") || request.contains("struct") {
            DataTypeKind::Record
        } else {
            DataTypeKind::Alias
        };
        
        Ok(CodeIntent {
            action: IntentAction::Create,
            target: IntentTarget::DataType {
                name: type_name,
                kind,
                fields: self.extract_fields(request),
            },
            constraints: vec![],
            examples: vec![],
        })
    }
    
    fn parse_algorithm_implementation(&self, request: &str, captures: regex::Captures) -> Result<CodeIntent> {
        let algorithm_name = captures.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(CodeIntent {
            action: IntentAction::Implement,
            target: IntentTarget::Algorithm {
                name: algorithm_name,
                complexity: self.extract_complexity(request),
            },
            constraints: self.extract_constraints(request),
            examples: self.extract_examples(request),
        })
    }
    
    fn parse_general_intent(&self, request: &str) -> Result<CodeIntent> {
        // Fallback parsing logic
        Ok(CodeIntent {
            action: IntentAction::Create,
            target: IntentTarget::Function {
                name: "generated_function".to_string(),
                parameters: vec![],
                return_type: None,
                description: request.to_string(),
            },
            constraints: vec![],
            examples: vec![],
        })
    }
    
    fn extract_parameters(&self, text: &str) -> Vec<ParameterIntent> {
        let mut params = Vec::new();
        
        // Look for patterns like "takes a number x and a string y"
        let param_regex = Regex::new(r"(?i)takes?\s+(?:a\s+)?(\w+)\s+(\w+)").unwrap();
        
        for cap in param_regex.captures_iter(text) {
            if let (Some(typ), Some(name)) = (cap.get(1), cap.get(2)) {
                params.push(ParameterIntent {
                    name: name.as_str().to_string(),
                    typ: Some(typ.as_str().to_string()),
                    description: None,
                });
            }
        }
        
        params
    }
    
    fn extract_return_type(&self, text: &str) -> Option<String> {
        let return_regex = Regex::new(r"(?i)returns?\s+(?:a\s+)?(\w+)").unwrap();
        return_regex.captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }
    
    fn extract_fields(&self, text: &str) -> Vec<FieldIntent> {
        let mut fields = Vec::new();
        
        // Look for patterns like "with fields x: int, y: string"
        let field_regex = Regex::new(r"(\w+)\s*:\s*(\w+)").unwrap();
        
        for cap in field_regex.captures_iter(text) {
            if let (Some(name), Some(typ)) = (cap.get(1), cap.get(2)) {
                fields.push(FieldIntent {
                    name: name.as_str().to_string(),
                    typ: typ.as_str().to_string(),
                    optional: text.contains(&format!("optional {}", name.as_str())),
                });
            }
        }
        
        fields
    }
    
    fn extract_constraints(&self, text: &str) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        
        if text.contains("tail recursive") || text.contains("tail-recursive") {
            constraints.push(Constraint::Performance(PerformanceConstraint::Tailrecursive));
        }
        
        if text.contains("O(n)") || text.contains("linear time") {
            constraints.push(Constraint::Performance(PerformanceConstraint::TimeComplexity("O(n)".to_string())));
        }
        
        if text.contains("functional style") {
            constraints.push(Constraint::Style(StyleConstraint::Functional));
        }
        
        constraints
    }
    
    fn extract_examples(&self, text: &str) -> Vec<Example> {
        let mut examples = Vec::new();
        
        // Look for patterns like "for example, f(5) = 10"
        let example_regex = Regex::new(r"(?i)(?:for\s+)?example[,:]?\s*(\w+)\(([^)]+)\)\s*=\s*(.+)").unwrap();
        
        for cap in example_regex.captures_iter(text) {
            if let (Some(input), Some(output)) = (cap.get(2), cap.get(3)) {
                examples.push(Example {
                    input: input.as_str().to_string(),
                    output: output.as_str().to_string(),
                    description: None,
                });
            }
        }
        
        examples
    }
    
    fn extract_complexity(&self, text: &str) -> Option<String> {
        if text.contains("O(n log n)") {
            Some("O(n log n)".to_string())
        } else if text.contains("O(n)") {
            Some("O(n)".to_string())
        } else if text.contains("O(1)") {
            Some("O(1)".to_string())
        } else {
            None
        }
    }
    
    fn extract_target(&self, text: &str) -> Option<String> {
        // Extract the target of refinement (function name, type name, etc.)
        let target_regex = Regex::new(r"(?i)(?:the\s+)?(?:function|type|method)\s+(\w+)").unwrap();
        target_regex.captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Default for IntentParser {
    fn default() -> Self {
        Self::new()
    }
}