//! Enhanced binary serialization for x Language AST with type checking support
//! 
//! This module provides efficient binary serialization/deserialization for AST nodes,
//! optimized for type checking and LSP scenarios with embedded type information.
//!
//! ## Magic Number
//! 
//! x Language binary files start with a 4-byte magic number: `\0xlg` (0x00786C67)
//! This follows the WebAssembly pattern where the magic number includes readable text.

use crate::{
    ast::*,
    span::{Span, FileId, ByteOffset},
    symbol::Symbol,
    error::{ParseError as Error, Result},
};

// Temporary type definitions for binary serialization
// These should be properly integrated with the type system in the future
#[derive(Debug, Clone)]
pub struct InternalType;

#[derive(Debug, Clone)]
pub struct EffectVar(u32);

#[derive(Debug, Clone)]
pub struct Effect {
    pub name: Symbol,
    pub operations: Vec<()>,
}

#[derive(Debug, Clone)]
pub enum EffectSet {
    Empty,
    Var(EffectVar),
    Row { effects: Vec<Effect>, tail: Option<Box<EffectSet>> },
}
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::io::{Write, Read};

/// Magic number for x Language binary format: '\0xlg' (0x00786C67)
pub const MAGIC_NUMBER: [u8; 4] = [0x00, 0x78, 0x6C, 0x67];

/// Current version of the binary format
pub const FORMAT_VERSION: u32 = 1;

/// Enhanced binary format type codes with type checking support
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TypeCode {
    // Header and metadata
    Header = 0x00,
    TypedNode = 0x01,
    
    // Literals
    LiteralInteger = 0x10,
    LiteralFloat = 0x11,
    LiteralString = 0x12,
    LiteralBool = 0x13,
    LiteralUnit = 0x14,
    
    // Expressions
    ExprVar = 0x20,
    ExprApp = 0x21,
    ExprLambda = 0x22,
    ExprLet = 0x23,
    ExprIf = 0x24,
    ExprMatch = 0x25,
    ExprDo = 0x26,
    ExprHandle = 0x27,
    ExprResume = 0x28,
    ExprPerform = 0x29,
    ExprAnn = 0x2A,
    
    // Patterns
    PatternWildcard = 0x30,
    PatternVariable = 0x31,
    PatternLiteral = 0x32,
    PatternConstructor = 0x33,
    PatternTuple = 0x34,
    PatternRecord = 0x35,
    PatternOr = 0x36,
    PatternAs = 0x37,
    PatternAnn = 0x38,
    
    // AST Types
    AstTypeVar = 0x40,
    AstTypeCon = 0x41,
    AstTypeApp = 0x42,
    AstTypeFun = 0x43,
    AstTypeForall = 0x44,
    AstTypeExists = 0x45,
    AstTypeRecord = 0x46,
    AstTypeVariant = 0x47,
    AstTypeTuple = 0x48,
    AstTypeRow = 0x49,
    AstTypeHole = 0x4A,
    
    // Internal Types (analysis types)
    InternalTypeVar = 0x50,
    InternalTypeCon = 0x51,
    InternalTypeApp = 0x52,
    InternalTypeFun = 0x53,
    InternalTypeTuple = 0x54,
    InternalTypeRec = 0x55,
    InternalTypeHole = 0x56,
    InternalTypeVariant = 0x57,
    
    // Effect System
    EffectSetEmpty = 0x60,
    EffectSetVar = 0x61,
    EffectSetRow = 0x62,
    Effect = 0x63,
    TypeScheme = 0x64,
    
    // Module items
    ItemTypeDef = 0x70,
    ItemValueDef = 0x71,
    ItemEffectDef = 0x72,
    ItemHandlerDef = 0x73,
    
    // Collections
    Vec = 0x80,
    Option = 0x81,
    Symbol = 0x82,
    Span = 0x83,
    HashMap = 0x84,
    
    // Special
    CompilationUnit = 0x90,
    Module = 0x91,
    TypeMetadata = 0x92,
    CachedInference = 0x93,
}

/// Binary format header with type checking capabilities
#[derive(Debug, Clone)]
pub struct BinaryHeader {
    pub magic: [u8; 4],           // "EFFL" magic bytes
    pub version: u16,             // Format version
    pub flags: BinaryFlags,       // Feature flags
    pub symbol_table_size: u32,   // Number of symbols
    pub type_metadata_offset: u32, // Offset to type metadata
    pub inference_cache_offset: u32, // Offset to cached inference results
    pub checksum: u32,            // Content checksum
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct BinaryFlags: u16 {
        const TYPE_CHECKED = 0x0001;     // AST includes type information
        const COMPRESSED = 0x0002;       // Content is compressed
        const INCREMENTAL = 0x0004;      // Supports incremental updates
        const CACHED_EFFECTS = 0x0008;   // Pre-computed effect analysis
        const SYMBOL_INTERNED = 0x0010;  // Symbols are interned
        const RECURSIVE_TYPES = 0x0020;  // Contains recursive types
        const POLYMORPHIC = 0x0040;      // Contains polymorphic types
    }
}

impl BinaryHeader {
    pub const MAGIC: [u8; 4] = *b"EFFL";
    pub const VERSION: u16 = 3;  // Enhanced version with type checking
    
    pub fn new() -> Self {
        BinaryHeader {
            magic: Self::MAGIC,
            version: Self::VERSION,
            flags: BinaryFlags::SYMBOL_INTERNED,
            symbol_table_size: 0,
            type_metadata_offset: 0,
            inference_cache_offset: 0,
            checksum: 0,
        }
    }
    
    pub fn with_type_checking() -> Self {
        let mut header = Self::new();
        header.flags |= BinaryFlags::TYPE_CHECKED | BinaryFlags::CACHED_EFFECTS;
        header
    }
}

/// Type-aware node in binary format
#[derive(Debug, Clone)]
pub struct TypedBinaryNode {
    pub node_type: TypeCode,
    pub inferred_type: Option<InternalType>,
    pub effects: Option<EffectSet>,
    pub span: Span,
    pub payload: Vec<u8>,
}

/// Enhanced binary AST serializer with type checking support
pub struct BinarySerializer {
    buffer: Vec<u8>,
    symbol_table: HashMap<Symbol, u32>,
    type_cache: HashMap<InternalType, u32>,
    effect_cache: HashMap<EffectSet, u32>,
    next_symbol_id: u32,
    next_type_id: u32,
    next_effect_id: u32,
    header: BinaryHeader,
    type_metadata: Vec<u8>,
    inference_cache: Vec<u8>,
}

impl BinarySerializer {
    pub fn new() -> Self {
        BinarySerializer {
            buffer: Vec::new(),
            symbol_table: HashMap::new(),
            type_cache: HashMap::new(),
            effect_cache: HashMap::new(),
            next_symbol_id: 0,
            next_type_id: 0,
            next_effect_id: 0,
            header: BinaryHeader::new(),
            type_metadata: Vec::new(),
            inference_cache: Vec::new(),
        }
    }
    
    pub fn with_type_checking() -> Self {
        let mut serializer = Self::new();
        serializer.header = BinaryHeader::with_type_checking();
        serializer
    }
    
    /// Serialize a compilation unit to binary format
    pub fn serialize_compilation_unit(&mut self, cu: &CompilationUnit) -> Result<Vec<u8>> {
        self.buffer.clear();
        self.symbol_table.clear();
        self.next_symbol_id = 0;
        
        // Write magic number
        self.buffer.extend_from_slice(&MAGIC_NUMBER);
        
        // Write format version
        self.write_u32(FORMAT_VERSION)?;
        
        // Write header
        self.serialize_header()?;
        
        // Write actual content
        self.write_u8(TypeCode::CompilationUnit as u8)?;
        self.serialize_module(&cu.module)?;
        self.serialize_span(&cu.span)?;
        
        Ok(self.buffer.clone())
    }
    
    /// Serialize binary header
    fn serialize_header(&mut self) -> Result<()> {
        // Flags
        self.write_u32(self.header.flags.bits() as u32)?;
        
        // Placeholder for symbol table offset (will be filled later)
        self.write_u32(0)?;
        
        // Placeholder for type metadata offset (will be filled later) 
        self.write_u32(0)?;
        
        // Placeholder for inference cache offset (will be filled later)
        self.write_u32(0)?;
        
        Ok(())
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
                match lit {
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
                self.write_u8(TypeCode::AstTypeVar as u8)?;
                self.serialize_symbol(*symbol)?;
                self.serialize_span(span)?;
            }
            Type::Con(symbol, span) => {
                self.write_u8(TypeCode::AstTypeCon as u8)?;
                self.serialize_symbol(*symbol)?;
                self.serialize_span(span)?;
            }
            Type::App(base, args, span) => {
                self.write_u8(TypeCode::AstTypeApp as u8)?;
                self.serialize_type(base)?;
                self.write_varint(args.len() as u64)?;
                for arg in args {
                    self.serialize_type(arg)?;
                }
                self.serialize_span(span)?;
            }
            Type::Fun { params, return_type, effects, span } => {
                self.write_u8(TypeCode::AstTypeFun as u8)?;
                // Serialize parameter types
                self.write_varint(params.len() as u64)?;
                for param in params {
                    self.serialize_type(param)?;
                }
                // Serialize return type
                self.serialize_type(return_type)?;
                // Serialize effects
                self.serialize_effect_set(effects)?;
                self.serialize_span(span)?;
            }
            Type::Hole(span) => {
                self.write_u8(TypeCode::AstTypeHole as u8)?;
                self.serialize_span(span)?;
            }
            // Add other type constructors as needed...
            _ => {
                return Err(Error::Parse {
                    message: format!("Unsupported type for serialization: {:?}", typ),
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
    
    /// Serialize an effect set
    fn serialize_effect_set(&mut self, effects: &crate::ast::EffectSet) -> Result<()> {
        // Serialize effect list
        self.write_varint(effects.effects.len() as u64)?;
        for effect in &effects.effects {
            self.serialize_symbol(effect.name)?;
            self.write_varint(effect.args.len() as u64)?;
            for arg in &effect.args {
                self.serialize_type(arg)?;
            }
            self.serialize_span(&effect.span)?;
        }
        
        // Serialize row variable
        match &effects.row_var {
            Some(var) => {
                self.write_u8(1)?; // Some
                self.serialize_symbol(*var)?;
            }
            None => {
                self.write_u8(0)?; // None
            }
        }
        
        self.serialize_span(&effects.span)?;
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
    
    fn serialize_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::ValueDef(value_def) => {
                self.write_u8(TypeCode::ItemValueDef as u8)?;
                self.serialize_symbol(value_def.name)?;
                
                // Serialize type annotation
                match &value_def.type_annotation {
                    Some(type_ann) => {
                        self.write_u8(1)?; // Some
                        self.serialize_type(type_ann)?;
                    }
                    None => {
                        self.write_u8(0)?; // None
                    }
                }
                
                // Serialize parameters
                self.write_varint(value_def.parameters.len() as u64)?;
                for param in &value_def.parameters {
                    self.serialize_pattern(param)?;
                }
                
                // Serialize body
                self.serialize_expr(&value_def.body)?;
                
                // Serialize visibility and purity
                self.write_u8(match value_def.visibility {
                    Visibility::Public => 0,
                    Visibility::Private => 1,
                    Visibility::Crate => 2,
                    Visibility::Package => 3,
                    Visibility::Super => 4,
                    Visibility::InPath(_) => 5,
                    Visibility::SelfModule => 6,
                    Visibility::Component { .. } => 7,
                })?;
                
                self.write_u8(match value_def.purity {
                    Purity::Pure => 0,
                    Purity::Impure => 1,
                    Purity::Inferred => 2,
                })?;
                
                self.serialize_span(&value_def.span)?;
            }
            Item::TypeDef(_) => {
                self.write_u8(TypeCode::ItemTypeDef as u8)?;
                // TODO: Implement type definition serialization
            }
            Item::EffectDef(_) => {
                self.write_u8(TypeCode::ItemEffectDef as u8)?;
                // TODO: Implement effect definition serialization
            }
            Item::HandlerDef(_) => {
                self.write_u8(TypeCode::ItemHandlerDef as u8)?;
                // TODO: Implement handler definition serialization
            }
            Item::ModuleTypeDef(_) => {
                self.write_u8(TypeCode::ItemTypeDef as u8)?; 
                // TODO: Implement module type definition serialization
            }
            Item::InterfaceDef(_) => {
                self.write_u8(TypeCode::ItemTypeDef as u8)?;
                // TODO: Implement interface definition serialization 
            }
        }
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

/// Enhanced binary AST deserializer with type checking support
pub struct BinaryDeserializer {
    data: Vec<u8>,
    pos: usize,
    symbol_table: Vec<Symbol>,
    header: Option<BinaryHeader>,
    type_cache: Vec<InternalType>,
    effect_cache: Vec<EffectSet>,
}

impl BinaryDeserializer {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut deserializer = BinaryDeserializer {
            data,
            pos: 0,
            symbol_table: Vec::new(),
            header: None,
            type_cache: Vec::new(),
            effect_cache: Vec::new(),
        };
        
        // Check minimum file size for magic number + version
        if deserializer.data.len() < 8 {
            return Err(Error::Parse {
                message: "File too short to be a valid x Language binary file".to_string(),
            });
        }
        
        // Check magic number
        let magic_bytes = &deserializer.data[0..4];
        if magic_bytes != MAGIC_NUMBER {
            return Err(Error::Parse {
                message: "Invalid magic number. This is not a valid x Language binary file".to_string(),
            });
        }
        
        // Read format version
        let version = u32::from_le_bytes([
            deserializer.data[4], 
            deserializer.data[5], 
            deserializer.data[6], 
            deserializer.data[7]
        ]);
        
        if version != FORMAT_VERSION {
            return Err(Error::Parse {
                message: format!("Unsupported format version: {}. Expected: {}", version, FORMAT_VERSION),
            });
        }
        
        // Move position past magic number and version
        deserializer.pos = 8;
        
        // Try to read header if present
        if let Ok(header) = deserializer.read_header() {
            deserializer.header = Some(header);
        }
        
        Ok(deserializer)
    }
    
    pub fn has_type_information(&self) -> bool {
        self.header
            .as_ref()
            .map(|h| h.flags.contains(BinaryFlags::TYPE_CHECKED))
            .unwrap_or(false)
    }
    
    pub fn has_cached_effects(&self) -> bool {
        self.header
            .as_ref()
            .map(|h| h.flags.contains(BinaryFlags::CACHED_EFFECTS))
            .unwrap_or(false)
    }
    
    /// Deserialize with type information if available
    pub fn deserialize_typed_expr(&mut self) -> Result<(Expr, Option<InternalType>, Option<EffectSet>)> {
        // Read type information if available
        let typ = if self.has_type_information() {
            let type_id = self.read_u32()?;
            if type_id == 0xFFFFFFFF {
                None
            } else {
                self.type_cache.get(type_id as usize).cloned()
            }
        } else {
            None
        };
        
        let effects = if self.has_cached_effects() {
            Some(self.deserialize_internal_effect_set()?)
        } else {
            None
        };
        
        // For now, return a placeholder expression
        let expr = Expr::Literal(
            Literal::Unit,
            Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)),
        );
        
        Ok((expr, typ, effects))
    }
    
    fn read_header(&mut self) -> Result<BinaryHeader> {
        // Magic number and version already read and validated in new()
        // Header starts at position 8
        
        if self.data.len() < self.pos + 16 {
            return Err(Error::Parse {
                message: "File too short to contain complete header".to_string(),
            });
        }
        
        let flags = self.read_u32()?;
        let symbol_table_offset = self.read_u32()?;
        let type_metadata_offset = self.read_u32()?;
        let inference_cache_offset = self.read_u32()?;
        
        Ok(BinaryHeader {
            magic: MAGIC_NUMBER,
            version: 1, // Already validated
            flags: BinaryFlags::from_bits_truncate(flags as u16),
            symbol_table_size: symbol_table_offset,
            type_metadata_offset,
            inference_cache_offset,
            checksum: 0, // TODO: Implement checksum if needed
        })
    }
    
    pub fn deserialize_internal_effect_set(&mut self) -> Result<EffectSet> {
        let effect_code = self.read_u8()?;
        match effect_code {
            0x60 => Ok(EffectSet::Empty), // EffectSetEmpty
            0x61 => {
                let var_id = self.read_u32()?;
                Ok(EffectSet::Var(EffectVar(var_id)))
            }
            0x62 => { // EffectSetRow
                let count = self.read_u32()?;
                let mut effects = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let effect_name = Symbol::intern(&format!("effect_{}", self.read_u32()?));
                    effects.push(Effect {
                        name: effect_name,
                        operations: Vec::new(), // Simplified
                    });
                }
                let has_tail = self.read_u8()? != 0;
                let tail = if has_tail {
                    Some(Box::new(self.deserialize_internal_effect_set()?))
                } else {
                    None
                };
                Ok(EffectSet::Row { effects, tail })
            }
            _ => Err(Error::Parse {
                message: format!("Unknown effect set code: {}", effect_code),
            }),
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
        use crate::symbol::Symbol;
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
        let type_code = self.read_u8()?;
        match type_code {
            code if code == TypeCode::ItemValueDef as u8 => {
                let name = self.deserialize_symbol()?;
                
                // Deserialize type annotation
                let type_annotation = if self.read_u8()? == 1 {
                    Some(self.deserialize_type()?)
                } else {
                    None
                };
                
                // Deserialize parameters
                let param_count = self.read_varint()? as usize;
                let mut parameters = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    parameters.push(self.deserialize_pattern()?);
                }
                
                // Deserialize body
                let body = self.deserialize_expr()?;
                
                // Deserialize visibility and purity
                let visibility = match self.read_u8()? {
                    0 => Visibility::Public,
                    1 => Visibility::Private,
                    2 => Visibility::Crate,
                    3 => Visibility::Package,
                    4 => Visibility::Super,
                    5 => Visibility::InPath(ModulePath::new(vec![], Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)))),
                    6 => Visibility::SelfModule,
                    7 => Visibility::Component { export: false, import: false, interface: None },
                    _ => Visibility::Public, // default
                };
                
                let purity = match self.read_u8()? {
                    0 => Purity::Pure,
                    1 => Purity::Impure,
                    2 => Purity::Inferred,
                    _ => Purity::Inferred, // default
                };
                
                let span = self.deserialize_span()?;
                
                Ok(Item::ValueDef(ValueDef {
                    name,
                    type_annotation,
                    parameters,
                    body,
                    visibility,
                    purity,
                    span,
                }))
            }
            code if code == TypeCode::ItemTypeDef as u8 => {
                // TODO: Implement type definition deserialization
                Ok(Item::ValueDef(ValueDef {
                    name: Symbol::intern("placeholder"),
                    type_annotation: None,
                    parameters: Vec::new(),
                    body: Expr::Literal(Literal::Unit, Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0))),
                    visibility: Visibility::Public,
                    purity: Purity::Inferred,
                    span: Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)),
                }))
            }
            _ => Err(Error::Parse {
                message: format!("Unknown item type code: {}", type_code),
            }),
        }
    }
    
    fn deserialize_symbol(&mut self) -> Result<Symbol> {
        let id = self.read_varint()?;
        if id < self.symbol_table.len() as u64 {
            Ok(self.symbol_table[id as usize])
        } else {
            // New symbol - read string
            let string_len = self.read_varint()? as usize;
            if self.pos + string_len > self.data.len() {
                return Err(Error::Parse {
                    message: "Not enough data for string".to_string(),
                });
            }
            
            let string_bytes = &self.data[self.pos..self.pos + string_len];
            self.pos += string_len;
            
            let symbol_str = std::str::from_utf8(string_bytes)
                .map_err(|_| Error::Parse {
                    message: "Invalid UTF-8 in symbol".to_string(),
                })?;
            
            let symbol = Symbol::intern(symbol_str);
            self.symbol_table.push(symbol);
            Ok(symbol)
        }
    }
    
    fn deserialize_expr(&mut self) -> Result<Expr> {
        let type_code = self.read_u8()?;
        match type_code {
            code if code == TypeCode::ExprVar as u8 => {
                let symbol = self.deserialize_symbol()?;
                let span = self.deserialize_span()?;
                Ok(Expr::Var(symbol, span))
            }
            code if code == TypeCode::ExprApp as u8 => {
                let func = Box::new(self.deserialize_expr()?);
                let arg_count = self.read_varint()? as usize;
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.deserialize_expr()?);
                }
                let span = self.deserialize_span()?;
                Ok(Expr::App(func, args, span))
            }
            code if code == TypeCode::ExprLambda as u8 => {
                let param_count = self.read_varint()? as usize;
                let mut parameters = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    parameters.push(self.deserialize_pattern()?);
                }
                let body = Box::new(self.deserialize_expr()?);
                let span = self.deserialize_span()?;
                Ok(Expr::Lambda { parameters, body, span })
            }
            code if code == TypeCode::ExprLet as u8 => {
                let pattern = self.deserialize_pattern()?;
                let type_annotation = if self.read_u8()? == 1 {
                    Some(self.deserialize_type()?)
                } else {
                    None
                };
                let value = Box::new(self.deserialize_expr()?);
                let body = Box::new(self.deserialize_expr()?);
                let span = self.deserialize_span()?;
                Ok(Expr::Let { pattern, type_annotation, value, body, span })
            }
            code if code == TypeCode::LiteralInteger as u8 => {
                let value = self.read_i64()?;
                let span = self.deserialize_span()?;
                Ok(Expr::Literal(Literal::Integer(value), span))
            }
            code if code == TypeCode::LiteralString as u8 => {
                let string_len = self.read_varint()? as usize;
                if self.pos + string_len > self.data.len() {
                    return Err(Error::Parse {
                        message: "Not enough data for string literal".to_string(),
                    });
                }
                
                let string_bytes = &self.data[self.pos..self.pos + string_len];
                self.pos += string_len;
                
                let string_value = std::str::from_utf8(string_bytes)
                    .map_err(|_| Error::Parse {
                        message: "Invalid UTF-8 in string literal".to_string(),
                    })?
                    .to_string();
                
                let span = self.deserialize_span()?;
                Ok(Expr::Literal(Literal::String(string_value), span))
            }
            code if code == TypeCode::LiteralBool as u8 => {
                let value = self.read_u8()? == 1;
                let span = self.deserialize_span()?;
                Ok(Expr::Literal(Literal::Bool(value), span))
            }
            code if code == TypeCode::LiteralFloat as u8 => {
                let value = self.read_f64()?;
                let span = self.deserialize_span()?;
                Ok(Expr::Literal(Literal::Float(value), span))
            }
            code if code == TypeCode::LiteralUnit as u8 => {
                let span = self.deserialize_span()?;
                Ok(Expr::Literal(Literal::Unit, span))
            }
            _ => Err(Error::Parse {
                message: format!("Unknown expression type code: {}", type_code),
            }),
        }
    }
    
    fn deserialize_pattern(&mut self) -> Result<Pattern> {
        let type_code = self.read_u8()?;
        match type_code {
            code if code == TypeCode::PatternVariable as u8 => {
                let symbol = self.deserialize_symbol()?;
                let span = self.deserialize_span()?;
                Ok(Pattern::Variable(symbol, span))
            }
            code if code == TypeCode::PatternWildcard as u8 => {
                let span = self.deserialize_span()?;
                Ok(Pattern::Wildcard(span))
            }
            code if code == TypeCode::PatternLiteral as u8 => {
                // Read the literal
                let literal_type = self.read_u8()?;
                let literal = match literal_type {
                    code if code == TypeCode::LiteralInteger as u8 => {
                        Literal::Integer(self.read_i64()?)
                    }
                    code if code == TypeCode::LiteralString as u8 => {
                        let string_len = self.read_varint()? as usize;
                        if self.pos + string_len > self.data.len() {
                            return Err(Error::Parse {
                                message: "Not enough data for string".to_string(),
                            });
                        }
                        
                        let string_bytes = &self.data[self.pos..self.pos + string_len];
                        self.pos += string_len;
                        
                        let string_value = std::str::from_utf8(string_bytes)
                            .map_err(|_| Error::Parse {
                                message: "Invalid UTF-8 in string".to_string(),
                            })?;
                        
                        Literal::String(string_value.to_string())
                    }
                    code if code == TypeCode::LiteralUnit as u8 => Literal::Unit,
                    _ => return Err(Error::Parse {
                        message: format!("Unknown literal type in pattern: {}", literal_type),
                    }),
                };
                
                let span = self.deserialize_span()?;
                Ok(Pattern::Literal(literal, span))
            }
            _ => Err(Error::Parse {
                message: format!("Unknown pattern type code: {}", type_code),
            }),
        }
    }
    
    fn deserialize_type(&mut self) -> Result<Type> {
        let type_code = self.read_u8()?;
        match type_code {
            code if code == TypeCode::AstTypeVar as u8 => {
                let symbol = self.deserialize_symbol()?;
                let span = self.deserialize_span()?;
                Ok(Type::Var(symbol, span))
            }
            code if code == TypeCode::AstTypeCon as u8 => {
                let symbol = self.deserialize_symbol()?;
                let span = self.deserialize_span()?;
                Ok(Type::Con(symbol, span))
            }
            code if code == TypeCode::AstTypeFun as u8 => {
                // Deserialize parameter types
                let param_count = self.read_varint()? as usize;
                let mut params = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    params.push(self.deserialize_type()?);
                }
                
                // Deserialize return type
                let return_type = Box::new(self.deserialize_type()?);
                
                // Deserialize effects
                let effects = self.deserialize_effect_set()?;
                
                let span = self.deserialize_span()?;
                Ok(Type::Fun { 
                    params, 
                    return_type, 
                    effects, 
                    span 
                })
            }
            code if code == TypeCode::AstTypeHole as u8 => {
                let span = self.deserialize_span()?;
                Ok(Type::Hole(span))
            }
            _ => Err(Error::Parse {
                message: format!("Unknown type code: {}", type_code),
            }),
        }
    }
    
    fn deserialize_effect_set(&mut self) -> Result<crate::ast::EffectSet> {
        // Deserialize effect list
        let effect_count = self.read_varint()? as usize;
        let mut effects = Vec::with_capacity(effect_count);
        for _ in 0..effect_count {
            let name = self.deserialize_symbol()?;
            let arg_count = self.read_varint()? as usize;
            let mut args = Vec::with_capacity(arg_count);
            for _ in 0..arg_count {
                args.push(self.deserialize_type()?);
            }
            let span = self.deserialize_span()?;
            effects.push(crate::ast::EffectRef { name, args, span });
        }
        
        // Deserialize row variable
        let row_var = if self.read_u8()? == 1 {
            Some(self.deserialize_symbol()?)
        } else {
            None
        };
        
        let span = self.deserialize_span()?;
        Ok(crate::ast::EffectSet {
            effects,
            row_var,
            span,
        })
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
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(Error::Parse {
                message: "Unexpected end of data".to_string(),
            });
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }
    
    pub fn read_u32(&mut self) -> Result<u32> {
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
    use crate::span::{FileId, ByteOffset};

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
        let mut deserializer = BinaryDeserializer::new(binary_data).unwrap();
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