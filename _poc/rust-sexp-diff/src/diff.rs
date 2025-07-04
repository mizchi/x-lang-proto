//! High-performance structural diff for S-expressions
//! 
//! Implements Myers algorithm for computing minimal edit distance

use crate::sexp::{SExp, Atom};
use std::fmt;
use std::collections::HashMap;

/// Diff operation types
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    /// Keep element unchanged
    Keep(SExp),
    /// Insert new element
    Insert(SExp),
    /// Delete existing element
    Delete(SExp),
    /// Replace element with another
    Replace(SExp, SExp),
}

/// Diff result with path information
#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    /// Path to the changed element in the AST
    pub path: Vec<usize>,
    /// The diff operation
    pub operation: DiffOp,
}

impl DiffResult {
    pub fn new(path: Vec<usize>, operation: DiffOp) -> Self {
        DiffResult { path, operation }
    }
}

/// High-performance structural diff engine
pub struct StructuralDiff {
    /// Whether to include unchanged elements in output
    include_unchanged: bool,
    /// Maximum depth for diff computation (prevents stack overflow)
    max_depth: usize,
}

impl StructuralDiff {
    /// Create a new structural diff engine
    pub fn new() -> Self {
        StructuralDiff {
            include_unchanged: false,
            max_depth: 1000,
        }
    }

    /// Configure whether to include unchanged elements
    pub fn include_unchanged(mut self, include: bool) -> Self {
        self.include_unchanged = include;
        self
    }

    /// Configure maximum diff depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Compute structural diff between two S-expressions
    pub fn diff(&self, left: &SExp, right: &SExp) -> Vec<DiffResult> {
        let mut results = Vec::new();
        self.diff_recursive(left, right, &mut Vec::new(), &mut results, 0);
        results
    }

    fn diff_recursive(
        &self,
        left: &SExp,
        right: &SExp,
        path: &mut Vec<usize>,
        results: &mut Vec<DiffResult>,
        depth: usize,
    ) {
        if depth > self.max_depth {
            // Fallback to simple replace at max depth
            results.push(DiffResult::new(path.clone(), DiffOp::Replace(left.clone(), right.clone())));
            return;
        }

        if self.sexp_equal(left, right) {
            if self.include_unchanged {
                results.push(DiffResult::new(path.clone(), DiffOp::Keep(left.clone())));
            }
            return;
        }

        match (left, right) {
            (SExp::List(left_elements), SExp::List(right_elements)) => {
                self.diff_lists(left_elements, right_elements, path, results, depth);
            }
            _ => {
                results.push(DiffResult::new(path.clone(), DiffOp::Replace(left.clone(), right.clone())));
            }
        }
    }

    fn diff_lists(
        &self,
        left: &[SExp],
        right: &[SExp],
        path: &mut Vec<usize>,
        results: &mut Vec<DiffResult>,
        depth: usize,
    ) {
        let diff_ops = self.myers_diff(left, right);
        let mut left_idx = 0;
        let mut right_idx = 0;

        for op in diff_ops {
            match op {
                MyersOp::Keep => {
                    path.push(left_idx);
                    self.diff_recursive(&left[left_idx], &right[right_idx], path, results, depth + 1);
                    path.pop();
                    left_idx += 1;
                    right_idx += 1;
                }
                MyersOp::Delete => {
                    path.push(left_idx);
                    results.push(DiffResult::new(path.clone(), DiffOp::Delete(left[left_idx].clone())));
                    path.pop();
                    left_idx += 1;
                }
                MyersOp::Insert => {
                    path.push(right_idx);
                    results.push(DiffResult::new(path.clone(), DiffOp::Insert(right[right_idx].clone())));
                    path.pop();
                    right_idx += 1;
                }
            }
        }
    }

    fn sexp_equal(&self, left: &SExp, right: &SExp) -> bool {
        match (left, right) {
            (SExp::Atom(a1), SExp::Atom(a2)) => self.atom_equal(a1, a2),
            (SExp::Symbol(s1), SExp::Symbol(s2)) => s1 == s2,
            (SExp::List(l1), SExp::List(l2)) => {
                l1.len() == l2.len() && l1.iter().zip(l2.iter()).all(|(e1, e2)| self.sexp_equal(e1, e2))
            }
            _ => false,
        }
    }

    fn atom_equal(&self, a1: &Atom, a2: &Atom) -> bool {
        match (a1, a2) {
            (Atom::String(s1), Atom::String(s2)) => s1 == s2,
            (Atom::Integer(i1), Atom::Integer(i2)) => i1 == i2,
            (Atom::Float(f1), Atom::Float(f2)) => (f1 - f2).abs() < f64::EPSILON,
            (Atom::Boolean(b1), Atom::Boolean(b2)) => b1 == b2,
            _ => false,
        }
    }

    /// Myers algorithm for computing optimal edit sequence
    fn myers_diff(&self, left: &[SExp], right: &[SExp]) -> Vec<MyersOp> {
        let n = left.len();
        let m = right.len();
        let max_d = n + m;

        let mut v: HashMap<isize, isize> = HashMap::new();
        v.insert(1, 0);

        let mut trace = Vec::new();

        for d in 0..=max_d {
            let mut current_v = v.clone();
            
            for k in (-(d as isize)..=(d as isize)).step_by(2) {
                let x = if k == -(d as isize) || (k != d as isize && v.get(&(k - 1)).unwrap_or(&-1) < v.get(&(k + 1)).unwrap_or(&-1)) {
                    *v.get(&(k + 1)).unwrap_or(&0)
                } else {
                    v.get(&(k - 1)).unwrap_or(&0) + 1
                };

                let mut y = x - k;
                
                while (x as usize) < n && (y as usize) < m && self.sexp_equal(&left[x as usize], &right[y as usize]) {
                    y += 1;
                }

                current_v.insert(k, x + (y - x));

                if (x + (y - x)) as usize >= n && y as usize >= m {
                    trace.push(current_v.clone());
                    return self.backtrack(&trace, left, right, n, m);
                }
            }
            
            trace.push(current_v.clone());
            v = current_v;
        }

        // Fallback: should not reach here with correct implementation
        Vec::new()
    }

    fn backtrack(
        &self,
        trace: &[HashMap<isize, isize>],
        left: &[SExp],
        right: &[SExp],
        n: usize,
        m: usize,
    ) -> Vec<MyersOp> {
        let mut x = n as isize;
        let mut y = m as isize;
        let mut operations = Vec::new();

        for d in (0..trace.len()).rev() {
            let v = &trace[d];
            let k = x - y;

            let prev_k = if k == -(d as isize) || (k != d as isize && 
                v.get(&(k - 1)).unwrap_or(&-1) < v.get(&(k + 1)).unwrap_or(&-1)) {
                k + 1
            } else {
                k - 1
            };

            let prev_x = *v.get(&prev_k).unwrap_or(&0);
            let prev_y = prev_x - prev_k;

            while x > prev_x && y > prev_y {
                operations.push(MyersOp::Keep);
                x -= 1;
                y -= 1;
            }

            if d > 0 {
                if x > prev_x {
                    operations.push(MyersOp::Delete);
                    x -= 1;
                } else {
                    operations.push(MyersOp::Insert);
                    y -= 1;
                }
            }
        }

        operations.reverse();
        operations
    }
}

impl Default for StructuralDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum MyersOp {
    Keep,
    Delete,
    Insert,
}

/// Diff formatter for colored output
pub struct DiffFormatter {
    pub use_color: bool,
    pub compact: bool,
    pub show_paths: bool,
}

impl DiffFormatter {
    pub fn new() -> Self {
        DiffFormatter {
            use_color: true,
            compact: false,
            show_paths: true,
        }
    }

    pub fn no_color(mut self) -> Self {
        self.use_color = false;
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn hide_paths(mut self) -> Self {
        self.show_paths = false;
        self
    }

    pub fn format(&self, results: &[DiffResult]) -> String {
        let mut output = String::new();
        
        for result in results {
            if self.compact && matches!(result.operation, DiffOp::Keep(_)) {
                continue;
            }

            let line = self.format_result(result);
            output.push_str(&line);
            output.push('\n');
        }

        output
    }

    fn format_result(&self, result: &DiffResult) -> String {
        let path_str = if self.show_paths {
            format!("@{} ", self.format_path(&result.path))
        } else {
            String::new()
        };

        match &result.operation {
            DiffOp::Keep(sexp) => {
                format!("  {} {}", self.format_sexp(sexp), path_str)
            }
            DiffOp::Insert(sexp) => {
                let formatted = self.format_sexp(sexp);
                if self.use_color {
                    format!("\x1b[32m+ {} {}\x1b[0m", formatted, path_str)
                } else {
                    format!("+ {} {}", formatted, path_str)
                }
            }
            DiffOp::Delete(sexp) => {
                let formatted = self.format_sexp(sexp);
                if self.use_color {
                    format!("\x1b[31m- {} {}\x1b[0m", formatted, path_str)
                } else {
                    format!("- {} {}", formatted, path_str)
                }
            }
            DiffOp::Replace(old, new) => {
                let old_formatted = self.format_sexp(old);
                let new_formatted = self.format_sexp(new);
                if self.use_color {
                    format!(
                        "\x1b[31m- {} {}\x1b[0m\n\x1b[32m+ {} {}",
                        old_formatted, path_str, new_formatted, path_str
                    )
                } else {
                    format!("- {} {}\n+ {} {}", old_formatted, path_str, new_formatted, path_str)
                }
            }
        }
    }

    fn format_path(&self, path: &[usize]) -> String {
        path.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    fn format_sexp(&self, sexp: &SExp) -> String {
        match sexp {
            SExp::Atom(atom) => format!("{}", atom),
            SExp::Symbol(symbol) => symbol.clone(),
            SExp::List(elements) => {
                if elements.is_empty() {
                    "()".to_string()
                } else {
                    format!(
                        "({})",
                        elements
                            .iter()
                            .map(|e| self.format_sexp(e))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
            }
        }
    }
}

impl Default for DiffFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics for diff results
#[derive(Debug, Clone, PartialEq)]
pub struct DiffSummary {
    pub insertions: usize,
    pub deletions: usize,
    pub replacements: usize,
    pub unchanged: usize,
}

impl DiffSummary {
    pub fn from_results(results: &[DiffResult]) -> Self {
        let mut summary = DiffSummary {
            insertions: 0,
            deletions: 0,
            replacements: 0,
            unchanged: 0,
        };

        for result in results {
            match &result.operation {
                DiffOp::Keep(_) => summary.unchanged += 1,
                DiffOp::Insert(_) => summary.insertions += 1,
                DiffOp::Delete(_) => summary.deletions += 1,
                DiffOp::Replace(_, _) => summary.replacements += 1,
            }
        }

        summary
    }
}

impl fmt::Display for DiffSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Summary: {} insertions, {} deletions, {} replacements",
            self.insertions, self.deletions, self.replacements
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_expressions() {
        let sexp1 = SExp::List(vec![
            SExp::Symbol("+".to_string()),
            SExp::Atom(Atom::Integer(1)),
            SExp::Atom(Atom::Integer(2)),
        ]);
        let sexp2 = sexp1.clone();

        let diff = StructuralDiff::new();
        let results = diff.diff(&sexp1, &sexp2);
        assert!(results.is_empty());
    }

    #[test]
    fn test_simple_replacement() {
        let sexp1 = SExp::Atom(Atom::Integer(1));
        let sexp2 = SExp::Atom(Atom::Integer(2));

        let diff = StructuralDiff::new();
        let results = diff.diff(&sexp1, &sexp2);
        
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].operation, DiffOp::Replace(_, _)));
    }

    #[test]
    fn test_list_diff() {
        let sexp1 = SExp::List(vec![
            SExp::Symbol("if".to_string()),
            SExp::List(vec![
                SExp::Symbol("=".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::Atom(Atom::Integer(0)),
            ]),
            SExp::Atom(Atom::Integer(1)),
        ]);

        let sexp2 = SExp::List(vec![
            SExp::Symbol("if".to_string()),
            SExp::List(vec![
                SExp::Symbol("<=".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::Atom(Atom::Integer(1)),
            ]),
            SExp::Atom(Atom::Integer(1)),
        ]);

        let diff = StructuralDiff::new();
        let results = diff.diff(&sexp1, &sexp2);
        
        // Should find replacements in the nested condition
        assert!(!results.is_empty());
        let summary = DiffSummary::from_results(&results);
        assert!(summary.replacements > 0);
    }

    #[test]
    fn test_diff_formatter() {
        let results = vec![
            DiffResult::new(vec![0], DiffOp::Keep(SExp::Symbol("if".to_string()))),
            DiffResult::new(vec![1, 0], DiffOp::Replace(
                SExp::Symbol("=".to_string()),
                SExp::Symbol("<=".to_string()),
            )),
        ];

        let formatter = DiffFormatter::new().no_color();
        let output = formatter.format(&results);
        
        assert!(output.contains("if"));
        assert!(output.contains("="));
        assert!(output.contains("<="));
    }

    #[test]
    fn test_diff_summary() {
        let results = vec![
            DiffResult::new(vec![0], DiffOp::Keep(SExp::Symbol("keep".to_string()))),
            DiffResult::new(vec![1], DiffOp::Insert(SExp::Symbol("insert".to_string()))),
            DiffResult::new(vec![2], DiffOp::Delete(SExp::Symbol("delete".to_string()))),
            DiffResult::new(vec![3], DiffOp::Replace(
                SExp::Symbol("old".to_string()),
                SExp::Symbol("new".to_string()),
            )),
        ];

        let summary = DiffSummary::from_results(&results);
        assert_eq!(summary.unchanged, 1);
        assert_eq!(summary.insertions, 1);
        assert_eq!(summary.deletions, 1);
        assert_eq!(summary.replacements, 1);
    }
}