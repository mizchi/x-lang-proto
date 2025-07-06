//! Code generator from intents to AST
//! 
//! This module transforms structured intents into x Language AST.

use anyhow::{Result, bail};
use x_parser::{Symbol, Span, FileId, span::ByteOffset};
use x_parser::ast::*;
use crate::{
    intent::*,
    context::*,
    GeneratedCode, GenerationMetadata, AlternativeCode,
    CompletionSuggestion, CompletionKind,
};

/// Code generator
pub struct CodeGenerator {
    file_id: FileId,
    template_library: TemplateLibrary,
}

/// Library of code templates
struct TemplateLibrary {
    function_templates: Vec<FunctionTemplate>,
    algorithm_templates: Vec<AlgorithmTemplate>,
}

/// Function template
struct FunctionTemplate {
    name: &'static str,
    pattern: &'static str,
    generate: fn(name: &str, params: &[ParameterIntent]) -> Expr,
}

/// Algorithm template
struct AlgorithmTemplate {
    name: &'static str,
    complexity: &'static str,
    generate: fn() -> Vec<Item>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            file_id: FileId::new(0),
            template_library: TemplateLibrary::new(),
        }
    }
    
    fn span(&self) -> Span {
        Span::new(self.file_id, ByteOffset::new(0), ByteOffset::new(1))
    }
    
    /// Generate code from intent and context
    pub fn generate(&self, intent: &CodeIntent, context: &CodeGenContext) -> Result<GeneratedCode> {
        let ast = match &intent.target {
            IntentTarget::Function { name, parameters, return_type, description } => {
                self.generate_function(name, parameters, return_type, description, &intent.constraints)?
            }
            IntentTarget::DataType { name, kind, fields } => {
                self.generate_data_type(name, kind, fields)?
            }
            IntentTarget::Module { name, exports } => {
                self.generate_module(name, exports)?
            }
            IntentTarget::Algorithm { name, complexity } => {
                self.generate_algorithm(name, complexity.as_deref(), &intent.constraints)?
            }
            IntentTarget::Interface { name, methods } => {
                self.generate_interface(name, methods)?
            }
            IntentTarget::Effect { name, operations } => {
                self.generate_effect(name, operations)?
            }
        };
        
        // Generate alternatives
        let alternatives = self.generate_alternatives(intent, context)?;
        
        // Create metadata
        let metadata = GenerationMetadata {
            intent: intent.clone(),
            confidence: self.calculate_confidence(intent, context),
            alternatives,
            explanation: self.generate_explanation(intent),
        };
        
        Ok(GeneratedCode {
            ast,
            metadata,
            suggestions: None,
            validation: None,
        })
    }
    
    /// Generate function from intent
    fn generate_function(
        &self,
        name: &str,
        parameters: &[ParameterIntent],
        return_type: &Option<String>,
        description: &str,
        constraints: &[Constraint],
    ) -> Result<CompilationUnit> {
        // Generate function body
        let body = self.generate_function_body(description, parameters, return_type, constraints)?;
        
        // Create value definition
        let value_def = if parameters.is_empty() {
            ValueDef {
                name: Symbol::intern(name),
                type_annotation: None,
                parameters: Vec::new(),
                body,
                visibility: Visibility::Public,
                purity: Purity::Inferred,
                span: self.span(),
            }
        } else {
            // Create lambda for function with parameters
            let params: Vec<Pattern> = parameters.iter()
                .map(|p| Pattern::Variable(Symbol::intern(&p.name), self.span()))
                .collect();
            
            ValueDef {
                name: Symbol::intern(name),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::Lambda {
                    parameters: params,
                    body: Box::new(body),
                    span: self.span(),
                },
                visibility: Visibility::Public,
                purity: Purity::Inferred,
                span: self.span(),
            }
        };
        
        // Create module
        let module = Module {
            name: ModulePath::single(Symbol::intern("Generated"), self.span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(value_def)],
            span: self.span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: self.span(),
        })
    }
    
    /// Generate function body from description
    fn generate_function_body(
        &self,
        description: &str,
        parameters: &[ParameterIntent],
        return_type: &Option<String>,
        constraints: &[Constraint],
    ) -> Result<Expr> {
        let desc_lower = description.to_lowercase();
        
        // Check for common patterns
        if desc_lower.contains("add") || desc_lower.contains("sum") {
            if parameters.len() == 2 {
                return Ok(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("+"), self.span())),
                    vec![
                        Expr::Var(Symbol::intern(&parameters[0].name), self.span()),
                        Expr::Var(Symbol::intern(&parameters[1].name), self.span()),
                    ],
                    self.span(),
                ));
            }
        }
        
        if desc_lower.contains("multiply") || desc_lower.contains("product") {
            if parameters.len() == 2 {
                return Ok(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("*"), self.span())),
                    vec![
                        Expr::Var(Symbol::intern(&parameters[0].name), self.span()),
                        Expr::Var(Symbol::intern(&parameters[1].name), self.span()),
                    ],
                    self.span(),
                ));
            }
        }
        
        // Check for tail recursion constraint
        let needs_tail_recursion = constraints.iter().any(|c| {
            matches!(c, Constraint::Performance(PerformanceConstraint::Tailrecursive))
        });
        
        if needs_tail_recursion && desc_lower.contains("factorial") {
            return self.generate_tail_recursive_factorial(parameters);
        }
        
        // Default: return unit
        Ok(Expr::Literal(Literal::Unit, self.span()))
    }
    
    /// Generate tail-recursive factorial
    fn generate_tail_recursive_factorial(&self, parameters: &[ParameterIntent]) -> Result<Expr> {
        if parameters.is_empty() {
            bail!("Factorial function requires at least one parameter");
        }
        
        let n = &parameters[0].name;
        let span = self.span();
        
        // Create helper function body: if n <= 0 then acc else fact_aux (n - 1) (n * acc)
        let helper_body = Expr::If {
            condition: Box::new(Expr::App(
                Box::new(Expr::Var(Symbol::intern("<="), span)),
                vec![
                    Expr::Var(Symbol::intern("n"), span),
                    Expr::Literal(Literal::Integer(0), span),
                ],
                span,
            )),
            then_branch: Box::new(Expr::Var(Symbol::intern("acc"), span)),
            else_branch: Box::new(Expr::App(
                Box::new(Expr::Var(Symbol::intern("fact_aux"), span)),
                vec![
                    Expr::App(
                        Box::new(Expr::Var(Symbol::intern("-"), span)),
                        vec![
                            Expr::Var(Symbol::intern("n"), span),
                            Expr::Literal(Literal::Integer(1), span),
                        ],
                        span,
                    ),
                    Expr::App(
                        Box::new(Expr::Var(Symbol::intern("*"), span)),
                        vec![
                            Expr::Var(Symbol::intern("n"), span),
                            Expr::Var(Symbol::intern("acc"), span),
                        ],
                        span,
                    ),
                ],
                span,
            )),
            span,
        };
        
        // Create helper function
        let helper = Expr::Lambda {
            parameters: vec![
                Pattern::Variable(Symbol::intern("n"), span),
                Pattern::Variable(Symbol::intern("acc"), span),
            ],
            body: Box::new(helper_body),
            span,
        };
        
        // Create let binding for helper
        Ok(Expr::Let {
            pattern: Pattern::Variable(Symbol::intern("fact_aux"), span),
            type_annotation: None,
            value: Box::new(helper),
            body: Box::new(Expr::App(
                Box::new(Expr::Var(Symbol::intern("fact_aux"), span)),
                vec![
                    Expr::Var(Symbol::intern(n), span),
                    Expr::Literal(Literal::Integer(1), span),
                ],
                span,
            )),
            span,
        })
    }
    
    /// Generate data type
    fn generate_data_type(
        &self,
        name: &str,
        kind: &DataTypeKind,
        fields: &[FieldIntent],
    ) -> Result<CompilationUnit> {
        let type_def = match kind {
            DataTypeKind::Variant => {
                // For variants, fields represent constructor names
                let constructors: Vec<Constructor> = fields.iter()
                    .map(|f| Constructor {
                        name: Symbol::intern(&f.name),
                        fields: Vec::new(),
                        span: self.span(),
                    })
                    .collect();
                
                TypeDef {
                    name: Symbol::intern(name),
                    type_params: Vec::new(),
                    kind: TypeDefKind::Data(constructors),
                    visibility: Visibility::Public,
                    span: self.span(),
                }
            }
            DataTypeKind::Record => {
                // Use record type as alias
                use std::collections::HashMap;
                let mut field_map = HashMap::new();
                
                for field in fields {
                    field_map.insert(
                        Symbol::intern(&field.name),
                        Type::Con(Symbol::intern(&field.typ), self.span()),
                    );
                }
                
                TypeDef {
                    name: Symbol::intern(name),
                    type_params: Vec::new(),
                    kind: TypeDefKind::Alias(Type::Record {
                        fields: field_map,
                        rest: None,
                        span: self.span(),
                    }),
                    visibility: Visibility::Public,
                    span: self.span(),
                }
            }
            DataTypeKind::Alias => {
                // Simple type alias
                let target = fields.first()
                    .map(|f| Type::Con(Symbol::intern(&f.typ), self.span()))
                    .unwrap_or(Type::Con(Symbol::intern("Unit"), self.span()));
                
                TypeDef {
                    name: Symbol::intern(name),
                    type_params: Vec::new(),
                    kind: TypeDefKind::Alias(target),
                    visibility: Visibility::Public,
                    span: self.span(),
                }
            }
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("Generated"), self.span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::TypeDef(type_def)],
            span: self.span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: self.span(),
        })
    }
    
    /// Generate module
    fn generate_module(
        &self,
        name: &str,
        exports: &[String],
    ) -> Result<CompilationUnit> {
        let export_list = if exports.is_empty() {
            None
        } else {
            Some(ExportList {
                items: exports.iter().map(|e| ExportItem {
                    kind: ExportKind::Value,
                    name: Symbol::intern(e),
                    alias: None,
                    span: self.span(),
                }).collect(),
                span: self.span(),
            })
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern(name), self.span()),
            exports: export_list,
            imports: Vec::new(),
            items: Vec::new(),
            span: self.span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: self.span(),
        })
    }
    
    /// Generate algorithm implementation
    fn generate_algorithm(
        &self,
        name: &str,
        complexity: Option<&str>,
        constraints: &[Constraint],
    ) -> Result<CompilationUnit> {
        // Check template library
        if let Some(template) = self.template_library.find_algorithm(name) {
            let items = (template.generate)();
            let module = Module {
                name: ModulePath::single(Symbol::intern("Generated"), self.span()),
                exports: None,
                imports: Vec::new(),
                items,
                span: self.span(),
            };
            
            return Ok(CompilationUnit {
                module,
                span: self.span(),
            });
        }
        
        // Generate based on algorithm name
        let item = match name.to_lowercase().as_str() {
            "quicksort" => self.generate_quicksort(),
            "fibonacci" => self.generate_fibonacci(constraints),
            _ => {
                // Generic algorithm placeholder
                Item::ValueDef(ValueDef {
                    name: Symbol::intern(name),
                    type_annotation: None,
                    parameters: Vec::new(),
                    body: Expr::Literal(Literal::Unit, self.span()),
                    visibility: Visibility::Public,
                    purity: Purity::Inferred,
                    span: self.span(),
                })
            }
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("Generated"), self.span()),
            exports: None,
            imports: Vec::new(),
            items: vec![item],
            span: self.span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: self.span(),
        })
    }
    
    /// Generate quicksort implementation
    fn generate_quicksort(&self) -> Item {
        let span = self.span();
        
        // Pattern: [] -> []
        let empty_arm = MatchArm {
            pattern: Pattern::Constructor {
                name: Symbol::intern("[]"),
                args: Vec::new(),
                span,
            },
            guard: None,
            body: Expr::Var(Symbol::intern("[]"), span),
            span,
        };
        
        // Pattern: pivot :: rest -> ...
        let cons_pattern = Pattern::Constructor {
            name: Symbol::intern("::"),
            args: vec![
                Pattern::Variable(Symbol::intern("pivot"), span),
                Pattern::Variable(Symbol::intern("rest"), span),
            ],
            span,
        };
        
        // Build the complex body for cons case
        let cons_body = {
            // smaller = filter (fun x -> x < pivot) rest
            let smaller_filter = Expr::App(
                Box::new(Expr::Var(Symbol::intern("filter"), span)),
                vec![
                    Expr::Lambda {
                        parameters: vec![Pattern::Variable(Symbol::intern("x"), span)],
                        body: Box::new(Expr::App(
                            Box::new(Expr::Var(Symbol::intern("<"), span)),
                            vec![
                                Expr::Var(Symbol::intern("x"), span),
                                Expr::Var(Symbol::intern("pivot"), span),
                            ],
                            span,
                        )),
                        span,
                    },
                    Expr::Var(Symbol::intern("rest"), span),
                ],
                span,
            );
            
            // larger = filter (fun x -> x >= pivot) rest
            let larger_filter = Expr::App(
                Box::new(Expr::Var(Symbol::intern("filter"), span)),
                vec![
                    Expr::Lambda {
                        parameters: vec![Pattern::Variable(Symbol::intern("x"), span)],
                        body: Box::new(Expr::App(
                            Box::new(Expr::Var(Symbol::intern(">="), span)),
                            vec![
                                Expr::Var(Symbol::intern("x"), span),
                                Expr::Var(Symbol::intern("pivot"), span),
                            ],
                            span,
                        )),
                        span,
                    },
                    Expr::Var(Symbol::intern("rest"), span),
                ],
                span,
            );
            
            // append (quicksort smaller) (pivot :: quicksort larger)
            let result = Expr::App(
                Box::new(Expr::Var(Symbol::intern("append"), span)),
                vec![
                    Expr::App(
                        Box::new(Expr::Var(Symbol::intern("quicksort"), span)),
                        vec![Expr::Var(Symbol::intern("smaller"), span)],
                        span,
                    ),
                    Expr::App(
                        Box::new(Expr::Var(Symbol::intern("::"), span)),
                        vec![
                            Expr::Var(Symbol::intern("pivot"), span),
                            Expr::App(
                                Box::new(Expr::Var(Symbol::intern("quicksort"), span)),
                                vec![Expr::Var(Symbol::intern("larger"), span)],
                                span,
                            ),
                        ],
                        span,
                    ),
                ],
                span,
            );
            
            // let smaller = ... in let larger = ... in result
            Expr::Let {
                pattern: Pattern::Variable(Symbol::intern("smaller"), span),
                type_annotation: None,
                value: Box::new(smaller_filter),
                body: Box::new(Expr::Let {
                    pattern: Pattern::Variable(Symbol::intern("larger"), span),
                    type_annotation: None,
                    value: Box::new(larger_filter),
                    body: Box::new(result),
                    span,
                }),
                span,
            }
        };
        
        let cons_arm = MatchArm {
            pattern: cons_pattern,
            guard: None,
            body: cons_body,
            span,
        };
        
        // Build match expression
        let match_expr = Expr::Match {
            scrutinee: Box::new(Expr::Var(Symbol::intern("lst"), span)),
            arms: vec![empty_arm, cons_arm],
            span,
        };
        
        // Create lambda
        let lambda = Expr::Lambda {
            parameters: vec![Pattern::Variable(Symbol::intern("lst"), span)],
            body: Box::new(match_expr),
            span,
        };
        
        Item::ValueDef(ValueDef {
            name: Symbol::intern("quicksort"),
            type_annotation: None,
            parameters: Vec::new(),
            body: lambda,
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            span,
        })
    }
    
    /// Generate Fibonacci implementation
    fn generate_fibonacci(&self, constraints: &[Constraint]) -> Item {
        let span = self.span();
        let is_tail_recursive = constraints.iter().any(|c| {
            matches!(c, Constraint::Performance(PerformanceConstraint::Tailrecursive))
        });
        
        let body = if is_tail_recursive {
            // Tail-recursive version with helper
            let helper_body = Expr::If {
                condition: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("<="), span)),
                    vec![
                        Expr::Var(Symbol::intern("n"), span),
                        Expr::Literal(Literal::Integer(0), span),
                    ],
                    span,
                )),
                then_branch: Box::new(Expr::Var(Symbol::intern("a"), span)),
                else_branch: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("fib_aux"), span)),
                    vec![
                        Expr::App(
                            Box::new(Expr::Var(Symbol::intern("-"), span)),
                            vec![
                                Expr::Var(Symbol::intern("n"), span),
                                Expr::Literal(Literal::Integer(1), span),
                            ],
                            span,
                        ),
                        Expr::Var(Symbol::intern("b"), span),
                        Expr::App(
                            Box::new(Expr::Var(Symbol::intern("+"), span)),
                            vec![
                                Expr::Var(Symbol::intern("a"), span),
                                Expr::Var(Symbol::intern("b"), span),
                            ],
                            span,
                        ),
                    ],
                    span,
                )),
                span,
            };
            
            let helper = Expr::Lambda {
                parameters: vec![
                    Pattern::Variable(Symbol::intern("n"), span),
                    Pattern::Variable(Symbol::intern("a"), span),
                    Pattern::Variable(Symbol::intern("b"), span),
                ],
                body: Box::new(helper_body),
                span,
            };
            
            // Main function body
            Expr::Lambda {
                parameters: vec![Pattern::Variable(Symbol::intern("n"), span)],
                body: Box::new(Expr::Let {
                    pattern: Pattern::Variable(Symbol::intern("fib_aux"), span),
                    type_annotation: None,
                    value: Box::new(helper),
                    body: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern("fib_aux"), span)),
                        vec![
                            Expr::Var(Symbol::intern("n"), span),
                            Expr::Literal(Literal::Integer(0), span),
                            Expr::Literal(Literal::Integer(1), span),
                        ],
                        span,
                    )),
                    span,
                }),
                span,
            }
        } else {
            // Simple recursive version
            Expr::Lambda {
                parameters: vec![Pattern::Variable(Symbol::intern("n"), span)],
                body: Box::new(Expr::If {
                    condition: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern("<="), span)),
                        vec![
                            Expr::Var(Symbol::intern("n"), span),
                            Expr::Literal(Literal::Integer(1), span),
                        ],
                        span,
                    )),
                    then_branch: Box::new(Expr::Var(Symbol::intern("n"), span)),
                    else_branch: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern("+"), span)),
                        vec![
                            Expr::App(
                                Box::new(Expr::Var(Symbol::intern("fibonacci"), span)),
                                vec![Expr::App(
                                    Box::new(Expr::Var(Symbol::intern("-"), span)),
                                    vec![
                                        Expr::Var(Symbol::intern("n"), span),
                                        Expr::Literal(Literal::Integer(1), span),
                                    ],
                                    span,
                                )],
                                span,
                            ),
                            Expr::App(
                                Box::new(Expr::Var(Symbol::intern("fibonacci"), span)),
                                vec![Expr::App(
                                    Box::new(Expr::Var(Symbol::intern("-"), span)),
                                    vec![
                                        Expr::Var(Symbol::intern("n"), span),
                                        Expr::Literal(Literal::Integer(2), span),
                                    ],
                                    span,
                                )],
                                span,
                            ),
                        ],
                        span,
                    )),
                    span,
                }),
                span,
            }
        };
        
        Item::ValueDef(ValueDef {
            name: Symbol::intern("fibonacci"),
            type_annotation: None,
            parameters: Vec::new(),
            body,
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            span,
        })
    }
    
    /// Generate interface (trait)
    fn generate_interface(
        &self,
        name: &str,
        methods: &[MethodIntent],
    ) -> Result<CompilationUnit> {
        // x Language doesn't have interfaces/traits yet, generate as comment
        let comment = format!("Interface {} with methods: {:?}", name, methods);
        
        let value_def = ValueDef {
            name: Symbol::intern(&format!("{}_interface", name)),
            type_annotation: None,
            parameters: Vec::new(),
            body: Expr::Literal(Literal::String(comment), self.span()),
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            span: self.span(),
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("Generated"), self.span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(value_def)],
            span: self.span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: self.span(),
        })
    }
    
    /// Generate effect definition
    fn generate_effect(
        &self,
        name: &str,
        operations: &[OperationIntent],
    ) -> Result<CompilationUnit> {
        let ops: Vec<EffectOperation> = operations.iter()
            .map(|op| {
                let params: Vec<Type> = op.parameters.iter()
                    .map(|p| Type::Con(Symbol::intern(p), self.span()))
                    .collect();
                
                EffectOperation {
                    name: Symbol::intern(&op.name),
                    parameters: params,
                    return_type: Type::Con(Symbol::intern(&op.return_type), self.span()),
                    span: self.span(),
                }
            })
            .collect();
        
        let effect_def = EffectDef {
            name: Symbol::intern(name),
            type_params: Vec::new(),
            operations: ops,
            visibility: Visibility::Public,
            span: self.span(),
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("Generated"), self.span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::EffectDef(effect_def)],
            span: self.span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: self.span(),
        })
    }
    
    /// Generate alternative implementations
    fn generate_alternatives(
        &self,
        intent: &CodeIntent,
        context: &CodeGenContext,
    ) -> Result<Vec<AlternativeCode>> {
        // Currently no alternatives
        Ok(Vec::new())
    }
    
    /// Calculate confidence score
    fn calculate_confidence(&self, intent: &CodeIntent, context: &CodeGenContext) -> f64 {
        let mut confidence = 0.5;
        
        // Increase confidence if we have examples
        if !intent.examples.is_empty() {
            confidence += 0.1 * intent.examples.len().min(3) as f64;
        }
        
        // Increase confidence for well-known patterns
        match &intent.target {
            IntentTarget::Algorithm { name, .. } => {
                if self.template_library.has_algorithm(name) {
                    confidence += 0.2;
                }
            }
            _ => {}
        }
        
        confidence.clamp(0.1, 0.95)
    }
    
    /// Generate explanation of the generated code
    fn generate_explanation(&self, intent: &CodeIntent) -> String {
        match &intent.target {
            IntentTarget::Function { name, parameters, .. } => {
                format!(
                    "Generated function '{}' with {} parameter(s) based on the description.",
                    name,
                    parameters.len()
                )
            }
            IntentTarget::DataType { name, kind, fields } => {
                format!(
                    "Generated {} type '{}' with {} field(s).",
                    match kind {
                        DataTypeKind::Record => "record",
                        DataTypeKind::Variant => "variant",
                        DataTypeKind::Alias => "alias",
                    },
                    name,
                    fields.len()
                )
            }
            IntentTarget::Algorithm { name, complexity } => {
                format!(
                    "Generated {} algorithm{}.",
                    name,
                    complexity.as_ref()
                        .map(|c| format!(" with {} complexity", c))
                        .unwrap_or_default()
                )
            }
            _ => "Generated code based on the provided intent.".to_string(),
        }
    }
    
    /// Suggest completions for partial code
    pub fn suggest_completions(
        &self,
        partial_code: &str,
        context: &CodeGenContext,
    ) -> Result<Vec<CompletionSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Analyze partial code to determine context
        let tokens: Vec<&str> = partial_code.split_whitespace().collect();
        
        if tokens.is_empty() {
            // Suggest top-level constructs
            suggestions.extend(vec![
                CompletionSuggestion {
                    text: "let ".to_string(),
                    kind: CompletionKind::Keyword,
                    description: "Define a value".to_string(),
                    score: 0.9,
                },
                CompletionSuggestion {
                    text: "data ".to_string(),
                    kind: CompletionKind::Keyword,
                    description: "Define a data type".to_string(),
                    score: 0.8,
                },
            ]);
        }
        
        Ok(suggestions)
    }
}

impl TemplateLibrary {
    fn new() -> Self {
        Self {
            function_templates: Vec::new(),
            algorithm_templates: Vec::new(),
        }
    }
    
    fn has_algorithm(&self, name: &str) -> bool {
        self.algorithm_templates.iter()
            .any(|t| t.name.eq_ignore_ascii_case(name))
    }
    
    fn find_algorithm(&self, name: &str) -> Option<&AlgorithmTemplate> {
        self.algorithm_templates.iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }
}