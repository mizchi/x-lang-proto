//! Binary AST Type Checker
//! 
//! This module provides type checking directly on binary AST format,
//! enabling faster type analysis by avoiding AST reconstruction.

use x_parser::{
    binary::{BinaryDeserializer, TypeCode, TypedBinaryNode},
    Symbol,
    Span,
};
use crate::{
    types::*,
    inference::{InferenceContext, InferenceResult},
    error_reporting::{TypeError, TypeErrorReporter},
};
use std::result::Result as StdResult;
use std::collections::HashMap;

/// Binary type checker that operates directly on binary AST
#[derive(Debug)]
#[allow(dead_code)]
pub struct BinaryTypeChecker {
    type_env: TypeEnv,
    var_gen: VarGen,
    error_reporter: TypeErrorReporter,
    type_cache: HashMap<u32, Type>,
    effect_cache: HashMap<u32, EffectSet>,
    inference_cache: HashMap<u32, InferenceResult>,
}

impl BinaryTypeChecker {
    pub fn new() -> Self {
        BinaryTypeChecker {
            type_env: TypeEnv::new(),
            var_gen: VarGen::new(),
            error_reporter: TypeErrorReporter::new(),
            type_cache: HashMap::new(),
            effect_cache: HashMap::new(),
            inference_cache: HashMap::new(),
        }
    }
    
    /// Type check a binary compilation unit
    pub fn check_binary_compilation_unit(&mut self, binary_data: &[u8]) -> StdResult<TypeCheckResult, String> {
        let mut deserializer = BinaryDeserializer::new(binary_data.to_vec()).map_err(|e| format!("Binary deserializer error: {:?}", e))?;
        
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
    fn validate_cached_types(&mut self, deserializer: &mut BinaryDeserializer) -> StdResult<TypeCheckResult, String> {
        let mut results = Vec::new();
        let mut node_count = 0;
        
        // Load type metadata
        self.load_type_cache(deserializer)?;
        
        // Validate each typed node
        while let Ok((expr, typ, effects)) = deserializer.deserialize_typed_expr() {
            node_count += 1;
            
            if let (Some(_cached_type), Some(_cached_effects)) = (typ, effects) {
                // TODO: Convert x-parser types to x-checker types and validate
                // For now, skip validation
                
                results.push(TypedNode {
                    node_id: node_count,
                    inferred_type: Type::Hole, // TODO: Convert from cached_type
                    effects: EffectSet::Empty, // TODO: Convert from cached_effects
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
    fn infer_types_from_binary(&mut self, _deserializer: &mut BinaryDeserializer) -> StdResult<TypeCheckResult, String> {
        let results = Vec::new();
        let node_count = 0;
        
        // Create inference context
        let _inference_ctx = InferenceContext::new();
        
        // TODO: Implement proper binary node processing
        // For now, skip binary node processing since the deserialize_binary_node method is not available
        
        Ok(TypeCheckResult {
            typed_nodes: results,
            errors: self.error_reporter.errors().to_vec(),
            validation_mode: ValidationMode::FullInference,
            node_count,
        })
    }
    
    /// Infer type for a single binary node
    #[allow(dead_code)]
    fn infer_binary_node(&mut self, ctx: &mut InferenceContext, node: &TypedBinaryNode) -> StdResult<InferenceResult, String> {
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
                    Err(format!("Unbound variable: {}", symbol))
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
    fn load_type_cache(&mut self, deserializer: &mut BinaryDeserializer) -> StdResult<(), String> {
        // Read type metadata section
        let type_count = deserializer.read_u32().map_err(|e| format!("Failed to read type count: {:?}", e))?;
        
        for _i in 0..type_count {
            let type_id = deserializer.read_u32().map_err(|e| format!("Failed to read type id: {:?}", e))?;
            // TODO: Implement proper type deserialization once API is available
            // For now, store placeholder types
            self.type_cache.insert(type_id, Type::Hole);
        }
        
        // Read effect cache
        let effect_count = deserializer.read_u32().map_err(|e| format!("Failed to read effect count: {:?}", e))?;
        for _i in 0..effect_count {
            let effect_id = deserializer.read_u32().map_err(|e| format!("Failed to read effect id: {:?}", e))?;
            // TODO: Implement proper effect deserialization once API is available
            // For now, store empty effect sets
            self.effect_cache.insert(effect_id, EffectSet::Empty);
        }
        
        Ok(())
    }
    
    /// Validate a cached type against current context
    #[allow(dead_code)]
    fn validate_type_in_context(&self, _typ: &Type, _effects: &EffectSet) -> StdResult<(), String> {
        // Simplified validation for now
        // TODO: Add proper type variable and effect validation
        Ok(())
    }
    
    /// Decode symbol from binary payload
    #[allow(dead_code)]
    fn decode_symbol_from_payload(&self, payload: &[u8]) -> StdResult<Symbol, String> {
        if payload.len() < 4 {
            return Err("Invalid symbol payload".to_string());
        }
        
        let symbol_id = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        // Look up symbol in table (would need access to deserializer's symbol table)
        Ok(Symbol::intern(&format!("sym_{}", symbol_id)))
    }
    
    /// Infer function application from binary payload
    #[allow(dead_code)]
    fn infer_application_from_payload(&mut self, ctx: &mut InferenceContext, _payload: &[u8]) -> StdResult<InferenceResult, String> {
        // Simplified implementation - would need proper payload parsing
        let result_type = ctx.fresh_type_var();
        Ok(InferenceResult {
            typ: result_type,
            effects: EffectSet::Empty,
            constraints: Vec::new(),
        })
    }
    
    /// Infer lambda expression from binary payload  
    #[allow(dead_code)]
    fn infer_lambda_from_payload(&mut self, ctx: &mut InferenceContext, _payload: &[u8]) -> StdResult<InferenceResult, String> {
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
            .map(|(i, error)| format!("Error {}: {:?}", i + 1, error))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// Extension trait for BinaryDeserializer to support type checking
#[allow(dead_code)]
trait BinaryDeserializerExt {
    fn deserialize_binary_node(&mut self) -> StdResult<TypedBinaryNode, String>;
    fn deserialize_internal_type(&mut self) -> StdResult<Type, String>;
}

/*
// TODO: Implement proper binary deserialization for x-checker types
impl BinaryDeserializerExt for BinaryDeserializer {
    fn deserialize_binary_node(&mut self) -> StdResult<TypedBinaryNode, String> {
        // Read node type
        let node_type_byte = self.read_u8().map_err(|e| format!("Failed to read node type: {:?}", e))?;
        let node_type = match node_type_byte {
            0x10 => TypeCode::LiteralInteger,
            0x11 => TypeCode::LiteralFloat,
            0x12 => TypeCode::LiteralString,
            0x13 => TypeCode::LiteralBool,
            0x14 => TypeCode::LiteralUnit,
            0x20 => TypeCode::ExprVar,
            0x21 => TypeCode::ExprApp,
            0x22 => TypeCode::ExprLambda,
            _ => return Err(format!("Unknown node type: {}", node_type_byte)),
        };
        
        // Read optional type information
        let has_type = self.read_u8().map_err(|e| format!("Failed to read has_type: {:?}", e))? != 0;
        let inferred_type = if has_type {
            Some(self.deserialize_internal_type().map_err(|e| format!("Failed to deserialize type: {:?}", e))?)
        } else {
            None
        };
        
        // Read optional effect information
        let has_effects = self.read_u8().map_err(|e| format!("Failed to read has_effects: {:?}", e))? != 0;
        let effects = if has_effects {
            Some(EffectSet::Empty) // Simplified for now
        } else {
            None
        };
        
        // Read span
        let file_id = self.read_u32().map_err(|e| format!("Failed to read file_id: {:?}", e))?;
        let start = self.read_u32().map_err(|e| format!("Failed to read start: {:?}", e))?;
        let end = self.read_u32().map_err(|e| format!("Failed to read end: {:?}", e))?;
        let span = Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)); // Simplified for now
        
        // Read payload
        let payload_size = self.read_u32().map_err(|e| format!("Failed to read payload size: {:?}", e))? as usize;
        let mut payload = vec![0u8; payload_size];
        self.read_exact(&mut payload).map_err(|e| format!("Failed to read payload: {:?}", e))?;
        
        Ok(TypedBinaryNode {
            node_type,
            inferred_type,
            effects,
            span,
            payload,
        })
    }
    
    
    fn deserialize_internal_type(&mut self) -> StdResult<Type, String> {
        let type_code = self.read_u8().map_err(|e| format!("Failed to read type code: {:?}", e))?;
        match type_code {
            0x50 => { // InternalTypeVar
                let var_id = self.read_u32().map_err(|e| format!("Failed to read var_id: {:?}", e))?;
                Ok(Type::Var(TypeVar(var_id)))
            }
            0x51 => { // InternalTypeCon
                let symbol_id = self.read_u32().map_err(|e| format!("Failed to read symbol_id: {:?}", e))?;
                Ok(Type::Con(Symbol::intern(&format!("type_{}", symbol_id))))
            }
            0x54 => { // InternalTypeTuple
                let count = self.read_u32().map_err(|e| format!("Failed to read count: {:?}", e))?;
                let mut types = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    types.push(self.deserialize_internal_type().map_err(|e| format!("Failed to deserialize type: {:?}", e))?);
                }
                Ok(Type::Tuple(types))
            }
            0x55 => { // InternalTypeRec
                let var_id = self.read_u32().map_err(|e| format!("Failed to read rec var_id: {:?}", e))?;
                let body = Box::new(self.deserialize_internal_type().map_err(|e| format!("Failed to deserialize rec body: {:?}", e))?);
                Ok(Type::Rec {
                    var: TypeVar(var_id),
                    body,
                })
            }
            0x56 => { // InternalTypeHole
                Ok(Type::Hole)
            }
            _ => Err(format!("Unknown internal type code: {}", type_code)),
        }
    }
    
}
*/

/// Extension trait for reading exact bytes
#[allow(dead_code)]
trait ReadExact {
    fn read_exact(&mut self, buf: &mut [u8]) -> StdResult<(), String>;
    fn read_u8(&mut self) -> StdResult<u8, String>;
}

/*
impl ReadExact for BinaryDeserializer {
    fn read_exact(&mut self, buf: &mut [u8]) -> StdResult<(), String> {
        for byte in buf.iter_mut() {
            *byte = BinaryDeserializer::read_u8(self).map_err(|e| format!("Failed to read byte: {:?}", e))?;
        }
        Ok(())
    }
    
    fn read_u8(&mut self) -> StdResult<u8, String> {
        BinaryDeserializer::read_u8(self).map_err(|e| format!("Read error: {:?}", e))
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::symbol::Symbol;
    
    #[test]
    fn test_binary_type_checker_creation() {
        let checker = BinaryTypeChecker::new();
        assert!(!checker.error_reporter.has_errors());
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
            span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
        };
        
        assert_eq!(node.node_id, 1);
        assert!(matches!(node.inferred_type, Type::Con(_)));
    }
}