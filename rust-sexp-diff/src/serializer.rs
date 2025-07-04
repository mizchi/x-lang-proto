//! Binary serialization and deserialization for S-expressions
//! 
//! Provides compact binary format with varint encoding for efficiency

use crate::sexp::{SExp, Atom};
use crate::error::{SExpError, Result};
use std::io::{Write, Read, Cursor};

// Binary format constants
const ATOM_STRING: u8 = 0x01;
const ATOM_INTEGER: u8 = 0x02;
const ATOM_FLOAT: u8 = 0x03;
const ATOM_BOOLEAN: u8 = 0x04;
const SYMBOL: u8 = 0x05;
const LIST: u8 = 0x06;

/// Binary serializer for S-expressions
pub struct BinarySerializer;

impl BinarySerializer {
    /// Create a new binary serializer
    pub fn new() -> Self {
        BinarySerializer
    }

    /// Serialize an S-expression to binary format
    pub fn serialize(&self, sexp: &SExp) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.serialize_sexp(sexp, &mut buffer)?;
        Ok(buffer)
    }

    fn serialize_sexp(&self, sexp: &SExp, buffer: &mut Vec<u8>) -> Result<()> {
        match sexp {
            SExp::Atom(atom) => self.serialize_atom(atom, buffer),
            SExp::Symbol(symbol) => self.serialize_symbol(symbol, buffer),
            SExp::List(elements) => self.serialize_list(elements, buffer),
        }
    }

    fn serialize_atom(&self, atom: &Atom, buffer: &mut Vec<u8>) -> Result<()> {
        match atom {
            Atom::String(s) => {
                buffer.push(ATOM_STRING);
                self.serialize_string(s, buffer)?;
            }
            Atom::Integer(i) => {
                buffer.push(ATOM_INTEGER);
                self.serialize_varint(*i as u64, buffer)?;
            }
            Atom::Float(f) => {
                buffer.push(ATOM_FLOAT);
                buffer.extend_from_slice(&f.to_le_bytes());
            }
            Atom::Boolean(b) => {
                buffer.push(ATOM_BOOLEAN);
                buffer.push(if *b { 1 } else { 0 });
            }
        }
        Ok(())
    }

    fn serialize_symbol(&self, symbol: &str, buffer: &mut Vec<u8>) -> Result<()> {
        buffer.push(SYMBOL);
        self.serialize_string(symbol, buffer)
    }

    fn serialize_list(&self, elements: &[SExp], buffer: &mut Vec<u8>) -> Result<()> {
        buffer.push(LIST);
        self.serialize_varint(elements.len() as u64, buffer)?;
        
        for element in elements {
            self.serialize_sexp(element, buffer)?;
        }
        
        Ok(())
    }

    fn serialize_string(&self, s: &str, buffer: &mut Vec<u8>) -> Result<()> {
        let bytes = s.as_bytes();
        self.serialize_varint(bytes.len() as u64, buffer)?;
        buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn serialize_varint(&self, mut value: u64, buffer: &mut Vec<u8>) -> Result<()> {
        while value >= 0x80 {
            buffer.push((value as u8) | 0x80);
            value >>= 7;
        }
        buffer.push(value as u8);
        Ok(())
    }
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary deserializer for S-expressions
pub struct BinaryDeserializer<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BinaryDeserializer<'a> {
    /// Create a new binary deserializer
    pub fn new(data: &'a [u8]) -> Self {
        BinaryDeserializer { data, position: 0 }
    }

    /// Deserialize an S-expression from binary format
    pub fn deserialize(&mut self) -> Result<SExp> {
        if self.position >= self.data.len() {
            return Err(SExpError::DeserializationError {
                message: "Unexpected end of data".to_string(),
            });
        }

        let type_byte = self.read_byte()?;
        
        match type_byte {
            ATOM_STRING => {
                let s = self.read_string()?;
                Ok(SExp::Atom(Atom::String(s)))
            }
            ATOM_INTEGER => {
                let i = self.read_varint()? as i64;
                Ok(SExp::Atom(Atom::Integer(i)))
            }
            ATOM_FLOAT => {
                let f = self.read_float()?;
                Ok(SExp::Atom(Atom::Float(f)))
            }
            ATOM_BOOLEAN => {
                let b = self.read_byte()? != 0;
                Ok(SExp::Atom(Atom::Boolean(b)))
            }
            SYMBOL => {
                let symbol = self.read_string()?;
                Ok(SExp::Symbol(symbol))
            }
            LIST => {
                let len = self.read_varint()? as usize;
                let mut elements = Vec::with_capacity(len);
                
                for _ in 0..len {
                    elements.push(self.deserialize()?);
                }
                
                Ok(SExp::List(elements))
            }
            _ => Err(SExpError::DeserializationError {
                message: format!("Unknown type byte: 0x{:02x}", type_byte),
            }),
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        if self.position >= self.data.len() {
            return Err(SExpError::DeserializationError {
                message: "Unexpected end of data".to_string(),
            });
        }
        
        let byte = self.data[self.position];
        self.position += 1;
        Ok(byte)
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_varint()? as usize;
        
        if self.position + len > self.data.len() {
            return Err(SExpError::DeserializationError {
                message: "String length exceeds available data".to_string(),
            });
        }
        
        let bytes = &self.data[self.position..self.position + len];
        self.position += len;
        
        String::from_utf8(bytes.to_vec()).map_err(|e| SExpError::DeserializationError {
            message: format!("Invalid UTF-8 string: {}", e),
        })
    }

    fn read_varint(&mut self) -> Result<u64> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            if shift >= 64 {
                return Err(SExpError::DeserializationError {
                    message: "Varint too large".to_string(),
                });
            }
            
            let byte = self.read_byte()?;
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
        }
        
        Ok(result)
    }

    fn read_float(&mut self) -> Result<f64> {
        if self.position + 8 > self.data.len() {
            return Err(SExpError::DeserializationError {
                message: "Not enough data for float".to_string(),
            });
        }
        
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.position..self.position + 8]);
        self.position += 8;
        
        Ok(f64::from_le_bytes(bytes))
    }
}

/// Convenience function to serialize an S-expression
pub fn serialize(sexp: &SExp) -> Result<Vec<u8>> {
    BinarySerializer::new().serialize(sexp)
}

/// Convenience function to deserialize an S-expression
pub fn deserialize(data: &[u8]) -> Result<SExp> {
    BinaryDeserializer::new(data).deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_atom_integer() {
        let sexp = SExp::Atom(Atom::Integer(42));
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_serialize_atom_float() {
        let sexp = SExp::Atom(Atom::Float(3.14));
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_serialize_atom_string() {
        let sexp = SExp::Atom(Atom::String("hello".to_string()));
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_serialize_atom_boolean() {
        let sexp = SExp::Atom(Atom::Boolean(true));
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_serialize_symbol() {
        let sexp = SExp::Symbol("factorial".to_string());
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_serialize_list() {
        let sexp = SExp::List(vec![
            SExp::Symbol("+".to_string()),
            SExp::Atom(Atom::Integer(1)),
            SExp::Atom(Atom::Integer(2)),
        ]);
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_serialize_nested_list() {
        let sexp = SExp::List(vec![
            SExp::Symbol("if".to_string()),
            SExp::List(vec![
                SExp::Symbol("=".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::Atom(Atom::Integer(0)),
            ]),
            SExp::Atom(Atom::Integer(1)),
            SExp::List(vec![
                SExp::Symbol("*".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::List(vec![
                    SExp::Symbol("factorial".to_string()),
                    SExp::List(vec![
                        SExp::Symbol("-".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::Atom(Atom::Integer(1)),
                    ]),
                ]),
            ]),
        ]);
        
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_varint_encoding() {
        let serializer = BinarySerializer::new();
        
        // Test small numbers
        let mut buffer = Vec::new();
        serializer.serialize_varint(127, &mut buffer).unwrap();
        assert_eq!(buffer, vec![127]);
        
        // Test larger numbers
        buffer.clear();
        serializer.serialize_varint(128, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x80, 0x01]);
        
        // Test reading back
        let mut deserializer = BinaryDeserializer::new(&buffer);
        assert_eq!(deserializer.read_varint().unwrap(), 128);
    }

    #[test]
    fn test_empty_list() {
        let sexp = SExp::List(vec![]);
        let serialized = serialize(&sexp).unwrap();
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(sexp, deserialized);
    }

    #[test]
    fn test_round_trip_performance() {
        let complex_sexp = SExp::List(vec![
            SExp::Symbol("module".to_string()),
            SExp::Symbol("math".to_string()),
            SExp::List(vec![
                SExp::Symbol("export".to_string()),
                SExp::Symbol("factorial".to_string()),
                SExp::Symbol("fibonacci".to_string()),
            ]),
            SExp::List(vec![
                SExp::Symbol("defun".to_string()),
                SExp::Symbol("factorial".to_string()),
                SExp::List(vec![SExp::Symbol("n".to_string())]),
                SExp::List(vec![
                    SExp::Symbol("if".to_string()),
                    SExp::List(vec![
                        SExp::Symbol("=".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::Atom(Atom::Integer(0)),
                    ]),
                    SExp::Atom(Atom::Integer(1)),
                    SExp::List(vec![
                        SExp::Symbol("*".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::List(vec![
                            SExp::Symbol("factorial".to_string()),
                            SExp::List(vec![
                                SExp::Symbol("-".to_string()),
                                SExp::Symbol("n".to_string()),
                                SExp::Atom(Atom::Integer(1)),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]);
        
        // Serialize and deserialize multiple times
        for _ in 0..1000 {
            let serialized = serialize(&complex_sexp).unwrap();
            let deserialized = deserialize(&serialized).unwrap();
            assert_eq!(complex_sexp, deserialized);
        }
    }
}