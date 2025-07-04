//! Binary AST Type Checker
//! 
//! This module provides type checking directly on binary AST format,
//! enabling faster type analysis by avoiding AST reconstruction.

use crate::core::{
    binary::{BinaryDeserializer, TypeCode, TypedBinaryNode},
    symbol::Symbol,
    span::Span,
};
use crate::analysis::{
    types::*,
    inference::{InferenceContext, InferenceResult},
    error_reporting::{TypeError, ErrorReporter},
};
use crate::{Error, Result};
use std::collections::HashMap;

/// Binary type checker that operates directly on binary AST
#[derive(Debug)]
pub struct BinaryTypeChecker {
    type_env: TypeEnv,
    var_gen: VarGen,
    error_reporter: ErrorReporter,
    type_cache: HashMap<u32, Type>,
    effect_cache: HashMap<u32, EffectSet>,
    inference_cache: HashMap<u32, InferenceResult>,
}

impl BinaryTypeChecker {
    pub fn new() -> Self {
        BinaryTypeChecker {
            type_env: TypeEnv::new(),
            var_gen: VarGen::new(),
            error_reporter: ErrorReporter::new(),
            type_cache: HashMap::new(),
            effect_cache: HashMap::new(),
            inference_cache: HashMap::new(),
        }
    }
    
    /// Type check a binary compilation unit
    pub fn check_binary_compilation_unit(&mut self, binary_data: &[u8]) -> Result<TypeCheckResult> {
        let mut deserializer = BinaryDeserializer::new(binary_data.to_vec())?;
        
        // Check if binary contains cached type information
        let has_types = deserializer.has_type_information();
        let has_effects = deserializer.has_cached_effects();
        
        if has_types && has_effects {
            // Use cached type information for fast validation
            self.validate_cached_types(&mut deserializer)
        } else {
            // Perform full type inference on binary AST
            self.infer_types_from_binary(&mut deserializer)
        }
    }
    
    /// Validate pre-computed type information in binary format
    fn validate_cached_types(&mut self, deserializer: &mut BinaryDeserializer) -> Result<TypeCheckResult> {
        let mut results = Vec::new();
        let mut node_count = 0;
        
        // Load type metadata
        self.load_type_cache(deserializer)?;
        
        // Validate each typed node
        while let Ok((expr, typ, effects)) = deserializer.deserialize_typed_expr() {
            node_count += 1;
            
            if let (Some(cached_type), Some(cached_effects)) = (typ, effects) {
                // Validate cached type against current environment
                if let Err(validation_error) = self.validate_type_in_context(&cached_type, &cached_effects) {
                    // Convert Error to TypeError for reporting
                    let type_error = TypeError::unbound_variable(
                        Symbol::intern("unknown"),
                        crate::analysis::error_reporting::VariableKind::Type,
                        Span::new(crate::core::span::FileId::new(0), crate::core::span::ByteOffset::new(0), crate::core::span::ByteOffset::new(0)),
                    );
                    self.error_reporter.report(type_error);
                }
                
                results.push(TypedNode {
                    node_id: node_count,
                    inferred_type: cached_type,
                    effects: cached_effects,
                    span: expr.span(),
                });
            } else {
                // Missing type information - create default result
                let inference_result = InferenceResult {
                    typ: Type::Hole,
                    effects: EffectSet::Empty,
                    constraints: Vec::new(),
                };
                results.push(TypedNode {
                    node_id: node_count,
                    inferred_type: inference_result.typ,
                    effects: inference_result.effects,
                    span: expr.span(),
                });
            }
        }
        
        Ok(TypeCheckResult {
            typed_nodes: results,
            errors: self.error_reporter.errors().to_vec(),
            validation_mode: ValidationMode::CachedValidation,
            node_count,
        })
    }
    
    /// Perform type inference directly on binary AST
    fn infer_types_from_binary(&mut self, deserializer: &mut BinaryDeserializer) -> Result<TypeCheckResult> {
        let mut results = Vec::new();
        let mut node_count = 0;
        
        // Create inference context
        let mut inference_ctx = InferenceContext::new();
        
        // Process binary nodes sequentially
        while let Ok(node) = deserializer.deserialize_binary_node() {
            node_count += 1;
            
            match self.infer_binary_node(&mut inference_ctx, &node) {
                Ok(inference_result) => {
                    results.push(TypedNode {
                        node_id: node_count,
                        inferred_type: inference_result.typ,
                        effects: inference_result.effects,
                        span: node.span,
                    });
                }
                Err(error) => {
                    let type_error = TypeError::unbound_variable(
                        Symbol::intern("unknown"),
                        crate::analysis::error_reporting::VariableKind::Type,
                        Span::new(crate::core::span::FileId::new(0), crate::core::span::ByteOffset::new(0), crate::core::span::ByteOffset::new(0)),
                    );
                    self.error_reporter.report(type_error);
                }
            }
        }
        
        Ok(TypeCheckResult {
            typed_nodes: results,
            errors: self.error_reporter.errors().to_vec(),
            validation_mode: ValidationMode::FullInference,
            node_count,
        })
    }
    
    /// Infer type for a single binary node
    fn infer_binary_node(&mut self, ctx: &mut InferenceContext, node: &TypedBinaryNode) -> Result<InferenceResult> {
        match node.node_type {
            TypeCode::LiteralInteger => {
                Ok(InferenceResult {
                    typ: Type::Con(Symbol::intern("Int")),
                    effects: EffectSet::Empty,
                    constraints: Vec::new(),
                })
            }
            TypeCode::LiteralString => {
                Ok(InferenceResult {
                    typ: Type::Con(Symbol::intern("String")),
                    effects: EffectSet::Empty,
                    constraints: Vec::new(),
                })
            }
            TypeCode::LiteralBool => {
                Ok(InferenceResult {
                    typ: Type::Con(Symbol::intern("Bool")),
                    effects: EffectSet::Empty,
                    constraints: Vec::new(),
                })
            }
            TypeCode::LiteralUnit => {
                Ok(InferenceResult {
                    typ: Type::Con(Symbol::intern("Unit")),
                    effects: EffectSet::Empty,
                    constraints: Vec::new(),
                })
            }
            TypeCode::ExprVar => {
                // Decode variable name from payload
                let symbol = self.decode_symbol_from_payload(&node.payload)?;
                
                // Look up in type environment
                if let Some(scheme) = ctx.env.lookup_var(symbol) {
                    let scheme_clone = scheme.clone();
                    let (typ, effects) = ctx.instantiate(&scheme_clone);
                    Ok(InferenceResult {
                        typ,
                        effects,
                        constraints: Vec::new(),
                    })
                } else {
                    Err(Error::Type {
                        message: format!("Unbound variable: {}", symbol),
                    })
                }
            }
            TypeCode::ExprApp => {
                // For function application, we need to deserialize function and arguments
                // This would require more complex binary payload parsing
                self.infer_application_from_payload(ctx, &node.payload)
            }
            TypeCode::ExprLambda => {
                self.infer_lambda_from_payload(ctx, &node.payload)
            }
            _ => {
                // Fallback to generic inference
                Ok(InferenceResult {
                    typ: Type::Hole,
                    effects: EffectSet::Empty,
                    constraints: Vec::new(),
                })
            }
        }
    }
    
    /// Load type cache from binary metadata
    fn load_type_cache(&mut self, deserializer: &mut BinaryDeserializer) -> Result<()> {
        // Read type metadata section
        let type_count = deserializer.read_u32()?;
        
        for i in 0..type_count {
            let type_id = deserializer.read_u32()?;
            let typ = deserializer.deserialize_internal_type()?;
            self.type_cache.insert(type_id, typ);
        }
        
        // Read effect cache
        let effect_count = deserializer.read_u32()?;
        for i in 0..effect_count {
            let effect_id = deserializer.read_u32()?;
            let effect_set = deserializer.deserialize_effect_set()?;
            self.effect_cache.insert(effect_id, effect_set);
        }
        
        Ok(())
    }
    
    /// Validate a cached type against current context
    fn validate_type_in_context(&self, _typ: &Type, _effects: &EffectSet) -> Result<()> {
        // Simplified validation for now
        // TODO: Add proper type variable and effect validation
        Ok(())
    }
    
    /// Decode symbol from binary payload
    fn decode_symbol_from_payload(&self, payload: &[u8]) -> Result<Symbol> {
        if payload.len() < 4 {
            return Err(Error::Type {
                message: "Invalid symbol payload".to_string(),
            });
        }
        
        let symbol_id = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        // Look up symbol in table (would need access to deserializer's symbol table)
        Ok(Symbol::intern(&format!("sym_{}", symbol_id)))
    }
    
    /// Infer function application from binary payload
    fn infer_application_from_payload(&mut self, ctx: &mut InferenceContext, payload: &[u8]) -> Result<InferenceResult> {
        // Simplified implementation - would need proper payload parsing
        let result_type = ctx.fresh_type_var();
        Ok(InferenceResult {
            typ: result_type,
            effects: EffectSet::Empty,
            constraints: Vec::new(),
        })
    }
    
    /// Infer lambda expression from binary payload  
    fn infer_lambda_from_payload(&mut self, ctx: &mut InferenceContext, payload: &[u8]) -> Result<InferenceResult> {
        // Simplified implementation - would need proper payload parsing
        let param_type = ctx.fresh_type_var();
        let return_type = ctx.fresh_type_var();
        
        let lambda_type = Type::Fun {
            params: vec![param_type],
            return_type: Box::new(return_type),
            effects: EffectSet::Empty,
        };
        
        Ok(InferenceResult {
            typ: lambda_type,
            effects: EffectSet::Empty,
            constraints: Vec::new(),
        })
    }
}

/// Result of binary type checking
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    pub typed_nodes: Vec<TypedNode>,
    pub errors: Vec<TypeError>,
    pub validation_mode: ValidationMode,
    pub node_count: u32,
}

/// Individual typed node result
#[derive(Debug, Clone)]
pub struct TypedNode {
    pub node_id: u32,
    pub inferred_type: Type,
    pub effects: EffectSet,
    pub span: Span,
}

/// Type checking validation mode
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationMode {
    /// Used cached type information for validation
    CachedValidation,
    /// Performed full type inference
    FullInference,
    /// Incremental update from previous state
    IncrementalUpdate,
}

impl TypeCheckResult {
    /// Check if type checking succeeded (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
    
    /// Get the total number of type errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    /// Check if result used cached types (faster path)
    pub fn used_cache(&self) -> bool {
        matches!(self.validation_mode, ValidationMode::CachedValidation)
    }
    
    /// Format all errors as a string
    pub fn format_errors(&self) -> String {
        self.errors
            .iter()
            .enumerate()
            .map(|(i, error)| format!("Error {}: {}", i + 1, error.format_error()))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// Extension trait for BinaryDeserializer to support type checking
trait BinaryDeserializerExt {
    fn deserialize_binary_node(&mut self) -> Result<TypedBinaryNode>;
    fn deserialize_internal_type(&mut self) -> Result<Type>;
}

impl BinaryDeserializerExt for BinaryDeserializer {
    fn deserialize_binary_node(&mut self) -> Result<TypedBinaryNode> {
        // Read node type
        let node_type_byte = self.read_u8()?;
        let node_type = match node_type_byte {
            0x10 => TypeCode::LiteralInteger,
            0x11 => TypeCode::LiteralFloat,
            0x12 => TypeCode::LiteralString,
            0x13 => TypeCode::LiteralBool,
            0x14 => TypeCode::LiteralUnit,
            0x20 => TypeCode::ExprVar,
            0x21 => TypeCode::ExprApp,
            0x22 => TypeCode::ExprLambda,
            _ => return Err(Error::Type {
                message: format!("Unknown node type: {}", node_type_byte),
            }),
        };
        
        // Read optional type information
        let has_type = self.read_u8()? != 0;
        let inferred_type = if has_type {
            Some(self.deserialize_internal_type()?)
        } else {
            None
        };
        
        // Read optional effect information
        let has_effects = self.read_u8()? != 0;
        let effects = if has_effects {
            Some(EffectSet::Empty) // Simplified for now
        } else {
            None
        };
        
        // Read span
        let file_id = self.read_u32()?;
        let start = self.read_u32()?;
        let end = self.read_u32()?;
        let span = Span::new(
            crate::core::span::FileId::new(file_id),
            crate::core::span::ByteOffset::new(start),
            crate::core::span::ByteOffset::new(end),
        );
        
        // Read payload
        let payload_size = self.read_u32()? as usize;
        let mut payload = vec![0u8; payload_size];
        self.read_exact(&mut payload)?;
        
        Ok(TypedBinaryNode {
            node_type,
            inferred_type,
            effects,
            span,
            payload,
        })
    }
    
    
    fn deserialize_internal_type(&mut self) -> Result<Type> {
        let type_code = self.read_u8()?;
        match type_code {
            0x50 => { // InternalTypeVar
                let var_id = self.read_u32()?;
                Ok(Type::Var(TypeVar(var_id)))
            }
            0x51 => { // InternalTypeCon
                let symbol_id = self.read_u32()?;
                Ok(Type::Con(Symbol::intern(&format!("type_{}", symbol_id))))
            }
            0x54 => { // InternalTypeTuple
                let count = self.read_u32()?;
                let mut types = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    types.push(self.deserialize_internal_type()?);
                }
                Ok(Type::Tuple(types))
            }
            0x55 => { // InternalTypeRec
                let var_id = self.read_u32()?;
                let body = Box::new(self.deserialize_internal_type()?);
                Ok(Type::Rec {
                    var: TypeVar(var_id),
                    body,
                })
            }
            0x56 => { // InternalTypeHole
                Ok(Type::Hole)
            }
            _ => Err(Error::Type {
                message: format!("Unknown internal type code: {}", type_code),
            }),
        }
    }
    
}

/// Extension trait for reading exact bytes
trait ReadExact {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;
    fn read_u8(&mut self) -> Result<u8>;
}

impl ReadExact for BinaryDeserializer {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        for byte in buf.iter_mut() {
            *byte = BinaryDeserializer::read_u8(self)?;
        }
        Ok(())
    }
    
    fn read_u8(&mut self) -> Result<u8> {
        BinaryDeserializer::read_u8(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::symbol::Symbol;
    
    #[test]
    fn test_binary_type_checker_creation() {
        let checker = BinaryTypeChecker::new();
        assert_eq!(checker.error_reporter.error_count(), 0);
    }
    
    #[test]
    fn test_type_check_result() {
        let result = TypeCheckResult {
            typed_nodes: vec![],
            errors: vec![],
            validation_mode: ValidationMode::FullInference,
            node_count: 0,
        };
        
        assert!(result.is_success());
        assert_eq!(result.error_count(), 0);
        assert!(!result.used_cache());
    }
    
    #[test]
    fn test_typed_node() {
        let node = TypedNode {
            node_id: 1,
            inferred_type: Type::Con(Symbol::intern("Int")),
            effects: EffectSet::Empty,
            span: Span::new(crate::core::span::FileId::new(0), crate::core::span::ByteOffset::new(0), crate::core::span::ByteOffset::new(0)),
        };
        
        assert_eq!(node.node_id, 1);
        assert!(matches!(node.inferred_type, Type::Con(_)));
    }
}