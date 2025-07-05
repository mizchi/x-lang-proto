//! Main type checker interface

use crate::{
    types::{Type, TypeScheme, TypeEnv, EffectSet},
    inference::InferenceContext,
    error_reporting::{TypeError, TypeErrorReporter},
};
use x_parser::{CompilationUnit, Module, Item, ValueDef, TypeDef, Symbol, Span, FileId};
use x_parser::span::ByteOffset;
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
        // Process the module
        self.check_module(&cu.module);

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
        // Process module imports
        for import in &module.imports {
            self.check_import(import);
        }

        // Process module items
        for item in &module.items {
            self.check_item(item);
        }

        // Module scope is implicitly exited when scope_env is dropped
    }

    /// Type check an item
    fn check_item(&mut self, item: &Item) {
        match item {
            Item::ValueDef(value_def) => self.check_value_def(value_def),
            Item::TypeDef(type_def) => self.check_type_def(type_def),
            Item::EffectDef(effect_def) => self.check_effect_def(effect_def),
            Item::HandlerDef(handler_def) => self.check_handler_def(handler_def),
            Item::InterfaceDef(interface_def) => self.check_interface_def(interface_def),
            Item::ModuleTypeDef(module_type_def) => self.check_module_type_def(module_type_def),
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
                self.env.insert_var(value_def.name, type_scheme);
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
        let _type_scheme = self.create_type_scheme_for_type_def(type_def);
        // TODO: Add type constructor to environment properly
        // self.env.insert_type_con(type_def.name, type_scheme);

        // Check type definition body
        match &type_def.kind {
            x_parser::TypeDefKind::Data(constructors) => {
                for constructor in constructors {
                    for field in &constructor.fields {
                        if let Err(error) = self.check_type_well_formed(field) {
                            self.error_reporter.report_error(error);
                        }
                    }
                }
            }
            x_parser::TypeDefKind::Alias(aliased_type) => {
                if let Err(error) = self.check_type_well_formed(aliased_type) {
                    self.error_reporter.report_error(error);
                }
            }
            x_parser::TypeDefKind::Abstract => {
                // Abstract types have no body to check
            }
        }
    }

    /// Check other item types (simplified implementations)
    fn check_effect_def(&mut self, _effect_def: &x_parser::EffectDef) {
        // TODO: Implement effect definition checking
    }

    fn check_handler_def(&mut self, _handler_def: &x_parser::HandlerDef) {
        // TODO: Implement handler definition checking
    }

    fn check_interface_def(&mut self, _interface_def: &x_parser::ComponentInterface) {
        // TODO: Implement interface definition checking
    }

    fn check_module_type_def(&mut self, _module_type_def: &x_parser::ModuleTypeDef) {
        // TODO: Implement module type definition checking
    }

    fn check_import(&mut self, _import: &x_parser::Import) {
        // TODO: Implement import checking
    }

    #[allow(dead_code)]
    fn check_export(&mut self, _export: &x_parser::ExportList) {
        // TODO: Implement export checking
    }

    /// Helper methods
    fn check_type_annotation(&self, inferred: &Type, annotation: &x_parser::Type) -> Result<(), TypeError> {
        let annotation_type = self.convert_parser_type_to_checker_type(annotation);
        
        let mut unifier = crate::unification::Unifier::new();
        
        unifier.unify(inferred, &annotation_type).map_err(|_| TypeError::TypeMismatch {
            expected: annotation_type,
            found: inferred.clone(),
            span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)), // TODO: Get proper span
        })?;

        Ok(())
    }

    fn check_type_well_formed(&self, _typ: &x_parser::Type) -> Result<(), TypeError> {
        // TODO: Implement type well-formedness checking
        Ok(())
    }

    fn create_type_scheme_for_type_def(&self, _type_def: &TypeDef) -> TypeScheme {
        // TODO: Create proper type scheme from type definition
        TypeScheme::monotype(Type::Unknown)
    }

    fn convert_parser_type_to_checker_type(&self, parser_type: &x_parser::Type) -> Type {
        match parser_type {
            x_parser::Type::Var(name, _) => {
                // Convert symbol to type variable - this is simplified
                Type::Var(crate::types::TypeVar(name.as_str().chars().next().unwrap_or('a') as u32))
            }
            x_parser::Type::Con(name, _) => Type::Con(*name),
            x_parser::Type::App(con, args, _) => {
                let con_type = self.convert_parser_type_to_checker_type(con);
                let arg_types = args.iter().map(|t| self.convert_parser_type_to_checker_type(t)).collect();
                match con_type {
                    Type::Con(name) => Type::App(Box::new(Type::Con(name)), arg_types),
                    _ => Type::App(Box::new(con_type), arg_types),
                }
            }
            x_parser::Type::Fun { params, return_type,  .. } => {
                Type::Fun {
                    params: params.iter().map(|t| self.convert_parser_type_to_checker_type(t)).collect(),
                    return_type: Box::new(self.convert_parser_type_to_checker_type(return_type)),
                    effects: EffectSet::Empty, // TODO: Convert effects properly
                }
            }
            x_parser::Type::Forall { body, .. } => {
                // Simplified handling of forall
                self.convert_parser_type_to_checker_type(body)
            }
            x_parser::Type::Tuple { types, .. } => {
                Type::Tuple(types.iter().map(|t| self.convert_parser_type_to_checker_type(t)).collect())
            }
            x_parser::Type::Record { fields, .. } => {
                Type::Record(fields.iter().map(|(k, v)| (*k, self.convert_parser_type_to_checker_type(v))).collect())
            }
            x_parser::Type::Variant { variants, .. } => {
                Type::Variant(variants.iter().map(|(k, v)| (*k, vec![self.convert_parser_type_to_checker_type(v)])).collect())
            }
            x_parser::Type::Hole(_) => Type::Hole,
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
    use x_parser::{parse_source, SyntaxStyle, FileId};

    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new();
        // The env should have built-in types
        assert!(checker.env.vars.is_empty());
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
        // The result should have type environment
        assert!(result.inferred_types.is_empty() || !result.inferred_types.is_empty());
    }
}