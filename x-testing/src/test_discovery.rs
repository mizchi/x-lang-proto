//! Test discovery module
//!
//! This module finds and collects test functions from x Language namespaces.

use std::collections::HashMap;
use anyhow::{Result, Context};
use x_parser::{Symbol, ast::*};
use x_editor::namespace::{Namespace, NamespacePath};
use x_editor::content_addressing::{ContentHash, ContentRepository};
use x_editor::annotated_ast::{AnnotatedValueDef, AnnotatedExpr};
use x_parser::ast::Visibility;
use x_checker::types::TypeScheme;

/// Test case
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Test name
    pub name: Symbol,
    
    /// Full path to the test
    pub full_path: String,
    
    /// Namespace containing the test
    pub namespace: NamespacePath,
    
    /// The test function
    pub function: AnnotatedValueDef,
    
    /// Content hash of the test function
    pub hash: ContentHash,
    
    /// Test attributes (e.g., tags, skip conditions)
    pub attributes: TestAttributes,
}

/// Test attributes
#[derive(Debug, Clone, Default)]
pub struct TestAttributes {
    /// Tags for grouping tests
    pub tags: Vec<String>,
    
    /// Whether this test should be skipped
    pub skip: bool,
    
    /// Skip reason
    pub skip_reason: Option<String>,
    
    /// Expected to fail
    pub should_fail: bool,
    
    /// Test timeout override
    pub timeout_seconds: Option<u64>,
}

/// Test suite
#[derive(Debug, Clone)]
pub struct TestSuite {
    /// Suite name
    pub name: String,
    
    /// All discovered tests
    pub tests: Vec<TestCase>,
    
    /// Tests grouped by namespace
    pub by_namespace: HashMap<NamespacePath, Vec<TestCase>>,
    
    /// Tests grouped by tag
    pub by_tag: HashMap<String, Vec<TestCase>>,
}

/// Test discovery
pub struct TestDiscovery {
    content_repo: ContentRepository,
}

impl TestDiscovery {
    pub fn new(content_repo: ContentRepository) -> Self {
        Self { content_repo }
    }
    
    /// Discover all tests in a namespace and its subnamespaces
    pub fn discover_in_namespace(&self, namespace: &Namespace) -> Result<TestSuite> {
        let mut suite = TestSuite {
            name: namespace.path.to_string(),
            tests: Vec::new(),
            by_namespace: HashMap::new(),
            by_tag: HashMap::new(),
        };
        
        self.discover_recursive(namespace, &mut suite)?;
        
        Ok(suite)
    }
    
    /// Discover tests in multiple namespaces
    pub fn discover_in_namespaces(&self, namespaces: &[Namespace]) -> Result<TestSuite> {
        let mut suite = TestSuite {
            name: "All Tests".to_string(),
            tests: Vec::new(),
            by_namespace: HashMap::new(),
            by_tag: HashMap::new(),
        };
        
        for namespace in namespaces {
            self.discover_recursive(namespace, &mut suite)?;
        }
        
        Ok(suite)
    }
    
    /// Discover tests by pattern
    pub fn discover_by_pattern(&self, namespace: &Namespace, pattern: &str) -> Result<TestSuite> {
        let all_tests = self.discover_in_namespace(namespace)?;
        
        let filtered_tests: Vec<TestCase> = all_tests.tests
            .into_iter()
            .filter(|test| test.name.as_str().contains(pattern))
            .collect();
        
        let mut suite = TestSuite {
            name: format!("Tests matching '{}'", pattern),
            tests: filtered_tests,
            by_namespace: HashMap::new(),
            by_tag: HashMap::new(),
        };
        
        // Rebuild indices
        self.rebuild_indices(&mut suite);
        
        Ok(suite)
    }
    
    /// Discover tests by tag
    pub fn discover_by_tag(&self, namespace: &Namespace, tag: &str) -> Result<TestSuite> {
        let all_tests = self.discover_in_namespace(namespace)?;
        
        let filtered_tests: Vec<TestCase> = all_tests.tests
            .into_iter()
            .filter(|test| test.attributes.tags.contains(&tag.to_string()))
            .collect();
        
        let mut suite = TestSuite {
            name: format!("Tests tagged '{}'", tag),
            tests: filtered_tests,
            by_namespace: HashMap::new(),
            by_tag: HashMap::new(),
        };
        
        // Rebuild indices
        self.rebuild_indices(&mut suite);
        
        Ok(suite)
    }
    
    fn discover_recursive(&self, namespace: &Namespace, suite: &mut TestSuite) -> Result<()> {
        // Find test functions in this namespace
        for (name, member) in &namespace.bindings {
            if let Some(test_case) = self.check_if_test(name, member, &namespace.path)? {
                suite.tests.push(test_case.clone());
                
                // Add to namespace index
                suite.by_namespace
                    .entry(namespace.path.clone())
                    .or_insert_with(Vec::new)
                    .push(test_case.clone());
                
                // Add to tag indices
                for tag in &test_case.attributes.tags {
                    suite.by_tag
                        .entry(tag.clone())
                        .or_insert_with(Vec::new)
                        .push(test_case.clone());
                }
            }
        }
        
        // Recursively discover in subnamespaces
        // In the current implementation, sub-namespaces are stored as NameBinding::Namespace
        for (_, binding) in &namespace.bindings {
            if let NameBinding::Namespace { namespace: sub_namespace } = binding {
                self.discover_recursive(sub_namespace, suite)?;
            }
        }
        
        Ok(())
    }
    
    fn check_if_test(
        &self,
        name: &Symbol,
        member: &NameBinding,
        namespace_path: &NamespacePath,
    ) -> Result<Option<TestCase>> {
        // Check if this is a value definition
        if let NameBinding::Value { hash, type_scheme, .. } = member {
            // Check if it's a test function based on name and type
            if self.is_test_function_by_name(name) || 
               (type_scheme.as_ref().map_or(false, |ts| self.is_test_return_type(&ts.body))) {
                let full_path = format!("{}::{}", namespace_path.to_string(), name);
                
                // For now, create a dummy AnnotatedValueDef since we don't have the full definition
                // In a real implementation, we'd retrieve this from the content repository
                let function = self.create_dummy_test_function(name, type_scheme.clone());
                let attributes = self.extract_test_attributes(&function)?;
                
                return Ok(Some(TestCase {
                    name: name.clone(),
                    full_path,
                    namespace: namespace_path.clone(),
                    function,
                    hash: hash.clone(),
                    attributes,
                }));
            }
        }
        
        Ok(None)
    }
    
    fn is_test_function_by_name(&self, name: &Symbol) -> bool {
        let name_str = name.as_str();
        name_str.starts_with("test_") || name_str.starts_with("test") && name_str.len() > 4
    }
    
    fn create_dummy_test_function(&self, name: &Symbol, type_scheme: Option<TypeScheme>) -> AnnotatedValueDef {
        use x_parser::{Span, FileId};
        use x_parser::span::ByteOffset;
        use x_parser::ast::Purity;
        
        AnnotatedValueDef {
            name: name.clone(),
            type_annotation: None,
            inferred_type: type_scheme,
            parameters: vec![],
            body: AnnotatedExpr {
                expr: Expr::Literal(Literal::Bool(true), Span::new(FileId(0), ByteOffset(0), ByteOffset(1))),
                inferred_type: None,
                span: Span::new(FileId(0), ByteOffset(0), ByteOffset(1)),
            },
            visibility: Visibility::Public,
            purity: Purity::Pure,
            span: Span::new(FileId(0), ByteOffset(0), ByteOffset(1)),
        }
    }
    
    fn is_test_return_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Con(name) => {
                let name_str = name.as_str();
                name_str == "Bool" || name_str == "Unit"
            }
            Type::Fun { return_type, .. } => self.is_test_return_type(return_type),
            _ => false,
        }
    }
    
    fn extract_test_attributes(&self, value_def: &AnnotatedValueDef) -> Result<TestAttributes> {
        let mut attributes = TestAttributes::default();
        
        // Look for special comments or annotations in the function body
        // For now, we'll just use defaults
        // In a real implementation, we'd parse doc comments or special annotations
        
        // Example: Check if function name contains "skip"
        if value_def.name.as_str().contains("skip") {
            attributes.skip = true;
            attributes.skip_reason = Some("Contains 'skip' in name".to_string());
        }
        
        // Example: Check if function name contains "fail"
        if value_def.name.as_str().contains("should_fail") {
            attributes.should_fail = true;
        }
        
        Ok(attributes)
    }
    
    fn rebuild_indices(&self, suite: &mut TestSuite) {
        suite.by_namespace.clear();
        suite.by_tag.clear();
        
        for test in &suite.tests {
            // Namespace index
            suite.by_namespace
                .entry(test.namespace.clone())
                .or_insert_with(Vec::new)
                .push(test.clone());
            
            // Tag indices
            for tag in &test.attributes.tags {
                suite.by_tag
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(test.clone());
            }
        }
    }
}

// Missing imports
use x_editor::namespace::NameBinding;
use x_checker::types::Type;
use x_parser::ast::Literal;

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::Span;
    use x_parser::ast::Visibility;
    use x_checker::Purity;
    
    #[test]
    fn test_discovery_basic() {
        let content_repo = ContentRepository::new();
        let discovery = TestDiscovery::new(content_repo);
        
        // Create a test namespace with test functions
        let mut namespace = Namespace::new(NamespacePath::from_str("TestModule").unwrap());
        
        // Add a test function
        let test_fn = AnnotatedValueDef {
            name: Symbol::intern("test_addition"),
            type_annotation: None,
            inferred_type: None,
            parameters: vec![],
            body: AnnotatedExpr {
                expr: Expr::Literal(Literal::Bool(true), Span::new(FileId(0), ByteOffset(0), ByteOffset(1))),
                inferred_type: None,
                span: Span::new(FileId(0), ByteOffset(0), ByteOffset(1)),
            },
            visibility: Visibility::Public,
            purity: Purity::Pure,
            span: Span::new(FileId(0), ByteOffset(0), ByteOffset(1)),
        };
        
        let test_hash = ContentHash::new(b"test_addition_hash");
        namespace.bindings.insert(
            Symbol::intern("test_addition"),
            NameBinding::Value {
                hash: test_hash,
                type_scheme: None,
                visibility: Visibility::Public,
            },
        );
        
        let suite = discovery.discover_in_namespace(&namespace).unwrap();
        assert_eq!(suite.tests.len(), 1);
        assert_eq!(suite.tests[0].name.as_str(), "test_addition");
    }
}