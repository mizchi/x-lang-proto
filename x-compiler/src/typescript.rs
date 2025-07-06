//! TypeScript code generation backend
//! 
//! This backend generates TypeScript code from x Language IR,
//! including full type information and effect system translation.

use crate::{
    backend::*,
    ir::*,
    utils,
    Result,
};
use crate::codegen_mod::TypeScriptModuleSystem;
use x_parser::{CompilationUnit, Module, Symbol, Visibility};
use x_checker::TypeScheme;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// TypeScript code generation backend
#[allow(dead_code)]
pub struct TypeScriptBackend {
    module_system: TypeScriptModuleSystem,
    emit_types: bool,
    strict_mode: bool,
    generated_names: HashSet<String>,
}

impl TypeScriptBackend {
    pub fn new() -> Self {
        TypeScriptBackend {
            module_system: TypeScriptModuleSystem::ES2020,
            emit_types: true,
            strict_mode: true,
            generated_names: HashSet::new(),
        }
    }
    
    pub fn with_module_system(mut self, module_system: TypeScriptModuleSystem) -> Self {
        self.module_system = module_system;
        self
    }
    
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
}

impl CodegenBackend for TypeScriptBackend {
    fn target_info(&self) -> CompilationTarget {
        CompilationTarget {
            name: "TypeScript".to_string(),
            file_extension: "ts".to_string(),
            supports_modules: true,
            supports_effects: true, // Via async/await and generators
            supports_gc: true,      // JavaScript has GC
        }
    }
    
    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "modules" | "types" | "effects" | "async" | "closures" => true,
            "gc" | "weakrefs" => true,
            "threads" => false, // Web Workers are different
            _ => false,
        }
    }
    
    fn generate_code(
        &mut self,
        cu: &CompilationUnit,
        type_info: &HashMap<Symbol, TypeScheme>,
        options: &CodegenOptions,
    ) -> Result<CodegenResult> {
        let start_time = std::time::Instant::now();
        
        // Convert AST to IR
        let mut ir_builder = IRBuilder::new();
        let ir = ir_builder.build_ir(cu)?;
        
        // Generate TypeScript code
        let mut files = HashMap::new();
        let diagnostics = Vec::new();
        
        for module in &ir.modules {
            let module_code = self.generate_ir_module(module, type_info, options)?;
            let filename = format!("{}.ts", module.name.as_str());
            files.insert(options.output_dir.join(&filename), module_code);
        }
        
        // Generate runtime if needed
        if self.needs_runtime(&ir) {
            let runtime_code = self.generate_runtime(options)?;
            files.insert(options.output_dir.join("runtime.ts"), runtime_code);
        }
        
        // Generate type definitions
        if self.emit_types {
            let types_code = self.generate_type_definitions(&ir)?;
            files.insert(options.output_dir.join("types.d.ts"), types_code);
        }
        
        let compilation_time = start_time.elapsed();
        let files_len = files.len();
        let total_size = files.values().map(|s| s.len()).sum();
        
        Ok(CodegenResult {
            files,
            source_maps: HashMap::new(), // TODO: Implement source maps
            diagnostics,
            metadata: CodegenMetadata {
                target_info: self.target_info(),
                generated_files: files_len,
                total_size,
                compilation_time,
            },
        })
    }
    
    fn generate_module(
        &mut self,
        module: &Module,
        type_info: &HashMap<Symbol, TypeScheme>,
        options: &CodegenOptions,
    ) -> Result<String> {
        let mut ir_builder = IRBuilder::new();
        // Convert single module to IR
        let ir_module = ir_builder.build_module(module)?; // This method doesn't exist yet
        self.generate_ir_module(&ir_module, type_info, options)
    }
    
    fn generate_runtime(&self, _options: &CodegenOptions) -> Result<String> {
        let mut code = String::new();
        
        writeln!(code, "// x Language Runtime for TypeScript")?;
        writeln!(code, "")?;
        
        // Effect system runtime
        writeln!(code, "// Effect System Runtime")?;
        writeln!(code, "export class EffectContext {{")?;
        writeln!(code, "  private handlers = new Map<string, Function>();")?;
        writeln!(code, "  private stack: any[] = [];")?;
        writeln!(code, "")?;
        writeln!(code, "  addHandler(effect: string, handler: Function): void {{")?;
        writeln!(code, "    this.handlers.set(effect, handler);")?;
        writeln!(code, "  }}")?;
        writeln!(code, "")?;
        writeln!(code, "  perform<T>(effect: string, operation: string, ...args: any[]): T {{")?;
        writeln!(code, "    const handler = this.handlers.get(effect);")?;
        writeln!(code, "    if (!handler) {{")?;
        writeln!(code, "      throw new Error(`Unhandled effect: ${{effect}}.${{operation}}`);")?;
        writeln!(code, "    }}")?;
        writeln!(code, "    return handler(operation, ...args);")?;
        writeln!(code, "  }}")?;
        writeln!(code, "}}")?;
        writeln!(code, "")?;
        
        // Helper functions
        writeln!(code, "// Utility Functions")?;
        writeln!(code, "export function curry<T extends (...args: any[]) => any>(fn: T): any {{")?;
        writeln!(code, "  return function curried(...args: any[]): any {{")?;
        writeln!(code, "    if (args.length >= fn.length) {{")?;
        writeln!(code, "      return fn.apply(this, args);")?;
        writeln!(code, "    }} else {{")?;
        writeln!(code, "      return function (...args2: any[]) {{")?;
        writeln!(code, "        return curried.apply(this, args.concat(args2));")?;
        writeln!(code, "      }};")?;
        writeln!(code, "    }}")?;
        writeln!(code, "  }};")?;
        writeln!(code, "}}")?;
        writeln!(code, "")?;
        
        // Pattern matching support
        writeln!(code, "export class MatchError extends Error {{")?;
        writeln!(code, "  constructor(value: any) {{")?;
        writeln!(code, "    super(`Non-exhaustive pattern match for value: ${{JSON.stringify(value)}}`);")?;
        writeln!(code, "  }}")?;
        writeln!(code, "}}")?;
        
        Ok(code)
    }
}

impl TypeScriptBackend {
    /// Generate code for an IR module
    fn generate_ir_module(
        &mut self,
        module: &IRModule,
        _type_info: &HashMap<Symbol, TypeScheme>,
        _options: &CodegenOptions,
    ) -> Result<String> {
        let mut code = String::new();
        
        // File header
        if self.strict_mode {
            writeln!(code, "\"use strict\";")?;
        }
        writeln!(code, "// Generated from x Language module: {}", module.name)?;
        writeln!(code, "")?;
        
        // Imports
        for import in &module.imports {
            writeln!(code, "{}", self.generate_import(import)?)?;
        }
        if !module.imports.is_empty() {
            writeln!(code, "")?;
        }
        
        // Type definitions
        if self.emit_types {
            for type_def in &module.types {
                writeln!(code, "{}", self.generate_type_definition(type_def)?)?;
                writeln!(code, "")?;
            }
        }
        
        // Constants
        for constant in &module.constants {
            writeln!(code, "{}", self.generate_constant(constant)?)?;
        }
        if !module.constants.is_empty() {
            writeln!(code, "")?;
        }
        
        // Functions
        for function in &module.functions {
            writeln!(code, "{}", self.generate_function(function)?)?;
            writeln!(code, "")?;
        }
        
        // Exports
        if !module.exports.is_empty() {
            writeln!(code, "// Exports")?;
            for export in &module.exports {
                writeln!(code, "{}", self.generate_export(export)?)?;
            }
        }
        
        Ok(code)
    }
    
    /// Generate TypeScript import statement
    fn generate_import(&self, import: &IRImport) -> Result<String> {
        let mut items = Vec::new();
        for item in &import.items {
            if let Some(alias) = &item.alias {
                items.push(format!("{} as {}", item.name, alias));
            } else {
                items.push(item.name.to_string());
            }
        }
        
        match self.module_system {
            TypeScriptModuleSystem::ES2020 => {
                Ok(format!("import {{ {} }} from \"{}\";", items.join(", "), import.module))
            }
            TypeScriptModuleSystem::CommonJS => {
                Ok(format!("const {{ {} }} = require(\"{}\");", items.join(", "), import.module))
            }
            _ => {
                Ok(format!("// TODO: Implement {} imports", 
                    format!("{:?}", self.module_system)))
            }
        }
    }
    
    /// Generate TypeScript function
    fn generate_function(&mut self, function: &IRFunction) -> Result<String> {
        let mut code = String::new();
        
        // Function signature
        let params = function.parameters.iter()
            .map(|p| format!("{}: {}", 
                utils::sanitize_identifier(p.name, "typescript"),
                self.generate_ir_type(&p.type_hint)))
            .collect::<Vec<_>>()
            .join(", ");
        
        let return_type = self.generate_ir_type(&function.return_type);
        let visibility = if function.visibility == Visibility::Public { "export " } else { "" };
        
        // Handle effects (simplified as async for now)
        let async_keyword = if !matches!(function.effects, IREffectSet::Empty) {
            "async "
        } else {
            ""
        };
        
        write!(code, "{}{}function {}({}): {} {{", 
               visibility, async_keyword, 
               utils::sanitize_identifier(function.name, "typescript"),
               params, return_type)?;
        
        // Function body
        writeln!(code, "")?;
        let body = self.generate_ir_expression(&function.body, 1)?;
        writeln!(code, "  return {};", body)?;
        write!(code, "}}")?;
        
        Ok(code)
    }
    
    /// Generate TypeScript expression
    fn generate_ir_expression(&mut self, expr: &IRExpression, indent: usize) -> Result<String> {
        let indent_str = "  ".repeat(indent);
        
        match expr {
            IRExpression::Literal(lit) => Ok(self.generate_ir_literal(lit)),
            IRExpression::Variable(symbol) => {
                Ok(utils::sanitize_identifier(*symbol, "typescript"))
            }
            IRExpression::Call { function, arguments } => {
                let func_code = self.generate_ir_expression(function, 0)?;
                let args_code = arguments.iter()
                    .map(|arg| self.generate_ir_expression(arg, 0))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");
                Ok(format!("{}({})", func_code, args_code))
            }
            IRExpression::Lambda { parameters, body, .. } => {
                let params = parameters.iter()
                    .map(|p| format!("{}: {}", 
                        utils::sanitize_identifier(p.name, "typescript"),
                        self.generate_ir_type(&p.type_hint)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_code = self.generate_ir_expression(body, 0)?;
                Ok(format!("({}) => {}", params, body_code))
            }
            IRExpression::Let { bindings, body } => {
                let mut code = String::new();
                writeln!(code, "{{")?;
                for binding in bindings {
                    writeln!(code, "{}const {} = {};", 
                             indent_str,
                             utils::sanitize_identifier(binding.name, "typescript"),
                             self.generate_ir_expression(&binding.value, 0)?)?;
                }
                let body_code = self.generate_ir_expression(body, indent)?;
                writeln!(code, "{}return {};", indent_str, body_code)?;
                write!(code, "{}}}", "  ".repeat(indent.saturating_sub(1)))?;
                Ok(code)
            }
            IRExpression::If { condition, then_branch, else_branch } => {
                let cond_code = self.generate_ir_expression(condition, 0)?;
                let then_code = self.generate_ir_expression(then_branch, 0)?;
                let else_code = self.generate_ir_expression(else_branch, 0)?;
                Ok(format!("({} ? {} : {})", cond_code, then_code, else_code))
            }
            IRExpression::Block(expressions) => {
                if expressions.is_empty() {
                    return Ok("undefined".to_string());
                }
                
                let mut code = String::new();
                writeln!(code, "{{")?;
                for (i, expr) in expressions.iter().enumerate() {
                    let expr_code = self.generate_ir_expression(expr, indent + 1)?;
                    if i == expressions.len() - 1 {
                        writeln!(code, "{}return {};", 
                                "  ".repeat(indent + 1), expr_code)?;
                    } else {
                        writeln!(code, "{}{};", 
                                "  ".repeat(indent + 1), expr_code)?;
                    }
                }
                write!(code, "{}}}", "  ".repeat(indent))?;
                Ok(code)
            }
            _ => {
                // Handle other expression types
                Ok("/* TODO: Implement expression */".to_string())
            }
        }
    }
    
    /// Generate TypeScript literal
    fn generate_ir_literal(&mut self, lit: &IRLiteral) -> String {
        match lit {
            IRLiteral::Integer(n) => n.to_string(),
            IRLiteral::Float(f) => f.to_string(),
            IRLiteral::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
            IRLiteral::Boolean(b) => b.to_string(),
            IRLiteral::Unit => "undefined".to_string(),
            IRLiteral::Array(elements) => {
                let elements_code = elements.iter()
                    .map(|e| match self.generate_ir_expression(e, 0) {
                        Ok(code) => code,
                        Err(_) => "/* error */".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements_code)
            }
            IRLiteral::Record(fields) => {
                let fields_code = fields.iter()
                    .map(|(name, expr)| {
                        let expr_code = match self.generate_ir_expression(expr, 0) {
                            Ok(code) => code,
                            Err(_) => "/* error */".to_string(),
                        };
                        format!("{}: {}", name, expr_code)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", fields_code)
            }
        }
    }
    
    /// Generate TypeScript type annotation
    fn generate_ir_type(&self, typ: &IRType) -> String {
        match typ {
            IRType::Primitive(prim) => self.generate_primitive_type(prim),
            IRType::Function { parameters, return_type, .. } => {
                let params = parameters.iter()
                    .enumerate()
                    .map(|(i, t)| format!("arg{}: {}", i, self.generate_ir_type(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = self.generate_ir_type(return_type);
                format!("({}) => {}", params, ret)
            }
            IRType::Tuple(types) => {
                let types_str = types.iter()
                    .map(|t| self.generate_ir_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", types_str)
            }
            IRType::Array(element_type) => {
                format!("{}[]", self.generate_ir_type(element_type))
            }
            IRType::Named(name) => utils::sanitize_identifier(*name, "typescript"),
            _ => "any".to_string(), // Fallback
        }
    }
    
    fn generate_primitive_type(&self, prim: &IRPrimitiveType) -> String {
        match prim {
            IRPrimitiveType::Int => "number".to_string(),
            IRPrimitiveType::Float => "number".to_string(),
            IRPrimitiveType::String => "string".to_string(),
            IRPrimitiveType::Bool => "boolean".to_string(),
            IRPrimitiveType::Unit => "void".to_string(),
        }
    }
    
    /// Generate other constructs
    fn generate_constant(&mut self, constant: &IRConstant) -> Result<String> {
        Ok(format!("export const {}: {} = {};",
                   utils::sanitize_identifier(constant.name, "typescript"),
                   self.generate_ir_type(&constant.type_hint),
                   self.generate_ir_literal(&match &constant.value {
                       IRExpression::Literal(lit) => lit.clone(),
                       _ => IRLiteral::Unit, // Simplified
                   })))
    }
    
    fn generate_export(&self, export: &IRExport) -> Result<String> {
        if let Some(alias) = &export.alias {
            Ok(format!("export {{ {} as {} }};", export.name, alias))
        } else {
            Ok(format!("export {{ {} }};", export.name))
        }
    }
    
    fn generate_type_definition(&self, _type_def: &IRTypeDefinition) -> Result<String> {
        // Simplified implementation
        Ok("// TODO: Type definition".to_string())
    }
    
    fn generate_type_definitions(&self, _ir: &IR) -> Result<String> {
        let mut code = String::new();
        writeln!(code, "// x Language Type Definitions")?;
        writeln!(code, "")?;
        writeln!(code, "declare global {{")?;
        writeln!(code, "  namespace x Language {{")?;
        writeln!(code, "    // Type definitions will be generated here")?;
        writeln!(code, "  }}")?;
        writeln!(code, "}}")?;
        Ok(code)
    }
    
    fn needs_runtime(&self, _ir: &IR) -> bool {
        // For now, always include runtime
        true
    }
}

impl Default for TypeScriptBackend {
    fn default() -> Self {
        Self::new()
    }
}