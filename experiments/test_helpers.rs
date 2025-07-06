//! Helper functions for test discovery from parsed AST

use anyhow::Result;
use x_parser::{
    CompilationUnit, Item, ValueDef, Type as AstType, 
    Symbol, ast::Visibility as AstVisibility
};
use x_editor::{
    namespace::{Namespace, NamespacePath, NameBinding, Visibility},
    content_addressing::ContentHash,
};
use x_checker::{TypeChecker, CheckResult, types::TypeScheme};

/// Convert a parsed CompilationUnit to a Namespace with test functions
pub fn compilation_unit_to_namespace(
    compilation_unit: &CompilationUnit,
    namespace_path: NamespacePath,
    check_result: &CheckResult,
) -> Result<Namespace> {
    let mut namespace = Namespace::new(namespace_path);
    
    // Extract all value definitions from the module
    for item in &compilation_unit.module.items {
        if let Item::ValueDef(value_def) = item {
            // Check if this might be a test function
            if is_potential_test_function(value_def) {
                // Create a NameBinding for this function
                let hash = ContentHash::new(value_def.name.as_str().as_bytes());
                
                // Get type information from check result if available
                let type_scheme = check_result.inferred_types
                    .get(&value_def.name)
                    .cloned();
                
                let visibility = convert_visibility(&value_def.visibility);
                
                namespace.bindings.insert(
                    value_def.name.clone(),
                    NameBinding::Value {
                        hash,
                        type_scheme,
                        visibility,
                    },
                );
            }
        }
    }
    
    Ok(namespace)
}

/// Check if a value definition might be a test function
fn is_potential_test_function(value_def: &ValueDef) -> bool {
    let name = value_def.name.as_str();
    
    // Name-based detection
    if name.starts_with("test_") || (name.starts_with("test") && name.len() > 4) {
        return true;
    }
    
    // Type-based detection: check if return type is Bool or Unit
    if let Some(type_ann) = &value_def.type_annotation {
        if is_test_return_type(type_ann) {
            return true;
        }
    }
    
    false
}

/// Check if a type is suitable for a test function (Bool or Unit)
fn is_test_return_type(ty: &AstType) -> bool {
    match ty {
        AstType::Con(name, _) => {
            let name_str = name.as_str();
            name_str == "Bool" || name_str == "Unit"
        }
        AstType::Fun { params: _, return_type, .. } => {
            is_test_return_type(return_type)
        }
        _ => false,
    }
}

/// Convert AST visibility to namespace visibility
fn convert_visibility(ast_vis: &AstVisibility) -> Visibility {
    match ast_vis {
        AstVisibility::Public => Visibility::Public,
        AstVisibility::Private => Visibility::Private,
        AstVisibility::Crate => Visibility::Public,     // Treat as public for testing
        AstVisibility::Package => Visibility::Public,   // Treat as public for testing
        AstVisibility::Super => Visibility::Protected,  // Treat as protected
        AstVisibility::InPath(_) => Visibility::Public, // Treat as public
        AstVisibility::SelfModule => Visibility::Private, // Same as private
        AstVisibility::Component { .. } => Visibility::Public, // Component visibility
    }
}