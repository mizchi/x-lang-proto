//! AI-powered code generation for x Language
//! 
//! This module provides the infrastructure for AI assistants to generate
//! x Language code from natural language descriptions.

pub mod intent;
pub mod context;
pub mod generator;
pub mod refiner;
pub mod validator;
pub mod session;

pub use intent::*;
pub use context::*;
pub use generator::*;
pub use refiner::*;
pub use validator::*;
pub use session::*;

use x_parser::ast::*;
use anyhow::Result;

/// Main AI code generation interface
pub struct AICodeGenerator {
    session: CodeGenSession,
    generator: CodeGenerator,
    validator: CodeValidator,
    refiner: CodeRefiner,
}

impl AICodeGenerator {
    pub fn new() -> Self {
        Self {
            session: CodeGenSession::new(),
            generator: CodeGenerator::new(),
            validator: CodeValidator::new(),
            refiner: CodeRefiner::new(),
        }
    }
    
    /// Generate code from a natural language request
    pub async fn generate_from_request(&mut self, request: &str) -> Result<GeneratedCode> {
        // 1. Parse intent from natural language
        let intent = self.parse_intent(request)?;
        
        // 2. Build context from current session
        let context = self.session.build_context(&intent)?;
        
        // 3. Generate initial code structure
        let initial_code = self.generator.generate(&intent, &context)?;
        
        // 4. Validate and get feedback
        let validation = self.validator.validate(&initial_code, &context)?;
        
        // 5. Refine based on validation
        let refined_code = if validation.has_issues() {
            self.refiner.refine(initial_code, &validation)?
        } else {
            initial_code
        };
        
        // 6. Update session
        self.session.add_generated_code(&refined_code);
        
        Ok(refined_code)
    }
    
    /// Interactive refinement based on user feedback
    pub async fn refine_with_feedback(
        &mut self, 
        code: GeneratedCode, 
        feedback: &str
    ) -> Result<GeneratedCode> {
        let refinement_intent = self.parse_refinement_intent(feedback)?;
        let context = self.session.build_refinement_context(&code, &refinement_intent)?;
        
        let refined = self.refiner.apply_feedback(code, &refinement_intent, &context)?;
        let validation = self.validator.validate(&refined, &context)?;
        
        if validation.has_critical_issues() {
            // Suggest fixes
            let suggestions = self.validator.suggest_fixes(&refined, &validation)?;
            Ok(GeneratedCode {
                ast: refined.ast,
                metadata: refined.metadata,
                suggestions: Some(suggestions),
                validation: Some(validation),
            })
        } else {
            self.session.add_generated_code(&refined);
            Ok(refined)
        }
    }
    
    /// Get completion suggestions for partial code
    pub async fn get_completions(&self, partial_code: &str) -> Result<Vec<CompletionSuggestion>> {
        let context = self.session.current_context();
        self.generator.suggest_completions(partial_code, &context)
    }
    
    /// Parse user intent from natural language
    fn parse_intent(&self, request: &str) -> Result<CodeIntent> {
        intent::IntentParser::new().parse(request)
    }
    
    /// Parse refinement intent from feedback
    fn parse_refinement_intent(&self, feedback: &str) -> Result<RefinementIntent> {
        intent::IntentParser::new().parse_refinement(feedback)
    }
}

/// Generated code with metadata
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub ast: CompilationUnit,
    pub metadata: GenerationMetadata,
    pub suggestions: Option<Vec<Suggestion>>,
    pub validation: Option<ValidationResult>,
}

/// Metadata about the generation process
#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    pub intent: CodeIntent,
    pub confidence: f64,
    pub alternatives: Vec<AlternativeCode>,
    pub explanation: String,
}

/// Alternative code generation
#[derive(Debug, Clone)]
pub struct AlternativeCode {
    pub ast: CompilationUnit,
    pub description: String,
    pub confidence: f64,
}

/// Suggestion for code improvement
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub kind: SuggestionKind,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum SuggestionKind {
    TypeAnnotation,
    ErrorHandling,
    Performance,
    Style,
    Documentation,
}


/// Location in code
#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub module: String,
    pub item: String,
    pub line: Option<usize>,
}

/// Completion suggestion
#[derive(Debug, Clone)]
pub struct CompletionSuggestion {
    pub text: String,
    pub kind: CompletionKind,
    pub description: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub enum CompletionKind {
    Function,
    Variable,
    Type,
    Module,
    Keyword,
    Pattern,
}

impl Default for AICodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}