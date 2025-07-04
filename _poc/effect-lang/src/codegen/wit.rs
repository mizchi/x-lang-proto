use crate::core::ast::*;
use crate::core::symbol::Symbol;
use std::collections::HashMap;
use std::fmt::Write;

/// WebAssembly Interface Types (WIT) generator
pub struct WitGenerator {
    output: String,
    indent_level: usize,
}

impl WitGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn generate(&mut self, compilation_unit: &CompilationUnit) -> Result<String, String> {
        self.output.clear();
        
        // Generate package declaration
        if let Some(package_name) = &compilation_unit.package_name {
            writeln!(self.output, "package {};\n", package_name.as_str())
                .map_err(|e| format!("Failed to write package declaration: {}", e))?;
        }

        // Generate world declaration
        writeln!(self.output, "world effect-lang {{")
            .map_err(|e| format!("Failed to write world declaration: {}", e))?;
        self.indent_level += 1;

        // Process modules
        for module in &compilation_unit.modules {
            self.generate_module(module)?;
        }

        self.indent_level -= 1;
        writeln!(self.output, "}}")
            .map_err(|e| format!("Failed to close world declaration: {}", e))?;

        Ok(self.output.clone())
    }

    fn generate_module(&mut self, module: &Module) -> Result<(), String> {
        writeln!(self.output, "{}// Module: {}", self.indent(), module.name.as_str())
            .map_err(|e| format!("Failed to write module comment: {}", e))?;

        for item in &module.items {
            self.generate_item(item)?;
        }

        Ok(())
    }

    fn generate_item(&mut self, item: &Item) -> Result<(), String> {
        match item {
            Item::InterfaceDef(interface) => self.generate_interface_def(interface),
            Item::TypeDef(type_def) => self.generate_type_def(type_def),
            Item::ValueDef(value_def) => self.generate_value_def(value_def),
            Item::ImportDef(import_def) => self.generate_import_def(import_def),
            Item::ExportDef(export_def) => self.generate_export_def(export_def),
            _ => Ok(()), // Skip other items for now
        }
    }

    fn generate_interface_def(&mut self, interface: &ComponentInterface) -> Result<(), String> {
        // Generate interface based on visibility
        let visibility = self.visibility_to_wit_export(&interface.visibility);
        
        writeln!(self.output, "{}interface {} {{", self.indent(), interface.name.as_str())
            .map_err(|e| format!("Failed to write interface declaration: {}", e))?;
        self.indent_level += 1;

        // Generate interface items
        for item in &interface.items {
            self.generate_interface_item(item)?;
        }

        self.indent_level -= 1;
        writeln!(self.output, "{}}}", self.indent())
            .map_err(|e| format!("Failed to close interface: {}", e))?;

        // Generate export/import statements
        if let Some(export_stmt) = visibility {
            writeln!(self.output, "{}{}", self.indent(), export_stmt)
                .map_err(|e| format!("Failed to write export statement: {}", e))?;
        }

        writeln!(self.output, "")
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        Ok(())
    }

    fn generate_interface_item(&mut self, item: &InterfaceItem) -> Result<(), String> {
        match item {
            InterfaceItem::Function(func) => self.generate_function_signature(func),
            InterfaceItem::Type(type_name, wasm_type) => self.generate_wasm_type(type_name, wasm_type),
            InterfaceItem::Resource(resource) => self.generate_resource(resource),
        }
    }

    fn generate_function_signature(&mut self, func: &FunctionSignature) -> Result<(), String> {
        write!(self.output, "{}{}: func(", self.indent(), func.name.as_str())
            .map_err(|e| format!("Failed to write function signature: {}", e))?;

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ")
                    .map_err(|e| format!("Failed to write parameter separator: {}", e))?;
            }
            write!(self.output, "{}: {}", param.name.as_str(), self.wasm_type_to_wit(&param.wasm_type))
                .map_err(|e| format!("Failed to write parameter: {}", e))?;
        }

        write!(self.output, ")")
            .map_err(|e| format!("Failed to close parameter list: {}", e))?;

        // Return type
        if !func.results.is_empty() {
            write!(self.output, " -> ")
                .map_err(|e| format!("Failed to write return arrow: {}", e))?;

            if func.results.len() == 1 {
                write!(self.output, "{}", self.wasm_type_to_wit(&func.results[0]))
                    .map_err(|e| format!("Failed to write return type: {}", e))?;
            } else {
                write!(self.output, "(")
                    .map_err(|e| format!("Failed to open return tuple: {}", e))?;
                for (i, result) in func.results.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ", ")
                            .map_err(|e| format!("Failed to write result separator: {}", e))?;
                    }
                    write!(self.output, "{}", self.wasm_type_to_wit(result))
                        .map_err(|e| format!("Failed to write result type: {}", e))?;
                }
                write!(self.output, ")")
                    .map_err(|e| format!("Failed to close return tuple: {}", e))?;
            }
        }

        writeln!(self.output, "")
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        Ok(())
    }

    fn generate_wasm_type(&mut self, name: &Symbol, wasm_type: &WasmType) -> Result<(), String> {
        writeln!(self.output, "{}type {} = {};", self.indent(), name.as_str(), self.wasm_type_to_wit(wasm_type))
            .map_err(|e| format!("Failed to write type definition: {}", e))?;
        Ok(())
    }

    fn generate_resource(&mut self, resource: &ResourceDefinition) -> Result<(), String> {
        writeln!(self.output, "{}resource {} {{", self.indent(), resource.name.as_str())
            .map_err(|e| format!("Failed to write resource declaration: {}", e))?;
        self.indent_level += 1;

        // Generate constructor
        if let Some(constructor) = &resource.constructor {
            self.generate_resource_method("constructor", constructor)?;
        }

        // Generate methods
        for method in &resource.methods {
            self.generate_resource_method(&method.name.as_str(), method)?;
        }

        self.indent_level -= 1;
        writeln!(self.output, "{}}}", self.indent())
            .map_err(|e| format!("Failed to close resource: {}", e))?;

        Ok(())
    }

    fn generate_resource_method(&mut self, name: &str, method: &ResourceMethod) -> Result<(), String> {
        let method_type = match method.method_type {
            MethodType::Constructor => "constructor",
            MethodType::Method => "",
            MethodType::Static => "static",
        };

        write!(self.output, "{}{} {}: func(", self.indent(), method_type, name)
            .map_err(|e| format!("Failed to write resource method: {}", e))?;

        // Parameters
        for (i, param) in method.params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ")
                    .map_err(|e| format!("Failed to write parameter separator: {}", e))?;
            }
            write!(self.output, "{}: {}", param.name.as_str(), self.wasm_type_to_wit(&param.wasm_type))
                .map_err(|e| format!("Failed to write parameter: {}", e))?;
        }

        write!(self.output, ")")
            .map_err(|e| format!("Failed to close parameter list: {}", e))?;

        // Return type
        if !method.results.is_empty() {
            write!(self.output, " -> ")
                .map_err(|e| format!("Failed to write return arrow: {}", e))?;

            if method.results.len() == 1 {
                write!(self.output, "{}", self.wasm_type_to_wit(&method.results[0]))
                    .map_err(|e| format!("Failed to write return type: {}", e))?;
            } else {
                write!(self.output, "(")
                    .map_err(|e| format!("Failed to open return tuple: {}", e))?;
                for (i, result) in method.results.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ", ")
                            .map_err(|e| format!("Failed to write result separator: {}", e))?;
                    }
                    write!(self.output, "{}", self.wasm_type_to_wit(result))
                        .map_err(|e| format!("Failed to write result type: {}", e))?;
                }
                write!(self.output, ")")
                    .map_err(|e| format!("Failed to close return tuple: {}", e))?;
            }
        }

        writeln!(self.output, "")
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        Ok(())
    }

    fn generate_type_def(&mut self, type_def: &TypeDef) -> Result<(), String> {
        // Generate WIT type definition for complex types
        writeln!(self.output, "{}// Type: {}", self.indent(), type_def.name.as_str())
            .map_err(|e| format!("Failed to write type comment: {}", e))?;
        
        // Convert Effect-lang type to WIT type
        match &type_def.definition {
            TypeDefinition::Record(fields) => {
                writeln!(self.output, "{}record {} {{", self.indent(), type_def.name.as_str())
                    .map_err(|e| format!("Failed to write record declaration: {}", e))?;
                self.indent_level += 1;

                for (field_name, field_type) in fields {
                    writeln!(self.output, "{}{}: {},", self.indent(), field_name.as_str(), self.type_to_wit(field_type))
                        .map_err(|e| format!("Failed to write record field: {}", e))?;
                }

                self.indent_level -= 1;
                writeln!(self.output, "{}}}", self.indent())
                    .map_err(|e| format!("Failed to close record: {}", e))?;
            }
            TypeDefinition::Variant(variants) => {
                writeln!(self.output, "{}variant {} {{", self.indent(), type_def.name.as_str())
                    .map_err(|e| format!("Failed to write variant declaration: {}", e))?;
                self.indent_level += 1;

                for variant in variants {
                    match &variant.data {
                        Some(data) => {
                            writeln!(self.output, "{}{}({}),", self.indent(), variant.name.as_str(), self.type_to_wit(data))
                                .map_err(|e| format!("Failed to write variant with data: {}", e))?;
                        }
                        None => {
                            writeln!(self.output, "{}{},", self.indent(), variant.name.as_str())
                                .map_err(|e| format!("Failed to write variant without data: {}", e))?;
                        }
                    }
                }

                self.indent_level -= 1;
                writeln!(self.output, "{}}}", self.indent())
                    .map_err(|e| format!("Failed to close variant: {}", e))?;
            }
            _ => {
                // For other types, generate a simple type alias
                writeln!(self.output, "{}type {} = {};", self.indent(), type_def.name.as_str(), self.type_to_wit(&type_def.definition.to_type()))
                    .map_err(|e| format!("Failed to write type alias: {}", e))?;
            }
        }

        writeln!(self.output, "")
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        Ok(())
    }

    fn generate_value_def(&mut self, value_def: &ValueDef) -> Result<(), String> {
        // Generate export for public functions
        if self.is_public_visibility(&value_def.visibility) {
            writeln!(self.output, "{}export {}: func() -> {};", 
                self.indent(), 
                value_def.name.as_str(), 
                self.type_to_wit(&value_def.type_annotation.as_ref().unwrap_or(&Type::Unknown)))
                .map_err(|e| format!("Failed to write value export: {}", e))?;
        }

        Ok(())
    }

    fn generate_import_def(&mut self, import_def: &ImportDef) -> Result<(), String> {
        match &import_def.kind {
            ImportKind::Interface => {
                writeln!(self.output, "{}import {}: interface;", self.indent(), import_def.name.as_str())
                    .map_err(|e| format!("Failed to write interface import: {}", e))?;
            }
            ImportKind::Core => {
                writeln!(self.output, "{}import {}: core;", self.indent(), import_def.name.as_str())
                    .map_err(|e| format!("Failed to write core import: {}", e))?;
            }
            ImportKind::Func => {
                writeln!(self.output, "{}import {}: func;", self.indent(), import_def.name.as_str())
                    .map_err(|e| format!("Failed to write func import: {}", e))?;
            }
            ImportKind::Value => {
                writeln!(self.output, "{}import {}: value;", self.indent(), import_def.name.as_str())
                    .map_err(|e| format!("Failed to write value import: {}", e))?;
            }
        }

        Ok(())
    }

    fn generate_export_def(&mut self, export_def: &ExportDef) -> Result<(), String> {
        writeln!(self.output, "{}export {}: {};", self.indent(), export_def.name.as_str(), export_def.exported_name.as_str())
            .map_err(|e| format!("Failed to write export: {}", e))?;

        Ok(())
    }

    fn wasm_type_to_wit(&self, wasm_type: &WasmType) -> &'static str {
        match wasm_type {
            WasmType::I32 => "s32",
            WasmType::I64 => "s64",
            WasmType::F32 => "f32",
            WasmType::F64 => "f64",
            WasmType::V128 => "v128",
            WasmType::FuncRef => "funcref",
            WasmType::ExternRef => "externref",
            WasmType::String => "string",
            WasmType::Bool => "bool",
            WasmType::List => "list",
            WasmType::Record => "record",
            WasmType::Variant => "variant",
            WasmType::Tuple => "tuple",
            WasmType::Option => "option",
            WasmType::Result => "result",
        }
    }

    fn type_to_wit(&self, type_expr: &Type) -> String {
        match type_expr {
            Type::Unit => "()".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int => "s32".to_string(),
            Type::Float => "f64".to_string(),
            Type::String => "string".to_string(),
            Type::Char => "char".to_string(),
            Type::List(inner) => format!("list<{}>", self.type_to_wit(inner)),
            Type::Option(inner) => format!("option<{}>", self.type_to_wit(inner)),
            Type::Result(ok, err) => format!("result<{}, {}>", self.type_to_wit(ok), self.type_to_wit(err)),
            Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.type_to_wit(t)).collect();
                format!("tuple<{}>", type_strs.join(", "))
            }
            Type::Function(params, return_type) => {
                let param_strs: Vec<String> = params.iter().map(|t| self.type_to_wit(t)).collect();
                format!("func({}) -> {}", param_strs.join(", "), self.type_to_wit(return_type))
            }
            Type::Named(name) => name.as_str().to_string(),
            Type::Variable(var) => format!("var{}", var),
            Type::Unknown => "unknown".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn visibility_to_wit_export(&self, visibility: &Visibility) -> Option<String> {
        match visibility {
            Visibility::Component { export: true, interface: Some(interface_name), .. } => {
                Some(format!("export {}: interface;", interface_name.as_str()))
            }
            Visibility::Component { export: true, interface: None, .. } => {
                Some("export: interface;".to_string())
            }
            _ => None,
        }
    }

    fn is_public_visibility(&self, visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public | Visibility::Component { export: true, .. })
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }
}

// Helper trait to convert TypeDefinition to Type
trait ToType {
    fn to_type(&self) -> Type;
}

impl ToType for TypeDefinition {
    fn to_type(&self) -> Type {
        match self {
            TypeDefinition::Alias(t) => t.clone(),
            TypeDefinition::Record(_) => Type::Named(Symbol::new("record")),
            TypeDefinition::Variant(_) => Type::Named(Symbol::new("variant")),
            TypeDefinition::Enum(_) => Type::Named(Symbol::new("enum")),
            TypeDefinition::Tuple(types) => Type::Tuple(types.clone()),
            TypeDefinition::Function(params, return_type) => Type::Function(params.clone(), Box::new(return_type.as_ref().clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::symbol::Symbol;

    #[test]
    fn test_basic_wit_generation() {
        let mut generator = WitGenerator::new();
        let compilation_unit = CompilationUnit {
            package_name: Some(Symbol::new("test:package")),
            modules: vec![],
            imports: vec![],
            exports: vec![],
        };

        let result = generator.generate(&compilation_unit).unwrap();
        assert!(result.contains("package test:package;"));
        assert!(result.contains("world effect-lang {"));
    }

    #[test]
    fn test_wasm_type_conversion() {
        let generator = WitGenerator::new();
        assert_eq!(generator.wasm_type_to_wit(&WasmType::I32), "s32");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::String), "string");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::Bool), "bool");
    }
}