//! DSL macros for AST construction
//! 
//! Provides Rust macros that make AST construction more concise and readable.

/// Macro for building x Language modules
#[macro_export]
macro_rules! x_module {
    ($name:expr => {
        $($item:tt)*
    }) => {{
        let mut builder = $crate::AstBuilder::new();
        let module_builder = builder.module($name);
        x_module_items!(module_builder, $($item)*)
    }};
}

#[macro_export]
macro_rules! x_module_items {
    ($builder:expr, ) => { $builder.build() };
    
    // Value definition
    ($builder:expr, let $name:ident = $value:expr; $($rest:tt)*) => {
        x_module_items!($builder.value(stringify!($name), |e| x_expr!(e, $value)), $($rest)*)
    };
    
    // Function definition
    ($builder:expr, let $name:ident = fun $($param:ident)+ -> $body:expr; $($rest:tt)*) => {
        x_module_items!(
            $builder.function(
                stringify!($name),
                vec![$(stringify!($param)),+],
                |e| x_expr!(e, $body)
            ),
            $($rest)*
        )
    };
    
    // Type definition
    ($builder:expr, data $name:ident = $($variant:ident $($field:ident)*)|+; $($rest:tt)*) => {
        x_module_items!(
            $builder.data_type(
                stringify!($name),
                vec![$((stringify!($variant), vec![$(stringify!($field)),*])),+]
            ),
            $($rest)*
        )
    };
    
    // Import
    ($builder:expr, import $module:ident; $($rest:tt)*) => {
        x_module_items!($builder.import(stringify!($module)), $($rest)*)
    };
}

/// Macro for building expressions
#[macro_export]
macro_rules! x_expr {
    // Literals
    ($builder:expr, $n:literal) => {
        x_expr_literal!($builder, $n)
    };
    
    // Variables
    ($builder:expr, $var:ident) => {
        $builder.var(stringify!($var))
    };
    
    // Binary operations
    ($builder:expr, $left:tt + $right:tt) => {
        $builder.binop("+", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    ($builder:expr, $left:tt - $right:tt) => {
        $builder.binop("-", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    ($builder:expr, $left:tt * $right:tt) => {
        $builder.binop("*", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    ($builder:expr, $left:tt / $right:tt) => {
        $builder.binop("/", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    // Comparison
    ($builder:expr, $left:tt > $right:tt) => {
        $builder.binop(">", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    ($builder:expr, $left:tt < $right:tt) => {
        $builder.binop("<", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    ($builder:expr, $left:tt == $right:tt) => {
        $builder.binop("==", |e| x_expr!(e, $left), |e| x_expr!(e, $right))
    };
    
    // Function application
    ($builder:expr, $func:ident($($arg:expr),*)) => {
        $builder.app(stringify!($func), vec![$(|e| x_expr!(e, $arg)),*])
    };
    
    // If-then-else (using block syntax to avoid parser conflicts)
    ($builder:expr, if $cond:tt then { $then:expr } else { $else:expr }) => {
        $builder.if_then_else(
            |e| x_expr!(e, $cond),
            |e| x_expr!(e, $then),
            |e| x_expr!(e, $else)
        )
    };
    
    // Let-in (using block syntax to avoid parser conflicts)
    ($builder:expr, let $name:ident = $value:tt in { $body:expr }) => {
        $builder.let_in(
            stringify!($name),
            |e| x_expr!(e, $value),
            |e| x_expr!(e, $body)
        )
    };
    
    // Lambda
    ($builder:expr, fun $($param:ident)+ -> $body:expr) => {
        $builder.lambda(
            vec![$(stringify!($param)),+],
            |e| x_expr!(e, $body)
        )
    };
    
    // List
    ($builder:expr, [$($elem:expr),*]) => {
        $builder.list(vec![$(|e| x_expr!(e, $elem)),*])
    };
    
    // Parentheses (just pass through)
    ($builder:expr, ($expr:expr)) => {
        x_expr!($builder, $expr)
    };
}

#[macro_export]
macro_rules! x_expr_literal {
    ($builder:expr, $n:literal) => {{
        // Use compile-time type checking to determine literal type
        let _ = $n; // Ensure $n is used
        match stringify!($n) {
            s if s.contains('.') => $builder.float($n as f64),
            s if s.starts_with('"') => $builder.string(&$n),
            s if s == "true" || s == "false" => $builder.bool($n),
            _ => $builder.int($n as i64),
        }
    }};
}

/// Macro for building types
#[macro_export]
macro_rules! x_type {
    // Type constructor
    ($builder:expr, $name:ident) => {
        $builder.con(stringify!($name))
    };
    
    // Type variable
    ($builder:expr, $var:ident) => {
        $builder.var(stringify!($var))
    };
    
    // Function type
    ($builder:expr, $param:tt -> $result:tt) => {
        $builder.fun(|t| x_type!(t, $param), |t| x_type!(t, $result))
    };
    
    // Type application
    ($builder:expr, $con:ident[$($arg:tt),+]) => {
        $builder.app(stringify!($con), vec![$(|t| x_type!(t, $arg)),+])
    };
}

/// Macro for building patterns
#[macro_export]
macro_rules! x_pattern {
    // Variable pattern
    ($builder:expr, $var:ident) => {
        $builder.var(stringify!($var))
    };
    
    // Wildcard
    ($builder:expr, _) => {
        $builder.wildcard()
    };
    
    // Constructor pattern
    ($builder:expr, $con:ident($($arg:tt)*)) => {
        x_pattern_constructor!($builder, $con, $($arg)*)
    };
    
    // Literal pattern
    ($builder:expr, $lit:literal) => {
        $builder.literal($lit.into())
    };
}

#[macro_export]
macro_rules! x_pattern_constructor {
    ($builder:expr, $con:ident, ) => {
        $builder.constructor(stringify!($con), vec![])
    };
    
    ($builder:expr, $con:ident, $($arg:tt),+) => {
        $builder.constructor(stringify!($con), vec![$(|p| x_pattern!(p, $arg)),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_macro() {
        let _module = x_module!("TestModule" => {
            let x = 42;
            let y = "hello";
            let add = fun a b -> a + b;
        });
    }
    
    #[test]
    fn test_complex_module() {
        let _module = x_module!("ComplexModule" => {
            import List;
            
            data Option = None | Some value;
            
            let map = fun f opt ->
                if opt == None then {
                    None
                } else {
                    Some(f(opt))
                };
                    
            let main = fun () ->
                let x = 10 in {
                    let y = x * 2 in {
                        print_endline(string_of_int(y))
                    }
                };
        });
    }
}