//! Tree similarity algorithms for AST comparison
//! 
//! Implements APTED (All Path Tree Edit Distance) and TSED (Tree Structure Edit Distance)
//! for structural similarity computation.

use x_parser::ast::*;

/// Tree node abstraction for similarity algorithms
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(label: String) -> Self {
        Self {
            label,
            children: Vec::new(),
        }
    }
    
    pub fn with_children(label: String, children: Vec<TreeNode>) -> Self {
        Self { label, children }
    }
    
    /// Convert expression to tree node
    pub fn from_expr(expr: &Expr) -> Self {
        match expr {
            Expr::Literal(lit, _) => {
                let label = match lit {
                    Literal::Integer(n) => format!("Int:{n}"),
                    Literal::Float(f) => format!("Float:{f}"),
                    Literal::String(s) => format!("String:{s}"),
                    Literal::Bool(b) => format!("Bool:{b}"),
                    Literal::Unit => "Unit".to_string(),
                };
                TreeNode::new(label)
            }
            
            Expr::Var(name, _) => {
                TreeNode::new(format!("Var:{}", name.as_str()))
            }
            
            Expr::App(func, args, _) => {
                let mut children = vec![TreeNode::from_expr(func)];
                children.extend(args.iter().map(TreeNode::from_expr));
                TreeNode::with_children("App".to_string(), children)
            }
            
            Expr::Lambda { parameters, body, .. } => {
                let param_nodes: Vec<_> = parameters.iter()
                    .map(TreeNode::from_pattern)
                    .collect();
                let body_node = TreeNode::from_expr(body);
                
                let children = vec![
                    TreeNode::with_children("Params".to_string(), param_nodes),
                    body_node,
                ];
                
                TreeNode::with_children("Lambda".to_string(), children)
            }
            
            Expr::Let { pattern, value, body, .. } => {
                TreeNode::with_children("Let".to_string(), vec![
                    TreeNode::from_pattern(pattern),
                    TreeNode::from_expr(value),
                    TreeNode::from_expr(body),
                ])
            }
            
            Expr::If { condition, then_branch, else_branch, .. } => {
                TreeNode::with_children("If".to_string(), vec![
                    TreeNode::from_expr(condition),
                    TreeNode::from_expr(then_branch),
                    TreeNode::from_expr(else_branch),
                ])
            }
            
            Expr::Match { scrutinee, arms, .. } => {
                let mut children = vec![TreeNode::from_expr(scrutinee)];
                for arm in arms {
                    children.push(TreeNode::with_children("Arm".to_string(), vec![
                        TreeNode::from_pattern(&arm.pattern),
                        TreeNode::from_expr(&arm.body),
                    ]));
                }
                TreeNode::with_children("Match".to_string(), children)
            }
            
            _ => TreeNode::new("Unknown".to_string()),
        }
    }
    
    /// Convert pattern to tree node
    pub fn from_pattern(pattern: &Pattern) -> Self {
        match pattern {
            Pattern::Variable(name, _) => {
                TreeNode::new(format!("PatVar:{}", name.as_str()))
            }
            Pattern::Wildcard(_) => {
                TreeNode::new("Wildcard".to_string())
            }
            Pattern::Literal(lit, _) => {
                let label = match lit {
                    Literal::Integer(n) => format!("PatInt:{n}"),
                    Literal::Float(f) => format!("PatFloat:{f}"),
                    Literal::String(s) => format!("PatString:{s}"),
                    Literal::Bool(b) => format!("PatBool:{b}"),
                    Literal::Unit => "PatUnit".to_string(),
                };
                TreeNode::new(label)
            }
            Pattern::Constructor { name, args, .. } => {
                let mut children = vec![TreeNode::new(format!("Ctor:{}", name.as_str()))];
                children.extend(args.iter().map(TreeNode::from_pattern));
                TreeNode::with_children("PatConstructor".to_string(), children)
            }
            Pattern::Tuple { patterns, .. } => {
                let children = patterns.iter().map(TreeNode::from_pattern).collect();
                TreeNode::with_children("PatTuple".to_string(), children)
            }
            _ => TreeNode::new("PatUnknown".to_string()),
        }
    }
    
    /// Get all subtrees (for all-path computation)
    pub fn all_subtrees(&self) -> Vec<&TreeNode> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.all_subtrees());
        }
        result
    }
    
    /// Get size (number of nodes)
    pub fn size(&self) -> usize {
        1 + self.children.iter().map(|c| c.size()).sum::<usize>()
    }
    
    /// Get depth
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }
}

/// APTED (All Path Tree Edit Distance) implementation
#[derive(Clone)]
pub struct APTED {
    /// Cost for node insertion
    pub insert_cost: f64,
    /// Cost for node deletion
    pub delete_cost: f64,
    /// Cost for node relabeling
    pub rename_cost: f64,
}

impl Default for APTED {
    fn default() -> Self {
        Self {
            insert_cost: 1.0,
            delete_cost: 1.0,
            rename_cost: 1.0,
        }
    }
}

impl APTED {
    /// Compute APTED distance between two trees
    pub fn distance(&self, tree1: &TreeNode, tree2: &TreeNode) -> f64 {
        // Get all subtrees
        let subtrees1 = tree1.all_subtrees();
        let subtrees2 = tree2.all_subtrees();
        
        let n1 = subtrees1.len();
        let n2 = subtrees2.len();
        
        // DP table for tree edit distance
        let mut dp = vec![vec![0.0; n2 + 1]; n1 + 1];
        
        // Initialize base cases
        for i in 1..=n1 {
            dp[i][0] = i as f64 * self.delete_cost;
        }
        for j in 1..=n2 {
            dp[0][j] = j as f64 * self.insert_cost;
        }
        
        // Fill DP table
        for i in 1..=n1 {
            for j in 1..=n2 {
                let t1 = subtrees1[i - 1];
                let t2 = subtrees2[j - 1];
                
                // Cost of operations
                let delete = dp[i - 1][j] + self.delete_cost;
                let insert = dp[i][j - 1] + self.insert_cost;
                let rename = if t1.label == t2.label {
                    dp[i - 1][j - 1]
                } else {
                    dp[i - 1][j - 1] + self.rename_cost
                };
                
                dp[i][j] = delete.min(insert).min(rename);
            }
        }
        
        dp[n1][n2]
    }
    
    /// Compute normalized similarity (0.0 to 1.0)
    pub fn similarity(&self, tree1: &TreeNode, tree2: &TreeNode) -> f64 {
        let distance = self.distance(tree1, tree2);
        let max_size = tree1.size().max(tree2.size()) as f64;
        
        if max_size == 0.0 {
            1.0
        } else {
            1.0 - (distance / max_size).min(1.0)
        }
    }
}

/// TSED (Tree Structure Edit Distance) implementation
/// Focuses on structural similarity, ignoring labels
#[derive(Clone)]
pub struct TSED {
    /// Weight for structural difference
    pub structure_weight: f64,
    /// Weight for depth difference
    pub depth_weight: f64,
    /// Weight for branching factor difference
    pub branching_weight: f64,
}

impl Default for TSED {
    fn default() -> Self {
        Self {
            structure_weight: 0.5,
            depth_weight: 0.3,
            branching_weight: 0.2,
        }
    }
}

impl TSED {
    /// Compute structural features of a tree
    fn compute_features(&self, tree: &TreeNode) -> TreeFeatures {
        let mut features = TreeFeatures {
            depth: tree.depth(),
            size: tree.size(),
            branching_factors: Vec::new(),
            shape_vector: Vec::new(),
        };
        
        // Compute branching factors at each level
        self.compute_branching_factors(tree, &mut features.branching_factors, 0);
        
        // Compute shape vector
        self.compute_shape_vector(tree, &mut features.shape_vector);
        
        features
    }
    
    fn compute_branching_factors(&self, tree: &TreeNode, factors: &mut Vec<f64>, level: usize) {
        if factors.len() <= level {
            factors.resize(level + 1, 0.0);
        }
        
        factors[level] += 1.0;
        
        for child in &tree.children {
            self.compute_branching_factors(child, factors, level + 1);
        }
    }
    
    fn compute_shape_vector(&self, tree: &TreeNode, shape: &mut Vec<usize>) {
        shape.push(tree.children.len());
        for child in &tree.children {
            self.compute_shape_vector(child, shape);
        }
    }
    
    /// Compute TSED distance
    pub fn distance(&self, tree1: &TreeNode, tree2: &TreeNode) -> f64 {
        let features1 = self.compute_features(tree1);
        let features2 = self.compute_features(tree2);
        
        // Depth difference
        let depth_diff = (features1.depth as f64 - features2.depth as f64).abs()
            / (features1.depth.max(features2.depth) as f64).max(1.0);
        
        // Size difference
        let size_diff = (features1.size as f64 - features2.size as f64).abs()
            / (features1.size.max(features2.size) as f64).max(1.0);
        
        // Branching factor difference
        let branching_diff = self.branching_factor_distance(
            &features1.branching_factors,
            &features2.branching_factors,
        );
        
        // Weighted combination
        self.structure_weight * size_diff +
        self.depth_weight * depth_diff +
        self.branching_weight * branching_diff
    }
    
    fn branching_factor_distance(&self, bf1: &[f64], bf2: &[f64]) -> f64 {
        let max_len = bf1.len().max(bf2.len());
        if max_len == 0 {
            return 0.0;
        }
        
        let mut sum = 0.0;
        for i in 0..max_len {
            let v1 = bf1.get(i).copied().unwrap_or(0.0);
            let v2 = bf2.get(i).copied().unwrap_or(0.0);
            sum += (v1 - v2).abs();
        }
        
        sum / max_len as f64
    }
    
    /// Compute normalized similarity
    pub fn similarity(&self, tree1: &TreeNode, tree2: &TreeNode) -> f64 {
        1.0 - self.distance(tree1, tree2)
    }
}

/// Tree features for TSED
struct TreeFeatures {
    depth: usize,
    size: usize,
    branching_factors: Vec<f64>,
    shape_vector: Vec<usize>,
}

/// Combined similarity using multiple algorithms
#[derive(Clone)]
pub struct CombinedSimilarity {
    pub apted: APTED,
    pub tsed: TSED,
    pub apted_weight: f64,
    pub tsed_weight: f64,
}

impl Default for CombinedSimilarity {
    fn default() -> Self {
        Self {
            apted: APTED::default(),
            tsed: TSED::default(),
            apted_weight: 0.7,
            tsed_weight: 0.3,
        }
    }
}

impl CombinedSimilarity {
    /// Compute combined similarity
    pub fn similarity(&self, tree1: &TreeNode, tree2: &TreeNode) -> f64 {
        let apted_sim = self.apted.similarity(tree1, tree2);
        let tsed_sim = self.tsed.similarity(tree1, tree2);
        
        self.apted_weight * apted_sim + self.tsed_weight * tsed_sim
    }
    
    /// Get detailed similarity report
    pub fn detailed_similarity(&self, tree1: &TreeNode, tree2: &TreeNode) -> SimilarityReport {
        SimilarityReport {
            apted_similarity: self.apted.similarity(tree1, tree2),
            tsed_similarity: self.tsed.similarity(tree1, tree2),
            combined_similarity: self.similarity(tree1, tree2),
            tree1_size: tree1.size(),
            tree2_size: tree2.size(),
            tree1_depth: tree1.depth(),
            tree2_depth: tree2.depth(),
        }
    }
}

/// Detailed similarity report
#[derive(Debug, Clone)]
pub struct SimilarityReport {
    pub apted_similarity: f64,
    pub tsed_similarity: f64,
    pub combined_similarity: f64,
    pub tree1_size: usize,
    pub tree2_size: usize,
    pub tree1_depth: usize,
    pub tree2_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{FileId, Span, span::ByteOffset, Symbol};
    
    fn make_span() -> Span {
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1))
    }
    
    #[test]
    fn test_tree_node_creation() {
        let expr = Expr::App(
            Box::new(Expr::Var(Symbol::intern("+"), make_span())),
            vec![
                Expr::Literal(Literal::Integer(1), make_span()),
                Expr::Literal(Literal::Integer(2), make_span()),
            ],
            make_span(),
        );
        
        let tree = TreeNode::from_expr(&expr);
        assert_eq!(tree.label, "App");
        assert_eq!(tree.children.len(), 3);
        assert_eq!(tree.size(), 4);
        assert_eq!(tree.depth(), 2);
    }
    
    #[test]
    fn test_apted_identical_trees() {
        let tree1 = TreeNode::with_children("root".to_string(), vec![
            TreeNode::new("a".to_string()),
            TreeNode::new("b".to_string()),
        ]);
        let tree2 = tree1.clone();
        
        let apted = APTED::default();
        assert_eq!(apted.distance(&tree1, &tree2), 0.0);
        assert_eq!(apted.similarity(&tree1, &tree2), 1.0);
    }
    
    #[test]
    fn test_tsed_structure() {
        let tree1 = TreeNode::with_children("root".to_string(), vec![
            TreeNode::new("a".to_string()),
            TreeNode::new("b".to_string()),
        ]);
        
        let tree2 = TreeNode::with_children("different".to_string(), vec![
            TreeNode::new("x".to_string()),
            TreeNode::new("y".to_string()),
        ]);
        
        let tsed = TSED::default();
        // Should have high similarity because structure is same
        assert!(tsed.similarity(&tree1, &tree2) > 0.8);
    }
}