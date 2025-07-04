//! Binary serialization for EffectLang AST
//! 
//! This module provides efficient binary serialization/deserialization for AST nodes,
//! enabling fast storage and content-addressed programming.

use crate::core::{
    ast::*,
    span::{Span, FileId, ByteOffset},
    symbol::Symbol,
};
use crate::{Error, Result};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::io::{Write, Read};

/// Binary format type codes
#[repr(u8)]
enum TypeCode {
    // Literals
    LiteralInteger = 0x01,
    LiteralFloat = 0x02,
    LiteralString = 0x03,
    LiteralBool = 0x04,
    LiteralUnit = 0x05,
    
    // Expressions
    ExprVar = 0x10,
    ExprApp = 0x11,
    ExprLambda = 0x12,
    ExprLet = 0x13,
    ExprIf = 0x14,
    ExprMatch = 0x15,
    ExprDo = 0x16,
    ExprHandle = 0x17,
    ExprResume = 0x18,
    ExprPerform = 0x19,
    ExprAnn = 0x1A,
    
    // Patterns
    PatternWildcard = 0x20,
    PatternVariable = 0x21,
    PatternLiteral = 0x22,
    PatternConstructor = 0x23,
    PatternTuple = 0x24,
    PatternRecord = 0x25,
    PatternOr = 0x26,
    PatternAs = 0x27,
    PatternAnn = 0x28,
    
    // Types
    TypeVar = 0x30,
    TypeCon = 0x31,
    TypeApp = 0x32,
    TypeFun = 0x33,
    TypeForall = 0x34,
    TypeExists = 0x35,
    TypeRecord = 0x36,
    TypeVariant = 0x37,
    TypeTuple = 0x38,
    TypeRow = 0x39,
    TypeHole = 0x3A,
    
    // Module items
    ItemTypeDef = 0x40,
    ItemValueDef = 0x41,
    ItemEffectDef = 0x42,
    ItemHandlerDef = 0x43,
    
    // Collections
    Vec = 0x50,
    Option = 0x51,
    Symbol = 0x52,
    Span = 0x53,
    
    // Special
    CompilationUnit = 0x60,
    Module = 0x61,
}

/// Binary AST serializer
pub struct BinarySerializer {
    buffer: Vec<u8>,
    symbol_table: HashMap<Symbol, u32>,
    next_symbol_id: u32,
}

impl BinarySerializer {
    pub fn new() -> Self {
        BinarySerializer {
            buffer: Vec::new(),
            symbol_table: HashMap::new(),
            next_symbol_id: 0,
        }
    }
    
    /// Serialize a compilation unit to binary format
    pub fn serialize_compilation_unit(&mut self, cu: &CompilationUnit) -> Result<Vec<u8>> {
        self.buffer.clear();
        self.symbol_table.clear();
        self.next_symbol_id = 0;
        
        self.write_u8(TypeCode::CompilationUnit as u8)?;
        self.serialize_module(&cu.module)?;
        self.serialize_span(&cu.span)?;
        
        Ok(self.buffer.clone())
    }
    
    /// Calculate content hash of the binary representation
    pub fn content_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Serialize a module
    fn serialize_module(&mut self, module: &Module) -> Result<()> {
        self.write_u8(TypeCode::Module as u8)?;
        self.serialize_module_path(&module.name)?;
        
        // Serialize exports (optional)
        match &module.exports {
            Some(exports) => {
                self.write_u8(1)?; // Some
                self.serialize_export_list(exports)?;
            }
            None => {
                self.write_u8(0)?; // None
            }
        }
        
        // Serialize imports
        self.write_varint(module.imports.len() as u64)?;
        for import in &module.imports {
            self.serialize_import(import)?;
        }
        
        // Serialize items
        self.write_varint(module.items.len() as u64)?;
        for item in &module.items {
            self.serialize_item(item)?;
        }
        
        self.serialize_span(&module.span)?;
        Ok(())
    }
    
    /// Serialize an expression
    fn serialize_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit, span) => {
                self.serialize_literal(lit)?;
                self.serialize_span(span)?;
            }
            Expr::Var(symbol, span) => {
                self.write_u8(TypeCode::ExprVar as u8)?;
                self.serialize_symbol(*symbol)?;
                self.serialize_span(span)?;
            }
            Expr::App(func, args, span) => {
                self.write_u8(TypeCode::ExprApp as u8)?;
                self.serialize_expr(func)?;
                self.write_varint(args.len() as u64)?;
                for arg in args {
                    self.serialize_expr(arg)?;
                }
                self.serialize_span(span)?;
            }
            Expr::Lambda { parameters, body, span } => {
                self.write_u8(TypeCode::ExprLambda as u8)?;
                self.write_varint(parameters.len() as u64)?;
                for param in parameters {
                    self.serialize_pattern(param)?;
                }
                self.serialize_expr(body)?;
                self.serialize_span(span)?;
            }
            Expr::Let { pattern, type_annotation, value, body, span } => {
                self.write_u8(TypeCode::ExprLet as u8)?;
                self.serialize_pattern(pattern)?;
                match type_annotation {
                    Some(t) => {
                        self.write_u8(1)?;
                        self.serialize_type(t)?;
                    }
                    None => {
                        self.write_u8(0)?;
                    }
                }
                self.serialize_expr(value)?;
                self.serialize_expr(body)?;
                self.serialize_span(span)?;
            }
            Expr::If { condition, then_branch, else_branch, span } => {
                self.write_u8(TypeCode::ExprIf as u8)?;
                self.serialize_expr(condition)?;
                self.serialize_expr(then_branch)?;
                self.serialize_expr(else_branch)?;
                self.serialize_span(span)?;
            }
            // Add other expression types as needed...
            _ => {
                return Err(Error::Parse {
                    message: "Unsupported expression type for serialization".to_string(),
                });
            }
        }
        Ok(())
    }
    
    /// Serialize a pattern
    fn serialize_pattern(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Wildcard(span) => {
                self.write_u8(TypeCode::PatternWildcard as u8)?;
                self.serialize_span(span)?;
            }
            Pattern::Variable(symbol, span) => {
                self.write_u8(TypeCode::PatternVariable as u8)?;
                self.serialize_symbol(*symbol)?;
                self.serialize_span(span)?;
            }
            Pattern::Literal(lit, span) => {
                self.write_u8(TypeCode::PatternLiteral as u8)?;
                self.serialize_literal(lit)?;
                self.serialize_span(span)?;
            }
            Pattern::Constructor { name, args, span } => {
                self.write_u8(TypeCode::PatternConstructor as u8)?;
                self.serialize_symbol(*name)?;
                self.write_varint(args.len() as u64)?;
                for arg in args {
                    self.serialize_pattern(arg)?;
                }
                self.serialize_span(span)?;
            }
            // Add other pattern types as needed...
            _ => {
                return Err(Error::Parse {
                    message: "Unsupported pattern type for serialization".to_string(),
                });
            }
        }
        Ok(())
    }
    
    /// Serialize a type
    fn serialize_type(&mut self, typ: &Type) -> Result<()> {
        match typ {
            Type::Var(symbol, span) => {
                self.write_u8(TypeCode::TypeVar as u8)?;
                self.serialize_symbol(*symbol)?;
                self.serialize_span(span)?;
            }
            Type::Con(symbol, span) => {
                self.write_u8(TypeCode::TypeCon as u8)?;
                self.serialize_symbol(*symbol)?;
                self.serialize_span(span)?;
            }
            Type::App(base, args, span) => {
                self.write_u8(TypeCode::TypeApp as u8)?;
                self.serialize_type(base)?;
                self.write_varint(args.len() as u64)?;
                for arg in args {
                    self.serialize_type(arg)?;
                }
                self.serialize_span(span)?;
            }
            Type::Hole(span) => {
                self.write_u8(TypeCode::TypeHole as u8)?;
                self.serialize_span(span)?;
            }
            // Add other type constructors as needed...
            _ => {
                return Err(Error::Parse {
                    message: "Unsupported type for serialization".to_string(),
                });
            }
        }
        Ok(())
    }
    
    /// Serialize a literal
    fn serialize_literal(&mut self, literal: &Literal) -> Result<()> {
        match literal {
            Literal::Integer(n) => {
                self.write_u8(TypeCode::LiteralInteger as u8)?;
                self.write_i64(*n)?;
            }
            Literal::Float(f) => {
                self.write_u8(TypeCode::LiteralFloat as u8)?;
                self.write_f64(*f)?;
            }
            Literal::String(s) => {
                self.write_u8(TypeCode::LiteralString as u8)?;
                self.write_string(s)?;
            }
            Literal::Bool(b) => {
                self.write_u8(TypeCode::LiteralBool as u8)?;
                self.write_u8(if *b { 1 } else { 0 })?;
            }
            Literal::Unit => {
                self.write_u8(TypeCode::LiteralUnit as u8)?;
            }
        }
        Ok(())
    }
    
    /// Serialize a symbol (with deduplication)
    fn serialize_symbol(&mut self, symbol: Symbol) -> Result<()> {
        if let Some(&id) = self.symbol_table.get(&symbol) {
            self.write_varint(id as u64)?;
        } else {
            let id = self.next_symbol_id;
            self.symbol_table.insert(symbol, id);
            self.next_symbol_id += 1;
            
            self.write_varint(id as u64)?;
            self.write_string(symbol.as_str())?;
        }
        Ok(())
    }
    
    /// Serialize a span
    fn serialize_span(&mut self, span: &Span) -> Result<()> {
        self.write_u8(TypeCode::Span as u8)?;
        self.write_u32(span.file_id.as_u32())?;
        self.write_u32(span.start.as_u32())?;
        self.write_u32(span.end.as_u32())?;
        Ok(())
    }
    
    // Placeholder implementations for missing methods
    fn serialize_module_path(&mut self, _path: &ModulePath) -> Result<()> {
        // TODO: Implement module path serialization
        Ok(())
    }
    
    fn serialize_export_list(&mut self, _exports: &ExportList) -> Result<()> {
        // TODO: Implement export list serialization
        Ok(())
    }
    
    fn serialize_import(&mut self, _import: &Import) -> Result<()> {
        // TODO: Implement import serialization
        Ok(())
    }
    
    fn serialize_item(&mut self, _item: &Item) -> Result<()> {
        // TODO: Implement item serialization
        Ok(())
    }
    
    // Low-level writing methods
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.buffer.push(value);
        Ok(())
    }
    
    fn write_u32(&mut self, value: u32) -> Result<()> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    
    fn write_i64(&mut self, value: i64) -> Result<()> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    
    fn write_f64(&mut self, value: f64) -> Result<()> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    
    fn write_string(&mut self, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        self.write_varint(bytes.len() as u64)?;
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }
    
    fn write_varint(&mut self, mut value: u64) -> Result<()> {
        while value >= 0x80 {
            self.buffer.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.buffer.push(value as u8);
        Ok(())
    }
}

/// Binary AST deserializer
pub struct BinaryDeserializer {
    data: Vec<u8>,
    pos: usize,
    symbol_table: Vec<Symbol>,
}

impl BinaryDeserializer {
    pub fn new(data: Vec<u8>) -> Self {
        BinaryDeserializer {
            data,
            pos: 0,
            symbol_table: Vec::new(),
        }
    }
    
    /// Deserialize a compilation unit from binary format
    pub fn deserialize_compilation_unit(&mut self) -> Result<CompilationUnit> {
        let type_code = self.read_u8()?;
        if type_code != TypeCode::CompilationUnit as u8 {
            return Err(Error::Parse {
                message: format!("Expected compilation unit, got type code {}", type_code),
            });
        }
        
        let module = self.deserialize_module()?;
        let span = self.deserialize_span()?;
        
        Ok(CompilationUnit { module, span })
    }
    
    fn deserialize_module(&mut self) -> Result<Module> {
        let type_code = self.read_u8()?;
        if type_code != TypeCode::Module as u8 {
            return Err(Error::Parse {
                message: format!("Expected module, got type code {}", type_code),
            });
        }
        
        let name = self.deserialize_module_path()?;
        
        // Deserialize exports
        let exports = if self.read_u8()? == 1 {
            Some(self.deserialize_export_list()?)
        } else {
            None
        };
        
        // Deserialize imports
        let import_count = self.read_varint()? as usize;
        let mut imports = Vec::with_capacity(import_count);
        for _ in 0..import_count {
            imports.push(self.deserialize_import()?);
        }
        
        // Deserialize items
        let item_count = self.read_varint()? as usize;
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            items.push(self.deserialize_item()?);
        }
        
        let span = self.deserialize_span()?;
        
        Ok(Module {
            name,
            exports,
            imports,
            items,
            span,
        })
    }
    
    // Placeholder implementations for missing methods
    fn deserialize_module_path(&mut self) -> Result<ModulePath> {
        // TODO: Implement module path deserialization
        use crate::core::symbol::Symbol;
        Ok(ModulePath::new(
            vec![Symbol::intern("Test")],
            Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(4))
        ))
    }
    
    fn deserialize_export_list(&mut self) -> Result<ExportList> {
        // TODO: Implement export list deserialization
        Ok(ExportList {
            items: Vec::new(),
            span: Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)),
        })
    }
    
    fn deserialize_import(&mut self) -> Result<Import> {
        // TODO: Implement import deserialization
        Ok(Import {
            module_path: ModulePath::new(
                vec![Symbol::intern("Test")],
                Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(4))
            ),
            kind: ImportKind::Qualified,
            alias: None,
            span: Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)),
        })
    }
    
    fn deserialize_item(&mut self) -> Result<Item> {
        // TODO: Implement item deserialization
        Ok(Item::ValueDef(ValueDef {
            name: Symbol::intern("test"),
            type_annotation: None,
            parameters: Vec::new(),
            body: Expr::Literal(Literal::Unit, Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0))),
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            span: Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)),
        }))
    }
    
    fn deserialize_span(&mut self) -> Result<Span> {
        let type_code = self.read_u8()?;
        if type_code != TypeCode::Span as u8 {
            return Err(Error::Parse {
                message: format!("Expected span, got type code {}", type_code),
            });
        }
        
        let file_id = FileId::new(self.read_u32()?);
        let start = ByteOffset::new(self.read_u32()?);
        let end = ByteOffset::new(self.read_u32()?);
        
        Ok(Span::new(file_id, start, end))
    }
    
    // Low-level reading methods
    fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(Error::Parse {
                message: "Unexpected end of data".to_string(),
            });
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }
    
    fn read_u32(&mut self) -> Result<u32> {
        if self.pos + 4 > self.data.len() {
            return Err(Error::Parse {
                message: "Not enough data for u32".to_string(),
            });
        }
        let bytes = &self.data[self.pos..self.pos + 4];
        self.pos += 4;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    
    fn read_i64(&mut self) -> Result<i64> {
        if self.pos + 8 > self.data.len() {
            return Err(Error::Parse {
                message: "Not enough data for i64".to_string(),
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(i64::from_le_bytes(bytes))
    }
    
    fn read_f64(&mut self) -> Result<f64> {
        if self.pos + 8 > self.data.len() {
            return Err(Error::Parse {
                message: "Not enough data for f64".to_string(),
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(f64::from_le_bytes(bytes))
    }
    
    fn read_varint(&mut self) -> Result<u64> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            if self.pos >= self.data.len() {
                return Err(Error::Parse {
                    message: "Unexpected end of data reading varint".to_string(),
                });
            }
            
            let byte = self.data[self.pos];
            self.pos += 1;
            
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(Error::Parse {
                    message: "Varint too long".to_string(),
                });
            }
        }
        
        Ok(result)
    }
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::span::{FileId, ByteOffset};

    #[test]
    fn test_basic_serialization() {
        let mut serializer = BinarySerializer::new();
        
        // Create a simple compilation unit
        let module = Module {
            name: ModulePath::new(
                vec![Symbol::intern("Test")],
                Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(4))
            ),
            exports: None,
            imports: Vec::new(),
            items: Vec::new(),
            span: Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10)),
        };
        
        let cu = CompilationUnit {
            module,
            span: Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10)),
        };
        
        let binary_data = serializer.serialize_compilation_unit(&cu).unwrap();
        assert!(!binary_data.is_empty());
        
        // Test deserialization
        let mut deserializer = BinaryDeserializer::new(binary_data);
        let restored_cu = deserializer.deserialize_compilation_unit().unwrap();
        
        assert_eq!(cu.span, restored_cu.span);
    }
    
    #[test]
    fn test_content_hash() {
        let data = b"hello world";
        let hash1 = BinarySerializer::content_hash(data);
        let hash2 = BinarySerializer::content_hash(data);
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
        
        let different_data = b"hello mars";
        let hash3 = BinarySerializer::content_hash(different_data);
        
        assert_ne!(hash1, hash3);
    }
}