use x_parser::{CompilationUnit, Module, Item, TypeDef, TypeDefKind, ValueDef, Symbol, Type, Visibility, WasmType, ComponentInterface, InterfaceItem, FunctionSignature, ResourceMethod, span::{Span, FileId, ByteOffset}};
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
        
        // Generate package declaration from module name
        let package_name = compilation_unit.module.name.to_string();
        writeln!(self.output, "package {};\n", package_name)
            .map_err(|e| format!("Failed to write package declaration: {}", e))?;

        // Generate world declaration
        writeln!(self.output, "world effect-lang {{")
            .map_err(|e| format!("Failed to write world declaration: {}", e))?;
        self.indent_level += 1;

        // Process the module
        self.generate_module(&compilation_unit.module)?;

        self.indent_level -= 1;
        writeln!(self.output, "}}")
            .map_err(|e| format!("Failed to close world declaration: {}", e))?;

        Ok(self.output.clone())
    }

    fn generate_module(&mut self, module: &Module) -> Result<(), String> {
        writeln!(self.output, "{}// Module: {}", self.indent(), module.name.to_string())
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
            Item::EffectDef(_) => Ok(()), // Skip effect definitions for now
            Item::HandlerDef(_) => Ok(()), // Skip handler definitions for now
            Item::ModuleTypeDef(_) => Ok(()), // Skip module type definitions for now
        }
    }

    fn generate_interface_def(&mut self, interface: &ComponentInterface) -> Result<(), String> {
        writeln!(self.output, "{}interface {} {{", self.indent(), &interface.name)
            .map_err(|e| format!("Failed to write interface declaration: {}", e))?;
        self.indent_level += 1;

        // Generate interface items
        for item in &interface.items {
            self.generate_interface_item(item)?;
        }

        self.indent_level -= 1;
        writeln!(self.output, "{}}}", self.indent())
            .map_err(|e| format!("Failed to close interface: {}", e))?;

        writeln!(self.output, "")
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        Ok(())
    }

    fn generate_interface_item(&mut self, item: &InterfaceItem) -> Result<(), String> {
        match item {
            InterfaceItem::Func { name, signature, .. } => self.generate_function_signature(name, signature),
            InterfaceItem::Type { name, definition, .. } => {
                if let Some(def) = definition {
                    writeln!(self.output, "{}type {} = {};", self.indent(), name.as_str(), self.type_to_wit(def))
                        .map_err(|e| format!("Failed to write type definition: {}", e))?;
                }
                Ok(())
            },
            InterfaceItem::Resource { name, methods, .. } => self.generate_resource(name, methods),
        }
    }

    fn generate_function_signature(&mut self, name: &Symbol, func: &FunctionSignature) -> Result<(), String> {
        write!(self.output, "{}{}: func(", self.indent(), name.as_str())
            .map_err(|e| format!("Failed to write function signature: {}", e))?;

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ")
                    .map_err(|e| format!("Failed to write parameter separator: {}", e))?;
            }
            write!(self.output, "param{}: {}", i, self.wasm_type_to_wit(param))
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

    #[allow(dead_code)]
    fn generate_wasm_type(&mut self, name: &Symbol, wasm_type: &WasmType) -> Result<(), String> {
        writeln!(self.output, "{}type {} = {};", self.indent(), name.as_str(), self.wasm_type_to_wit(wasm_type))
            .map_err(|e| format!("Failed to write type definition: {}", e))?;
        Ok(())
    }

    fn generate_resource(&mut self, name: &Symbol, methods: &[ResourceMethod]) -> Result<(), String> {
        writeln!(self.output, "{}resource {} {{", self.indent(), name.as_str())
            .map_err(|e| format!("Failed to write resource declaration: {}", e))?;
        self.indent_level += 1;

        // Generate methods
        for method in methods {
            self.generate_resource_method(method)?;
        }

        self.indent_level -= 1;
        writeln!(self.output, "{}}}", self.indent())
            .map_err(|e| format!("Failed to close resource: {}", e))?;

        Ok(())
    }

    fn generate_resource_method(&mut self, method: &ResourceMethod) -> Result<(), String> {
        let method_type = if method.is_constructor {
            "constructor"
        } else if method.is_static {
            "static"
        } else {
            ""
        };

        write!(self.output, "{}{} {}: func(", self.indent(), method_type, method.name.as_str())
            .map_err(|e| format!("Failed to write resource method: {}", e))?;

        // Parameters
        for (i, param) in method.signature.params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ")
                    .map_err(|e| format!("Failed to write parameter separator: {}", e))?;
            }
            write!(self.output, "param{}: {}", i, self.wasm_type_to_wit(param))
                .map_err(|e| format!("Failed to write parameter: {}", e))?;
        }

        write!(self.output, ")")
            .map_err(|e| format!("Failed to close parameter list: {}", e))?;

        // Return type
        if !method.signature.results.is_empty() {
            write!(self.output, " -> ")
                .map_err(|e| format!("Failed to write return arrow: {}", e))?;

            if method.signature.results.len() == 1 {
                write!(self.output, "{}", self.wasm_type_to_wit(&method.signature.results[0]))
                    .map_err(|e| format!("Failed to write return type: {}", e))?;
            } else {
                write!(self.output, "(")
                    .map_err(|e| format!("Failed to open return tuple: {}", e))?;
                for (i, result) in method.signature.results.iter().enumerate() {
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
        
        // Convert x-lang type to WIT type
        match &type_def.kind {
            TypeDefKind::Data(constructors) => {
                // Generate variant for sum types
                writeln!(self.output, "{}variant {} {{", self.indent(), type_def.name.as_str())
                    .map_err(|e| format!("Failed to write variant declaration: {}", e))?;
                self.indent_level += 1;

                for constructor in constructors {
                    if constructor.fields.is_empty() {
                        writeln!(self.output, "{}{},", self.indent(), constructor.name.as_str())
                            .map_err(|e| format!("Failed to write variant constructor: {}", e))?;
                    } else if constructor.fields.len() == 1 {
                        writeln!(self.output, "{}{}({}),", self.indent(), constructor.name.as_str(), self.type_to_wit(&constructor.fields[0]))
                            .map_err(|e| format!("Failed to write variant constructor: {}", e))?;
                    } else {
                        // Multiple fields become a tuple
                        let fields_str = constructor.fields.iter()
                            .map(|t| self.type_to_wit(t))
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(self.output, "{}{}(tuple<{}>),", self.indent(), constructor.name.as_str(), fields_str)
                            .map_err(|e| format!("Failed to write variant constructor: {}", e))?;
                    }
                }

                self.indent_level -= 1;
                writeln!(self.output, "{}}}", self.indent())
                    .map_err(|e| format!("Failed to close variant: {}", e))?;
            }
            TypeDefKind::Alias(aliased_type) => {
                writeln!(self.output, "{}type {} = {};", self.indent(), type_def.name.as_str(), self.type_to_wit(aliased_type))
                    .map_err(|e| format!("Failed to write type alias: {}", e))?;
            }
            TypeDefKind::Abstract => {
                // Abstract types can't be directly represented in WIT
                writeln!(self.output, "{}// Abstract type: {}", self.indent(), type_def.name.as_str())
                    .map_err(|e| format!("Failed to write abstract type comment: {}", e))?;
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
                self.type_to_wit(&value_def.type_annotation.as_ref().unwrap_or(&Type::Con(Symbol::from("any"), Span::new(FileId::INVALID, ByteOffset::INVALID, ByteOffset::INVALID)))))
                .map_err(|e| format!("Failed to write value export: {}", e))?;
        }

        Ok(())
    }


    fn wasm_type_to_wit(&self, wasm_type: &WasmType) -> String {
        match wasm_type {
            WasmType::I32 => "s32".to_string(),
            WasmType::I64 => "s64".to_string(),
            WasmType::F32 => "f32".to_string(),
            WasmType::F64 => "f64".to_string(),
            WasmType::V128 => "v128".to_string(),
            WasmType::FuncRef => "funcref".to_string(),
            WasmType::ExternRef => "externref".to_string(),
            WasmType::Named(name) => name.as_str().to_string(),
        }
    }

    fn type_to_wit(&self, type_expr: &Type) -> String {
        match type_expr {
            Type::Var(name, _) => name.as_str().to_string(),
            Type::Con(name, _) => {
                // Map x-lang type constructors to WIT types
                match name.as_str() {
                    "Unit" => "()".to_string(),
                    "Bool" => "bool".to_string(),
                    "Int" => "s32".to_string(),
                    "Float" => "f64".to_string(),
                    "String" => "string".to_string(),
                    "Char" => "char".to_string(),
                    _ => name.as_str().to_string(),
                }
            }
            Type::App(con, args, _) => {
                if let Type::Con(name, _) = con.as_ref() {
                    match name.as_str() {
                        "List" if args.len() == 1 => {
                            format!("list<{}>", self.type_to_wit(&args[0]))
                        }
                        "Option" if args.len() == 1 => {
                            format!("option<{}>", self.type_to_wit(&args[0]))
                        }
                        "Result" if args.len() == 2 => {
                            format!("result<{}, {}>", self.type_to_wit(&args[0]), self.type_to_wit(&args[1]))
                        }
                        _ => {
                            // Generic type application
                            format!("{}<{}>", name.as_str(), 
                                args.iter().map(|t| self.type_to_wit(t)).collect::<Vec<_>>().join(", "))
                        }
                    }
                } else {
                    "unknown".to_string()
                }
            }
            Type::Fun { params, return_type, .. } => {
                let param_strs: Vec<String> = params.iter().map(|t| self.type_to_wit(t)).collect();
                format!("func({}) -> {}", param_strs.join(", "), self.type_to_wit(return_type))
            }
            _ => "unknown".to_string(),
        }
    }

    #[allow(dead_code)]
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


#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::Symbol;

    #[test]
    fn test_basic_wit_generation() {
        let mut generator = WitGenerator::new();
        let compilation_unit = CompilationUnit {
            module: Module {
                name: ModulePath::single(Symbol::intern("test:package"), Span::default()),
                exports: None,
                imports: vec![],
                items: vec![],
                span: Span::default(),
            },
            span: Span::default(),
        };

        let result = generator.generate(&compilation_unit).unwrap();
        assert!(result.contains("package test:package;"));
        assert!(result.contains("world effect-lang {"));
    }

    #[test]
    fn test_wasm_type_conversion() {
        let generator = WitGenerator::new();
        assert_eq!(generator.wasm_type_to_wit(&WasmType::I32), "s32".to_string());
        assert_eq!(generator.wasm_type_to_wit(&WasmType::Named(Symbol::intern("string"))), "string".to_string());
    }
}