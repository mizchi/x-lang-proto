//! Enhanced binary serialization for EffectLang AST with type checking support
//! 
//! This module provides efficient binary serialization/deserialization for AST nodes,
//! optimized for type checking and LSP scenarios with embedded type information.

use crate::core::{
    ast::*,
    span::{Span, FileId, ByteOffset},
    symbol::Symbol,
};
use crate::analysis::types::{Type as InternalType, EffectSet, EffectVar, Effect};
use crate::{Error, Result};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::io::{Write, Read};

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
            Type::Hole(span) => {
                self.write_u8(TypeCode::AstTypeHole as u8)?;
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
        
        // Try to read header if present
        if deserializer.data.len() >= 16 {
            if let Ok(header) = deserializer.read_header() {
                deserializer.header = Some(header);
            }
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
    pub fn deserialize_typed_expr(&mut self) -> Result<(crate::core::ast::Expr, Option<InternalType>, Option<EffectSet>)> {
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
            Some(self.deserialize_effect_set()?)
        } else {
            None
        };
        
        // For now, return a placeholder expression
        let expr = crate::core::ast::Expr::Literal(
            crate::core::ast::Literal::Unit,
            Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)),
        );
        
        Ok((expr, typ, effects))
    }
    
    fn read_header(&mut self) -> Result<BinaryHeader> {
        let magic_bytes = &self.data[0..4];
        let mut magic = [0u8; 4];
        magic.copy_from_slice(magic_bytes);
        
        let version = u16::from_le_bytes([self.data[4], self.data[5]]);
        let flags = BinaryFlags::from_bits_truncate(u16::from_le_bytes([self.data[6], self.data[7]]));
        let symbol_table_size = u32::from_le_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]);
        let type_metadata_offset = u32::from_le_bytes([self.data[12], self.data[13], self.data[14], self.data[15]]);
        let inference_cache_offset = if self.data.len() >= 20 {
            u32::from_le_bytes([self.data[16], self.data[17], self.data[18], self.data[19]])
        } else {
            0
        };
        let checksum = if self.data.len() >= 24 {
            u32::from_le_bytes([self.data[20], self.data[21], self.data[22], self.data[23]])
        } else {
            0
        };
        
        self.pos = 24; // Skip header
        
        Ok(BinaryHeader {
            magic,
            version,
            flags,
            symbol_table_size,
            type_metadata_offset,
            inference_cache_offset,
            checksum,
        })
    }
    
    pub fn deserialize_effect_set(&mut self) -> Result<EffectSet> {
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
                    Some(Box::new(self.deserialize_effect_set()?))
                } else {
                    None
                };
                Ok(EffectSet::Row { effects, tail })
            }
            _ => Err(Error::Type {
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