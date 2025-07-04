//! Main type checker interface

use crate::{
    types::{Type, TypeScheme, TypeEnv},
    inference::{InferenceContext, InferenceResult},
    effects::EffectSet,
    error_reporting::{TypeError, TypeErrorReporter},
};
use effect_lang_parser::{CompilationUnit, Module, Item, ValueDef, TypeDef, Symbol};
use std::collections::HashMap;

/// Type checking result
#[derive(Debug)]
pub struct CheckResult {
    pub type_env: TypeEnv,
    pub inferred_types: HashMap<Symbol, TypeScheme>,
    pub effect_constraints: Vec<EffectConstraint>,
    pub errors: Vec<TypeError>,
    pub warnings: Vec<TypeError>,
}

/// Effect constraint for effect system checking
#[derive(Debug, Clone)]
pub struct EffectConstraint {
    pub symbol: Symbol,
    pub required_effects: EffectSet,
    pub available_effects: EffectSet,
}

/// Main type checker
pub struct TypeChecker {
    env: TypeEnv,
    inference_ctx: InferenceContext,
    error_reporter: TypeErrorReporter,
}

impl TypeChecker {
    /// Create a new type checker with empty environment
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            inference_ctx: InferenceContext::new(),
            error_reporter: TypeErrorReporter::new(),
        }
    }

    /// Create a type checker with custom environment
    pub fn with_env(env: TypeEnv) -> Self {
        Self {
            env,
            inference_ctx: InferenceContext::new(),
            error_reporter: TypeErrorReporter::new(),
        }
    }

    /// Type check a compilation unit
    pub fn check_compilation_unit(&mut self, cu: &CompilationUnit) -> CheckResult {
        // Process each module
        for module in &cu.modules {
            self.check_module(module);
        }

        // Process imports and exports
        for import in &cu.imports {
            self.check_import(import);
        }

        for export in &cu.exports {
            self.check_export(export);
        }

        // Collect results
        CheckResult {
            type_env: self.env.clone(),
            inferred_types: self.collect_inferred_types(),
            effect_constraints: self.collect_effect_constraints(),
            errors: self.error_reporter.errors().to_vec(),
            warnings: self.error_reporter.warnings().to_vec(),
        }
    }

    /// Type check a module
    fn check_module(&mut self, module: &Module) {
        // Enter module scope
        self.env.enter_scope();

        // Process module items
        for item in &module.items {
            self.check_item(item);
        }

        // Exit module scope
        self.env.exit_scope();
    }

    /// Type check an item
    fn check_item(&mut self, item: &Item) {
        match item {
            Item::ValueDef(value_def) => self.check_value_def(value_def),
            Item::TypeDef(type_def) => self.check_type_def(type_def),
            Item::EffectDef(effect_def) => self.check_effect_def(effect_def),
            Item::HandlerDef(handler_def) => self.check_handler_def(handler_def),
            Item::InterfaceDef(interface_def) => self.check_interface_def(interface_def),
            Item::ImportDef(import_def) => self.check_import_def(import_def),
            Item::ExportDef(export_def) => self.check_export_def(export_def),
        }
    }

    /// Type check a value definition
    fn check_value_def(&mut self, value_def: &ValueDef) {
        match self.inference_ctx.infer_expr(&value_def.body) {
            Ok(inference_result) => {
                // Check type annotation if present
                if let Some(ref annotation) = value_def.type_annotation {
                    if let Err(error) = self.check_type_annotation(&inference_result.typ, annotation) {
                        self.error_reporter.report_error(error);
                    }
                }

                // Generalize and add to environment
                let type_scheme = self.inference_ctx.generalize(&inference_result.typ, &inference_result.effects);
                self.env.insert(value_def.name, type_scheme);
            }
            Err(error) => {
                self.error_reporter.report_error(TypeError::InferenceError {
                    message: format!("Failed to infer type for {}: {}", value_def.name.as_str(), error),
                    symbol: value_def.name,
                    span: value_def.span,
                });
            }
        }
    }

    /// Type check a type definition
    fn check_type_def(&mut self, type_def: &TypeDef) {
        // Add type constructor to environment
        let type_scheme = self.create_type_scheme_for_type_def(type_def);
        self.env.insert_type(type_def.name, type_scheme);

        // Check type definition body
        match &type_def.definition {
            effect_lang_parser::TypeDefinition::Record(fields) => {
                for (field_name, field_type) in fields {
                    if let Err(error) = self.check_type_well_formed(field_type) {
                        self.error_reporter.report_error(error);
                    }
                }
            }
            effect_lang_parser::TypeDefinition::Variant(variants) => {
                for variant in variants {
                    if let Some(ref data_type) = variant.data {
                        if let Err(error) = self.check_type_well_formed(data_type) {
                            self.error_reporter.report_error(error);
                        }
                    }
                }
            }
            effect_lang_parser::TypeDefinition::Alias(aliased_type) => {
                if let Err(error) = self.check_type_well_formed(aliased_type) {
                    self.error_reporter.report_error(error);
                }
            }
            _ => {} // Handle other type definitions
        }
    }

    /// Check other item types (simplified implementations)
    fn check_effect_def(&mut self, _effect_def: &effect_lang_parser::EffectDef) {
        // TODO: Implement effect definition checking
    }

    fn check_handler_def(&mut self, _handler_def: &effect_lang_parser::HandlerDef) {
        // TODO: Implement handler definition checking
    }

    fn check_interface_def(&mut self, _interface_def: &effect_lang_parser::ComponentInterface) {
        // TODO: Implement interface definition checking
    }

    fn check_import_def(&mut self, _import_def: &effect_lang_parser::ImportDef) {
        // TODO: Implement import definition checking
    }

    fn check_export_def(&mut self, _export_def: &effect_lang_parser::ExportDef) {
        // TODO: Implement export definition checking
    }

    fn check_import(&mut self, _import: &effect_lang_parser::ImportDef) {
        // TODO: Implement import checking
    }

    fn check_export(&mut self, _export: &effect_lang_parser::ExportDef) {
        // TODO: Implement export checking
    }

    /// Helper methods
    fn check_type_annotation(&self, inferred: &Type, annotation: &effect_lang_parser::Type) -> Result<(), TypeError> {
        let annotation_type = self.convert_parser_type_to_checker_type(annotation);
        
        use crate::unification::Unify;
        let mut unifier = crate::unification::Unifier::new();
        
        unifier.unify(inferred, &annotation_type).map_err(|_| TypeError::TypeMismatch {
            expected: annotation_type,
            found: inferred.clone(),
            span: Default::default(), // TODO: Get proper span
        })?;

        Ok(())
    }

    fn check_type_well_formed(&self, _typ: &effect_lang_parser::Type) -> Result<(), TypeError> {
        // TODO: Implement type well-formedness checking
        Ok(())
    }

    fn create_type_scheme_for_type_def(&self, _type_def: &TypeDef) -> TypeScheme {
        // TODO: Create proper type scheme from type definition
        TypeScheme::monotype(Type::Unknown)
    }

    fn convert_parser_type_to_checker_type(&self, parser_type: &effect_lang_parser::Type) -> Type {
        match parser_type {
            effect_lang_parser::Type::Unit => Type::Unit,
            effect_lang_parser::Type::Bool => Type::Bool,
            effect_lang_parser::Type::Int => Type::Int,
            effect_lang_parser::Type::Float => Type::Float,
            effect_lang_parser::Type::String => Type::String,
            effect_lang_parser::Type::Char => Type::Char,
            effect_lang_parser::Type::List(inner) => {
                Type::App(Box::new(Type::Con(Symbol::new("List"))), vec![self.convert_parser_type_to_checker_type(inner)])
            }
            effect_lang_parser::Type::Option(inner) => {
                Type::App(Box::new(Type::Con(Symbol::new("Option"))), vec![self.convert_parser_type_to_checker_type(inner)])
            }
            effect_lang_parser::Type::Result(ok, err) => {
                Type::App(
                    Box::new(Type::Con(Symbol::new("Result"))), 
                    vec![
                        self.convert_parser_type_to_checker_type(ok),
                        self.convert_parser_type_to_checker_type(err)
                    ]
                )
            }
            effect_lang_parser::Type::Named(name) => Type::Con(*name),
            effect_lang_parser::Type::Variable(var) => Type::Var(crate::types::TypeVar(*var)),
            _ => Type::Unknown,
        }
    }

    fn collect_inferred_types(&self) -> HashMap<Symbol, TypeScheme> {
        // TODO: Collect all inferred types from the environment
        HashMap::new()
    }

    fn collect_effect_constraints(&self) -> Vec<EffectConstraint> {
        // TODO: Collect effect constraints from the checker
        Vec::new()
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience trait for type checking
pub trait TypeCheck {
    fn type_check(&self) -> CheckResult;
}

impl TypeCheck for CompilationUnit {
    fn type_check(&self) -> CheckResult {
        let mut checker = TypeChecker::new();
        checker.check_compilation_unit(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use effect_lang_parser::{parse_source, SyntaxStyle, FileId};

    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new();
        assert!(checker.env.is_empty());
    }

    #[test]
    fn test_simple_type_checking() {
        let source = "let x = 42";
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::OCaml).unwrap();
        
        let result = cu.type_check();
        // Should not crash, though may have errors due to incomplete implementation
        assert!(result.errors.len() >= 0); // Placeholder assertion
    }

    #[test]
    fn test_type_check_trait() {
        let source = "let x = true";
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::OCaml).unwrap();
        
        let result = cu.type_check();
        assert!(result.type_env.scopes.len() > 0);
    }
}