//! Built-in types and operators for x Language
//! 
//! This module defines the core built-in types and operators that are
//! available in every x Language program without explicit imports.

use crate::types::{Type, TypeScheme, TypeVar, EffectSet, Effect};
use x_parser::{Symbol, Span, span::ByteOffset, FileId};
use std::collections::HashMap;

/// Built-in types
#[derive(Debug)]
pub struct BuiltinTypes {
    types: HashMap<Symbol, Type>,
}

impl BuiltinTypes {
    pub fn new() -> Self {
        let mut types = HashMap::new();
        
        // Basic types
        types.insert(Symbol::intern("Int"), Type::Con(Symbol::intern("Int")));
        types.insert(Symbol::intern("Float"), Type::Con(Symbol::intern("Float")));
        types.insert(Symbol::intern("String"), Type::Con(Symbol::intern("String")));
        types.insert(Symbol::intern("Bool"), Type::Con(Symbol::intern("Bool")));
        types.insert(Symbol::intern("Unit"), Type::Con(Symbol::intern("Unit")));
        
        // Container types
        types.insert(Symbol::intern("List"), Type::Con(Symbol::intern("List")));
        types.insert(Symbol::intern("Option"), Type::Con(Symbol::intern("Option")));
        types.insert(Symbol::intern("Result"), Type::Con(Symbol::intern("Result")));
        
        Self { types }
    }
    
    pub fn get(&self, name: &Symbol) -> Option<&Type> {
        self.types.get(name)
    }
    
    pub fn contains(&self, name: &Symbol) -> bool {
        self.types.contains_key(name)
    }
}

/// Built-in operators with their type schemes
#[derive(Debug)]
pub struct BuiltinOperators {
    operators: HashMap<Symbol, TypeScheme>,
}

impl BuiltinOperators {
    pub fn new() -> Self {
        let mut operators = HashMap::new();
        let _dummy_span = Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0));
        
        // Arithmetic operators
        let int_type = Type::Con(Symbol::intern("Int"));
        let float_type = Type::Con(Symbol::intern("Float"));
        let bool_type = Type::Con(Symbol::intern("Bool"));
        let string_type = Type::Con(Symbol::intern("String"));
        
        // Int -> Int -> Int
        let int_binop = TypeScheme::monotype(Type::Fun {
            params: vec![int_type.clone()],
            return_type: Box::new(Type::Fun {
                params: vec![int_type.clone()],
                return_type: Box::new(int_type.clone()),
                effects: EffectSet::empty(),
            }),
            effects: EffectSet::empty(),
        });
        
        operators.insert(Symbol::intern("+"), int_binop.clone());
        operators.insert(Symbol::intern("-"), int_binop.clone());
        operators.insert(Symbol::intern("*"), int_binop.clone());
        operators.insert(Symbol::intern("/"), int_binop.clone());
        operators.insert(Symbol::intern("mod"), int_binop.clone());
        
        // Float -> Float -> Float
        let float_binop = TypeScheme::monotype(Type::Fun {
            params: vec![float_type.clone()],
            return_type: Box::new(Type::Fun {
                params: vec![float_type.clone()],
                return_type: Box::new(float_type.clone()),
                effects: EffectSet::empty(),
            }),
            effects: EffectSet::empty(),
        });
        
        operators.insert(Symbol::intern("+."), float_binop.clone());
        operators.insert(Symbol::intern("-."), float_binop.clone());
        operators.insert(Symbol::intern("*."), float_binop.clone());
        operators.insert(Symbol::intern("/."), float_binop.clone());
        
        // Comparison operators: 'a -> 'a -> Bool
        let a = TypeVar(0);
        let comparison = TypeScheme {
            type_vars: vec![a],
            effect_vars: vec![],
            constraints: vec![],
            body: Type::Fun {
                params: vec![Type::Var(a)],
                return_type: Box::new(Type::Fun {
                    params: vec![Type::Var(a)],
                    return_type: Box::new(bool_type.clone()),
                    effects: EffectSet::empty(),
                }),
                effects: EffectSet::empty(),
            },
        };
        
        operators.insert(Symbol::intern("="), comparison.clone());
        operators.insert(Symbol::intern("<>"), comparison.clone());
        operators.insert(Symbol::intern("<"), comparison.clone());
        operators.insert(Symbol::intern(">"), comparison.clone());
        operators.insert(Symbol::intern("<="), comparison.clone());
        operators.insert(Symbol::intern(">="), comparison.clone());
        
        // Boolean operators
        let bool_binop = TypeScheme::monotype(Type::Fun {
            params: vec![bool_type.clone()],
            return_type: Box::new(Type::Fun {
                params: vec![bool_type.clone()],
                return_type: Box::new(bool_type.clone()),
                effects: EffectSet::empty(),
            }),
            effects: EffectSet::empty(),
        });
        
        operators.insert(Symbol::intern("&&"), bool_binop.clone());
        operators.insert(Symbol::intern("||"), bool_binop.clone());
        
        // Boolean not: Bool -> Bool
        let bool_not = TypeScheme::monotype(Type::Fun {
            params: vec![bool_type.clone()],
            return_type: Box::new(bool_type.clone()),
            effects: EffectSet::empty(),
        });
        
        operators.insert(Symbol::intern("not"), bool_not);
        
        // String concatenation: String -> String -> String
        let string_concat = TypeScheme::monotype(Type::Fun {
            params: vec![string_type.clone()],
            return_type: Box::new(Type::Fun {
                params: vec![string_type.clone()],
                return_type: Box::new(string_type.clone()),
                effects: EffectSet::empty(),
            }),
            effects: EffectSet::empty(),
        });
        
        operators.insert(Symbol::intern("^"), string_concat);
        
        // List operations
        let list_cons_type = {
            let a = TypeVar(0);
            let list_a = Type::App(
                Box::new(Type::Con(Symbol::intern("List"))),
                vec![Type::Var(a)]
            );
            
            TypeScheme {
                type_vars: vec![a],
                effect_vars: vec![],
                constraints: vec![],
                body: Type::Fun {
                    params: vec![Type::Var(a)],
                    return_type: Box::new(Type::Fun {
                        params: vec![list_a.clone()],
                        return_type: Box::new(list_a),
                        effects: EffectSet::empty(),
                    }),
                    effects: EffectSet::empty(),
                },
            }
        };
        
        operators.insert(Symbol::intern("::"), list_cons_type);
        
        Self { operators }
    }
    
    pub fn get(&self, name: &Symbol) -> Option<&TypeScheme> {
        self.operators.get(name)
    }
    
    pub fn contains(&self, name: &Symbol) -> bool {
        self.operators.contains_key(name)
    }
}

/// Built-in functions
#[derive(Debug)]
pub struct BuiltinFunctions {
    functions: HashMap<Symbol, TypeScheme>,
}

impl BuiltinFunctions {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        let string_type = Type::Con(Symbol::intern("String"));
        let unit_type = Type::Con(Symbol::intern("Unit"));
        let io_effect = EffectSet::Row {
            effects: vec![Effect {
                name: Symbol::intern("IO"),
                operations: vec![],
            }],
            tail: None,
        };
        
        // print_endline : String -> Unit | IO
        let print_endline_type = TypeScheme::monotype(Type::Fun {
            params: vec![string_type.clone()],
            return_type: Box::new(unit_type.clone()),
            effects: io_effect.clone(),
        });
        
        functions.insert(Symbol::intern("print_endline"), print_endline_type);
        
        // print_string : String -> Unit | IO
        let print_string_type = TypeScheme::monotype(Type::Fun {
            params: vec![string_type.clone()],
            return_type: Box::new(unit_type.clone()),
            effects: io_effect.clone(),
        });
        
        functions.insert(Symbol::intern("print_string"), print_string_type);
        
        // string_of_int : Int -> String
        let string_of_int_type = TypeScheme::monotype(Type::Fun {
            params: vec![Type::Con(Symbol::intern("Int"))],
            return_type: Box::new(string_type.clone()),
            effects: EffectSet::empty(),
        });
        
        functions.insert(Symbol::intern("string_of_int"), string_of_int_type);
        
        // int_of_string : String -> Int
        let int_of_string_type = TypeScheme::monotype(Type::Fun {
            params: vec![string_type.clone()],
            return_type: Box::new(Type::Con(Symbol::intern("Int"))),
            effects: EffectSet::empty(),
        });
        
        functions.insert(Symbol::intern("int_of_string"), int_of_string_type);
        
        Self { functions }
    }
    
    pub fn get(&self, name: &Symbol) -> Option<&TypeScheme> {
        self.functions.get(name)
    }
    
    pub fn contains(&self, name: &Symbol) -> bool {
        self.functions.contains_key(name)
    }
}

/// Main builtin environment
#[derive(Debug)]
pub struct Builtins {
    pub types: BuiltinTypes,
    pub operators: BuiltinOperators,
    pub functions: BuiltinFunctions,
}

impl Builtins {
    pub fn new() -> Self {
        Self {
            types: BuiltinTypes::new(),
            operators: BuiltinOperators::new(),
            functions: BuiltinFunctions::new(),
        }
    }
    
    /// Get type scheme for a symbol (operator or function)
    pub fn get_type_scheme(&self, name: &Symbol) -> Option<&TypeScheme> {
        self.operators.get(name)
            .or_else(|| self.functions.get(name))
    }
    
    /// Check if a symbol is a builtin
    pub fn is_builtin(&self, name: &Symbol) -> bool {
        self.operators.contains(name) || 
        self.functions.contains(name) ||
        self.types.contains(name)
    }
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}