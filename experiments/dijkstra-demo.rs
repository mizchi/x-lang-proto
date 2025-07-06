use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct Edge {
    to: usize,
    weight: u32,
}

#[derive(Debug)]
struct Graph {
    nodes: Vec<Vec<Edge>>,
}

impl Graph {
    fn new(size: usize) -> Self {
        Graph {
            nodes: vec![vec![]; size],
        }
    }
    
    fn add_edge(&mut self, from: usize, to: usize, weight: u32) {
        self.nodes[from].push(Edge { to, weight });
    }
    
    fn add_undirected_edge(&mut self, a: usize, b: usize, weight: u32) {
        self.add_edge(a, b, weight);
        self.add_edge(b, a, weight);
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct State {
    cost: u32,
    node: usize,
}

// Reverse ordering for min-heap
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct PathResult {
    distance: u32,
    path: Vec<usize>,
}

fn dijkstra(graph: &Graph, start: usize, target: usize) -> Option<PathResult> {
    let n = graph.nodes.len();
    let mut distances = vec![u32::MAX; n];
    let mut previous = vec![None; n];
    let mut heap = BinaryHeap::new();
    
    distances[start] = 0;
    heap.push(State { cost: 0, node: start });
    
    while let Some(State { cost, node }) = heap.pop() {
        // If we reached the target, we can stop early
        if node == target {
            break;
        }
        
        // If we found a longer path to this node, skip it
        if cost > distances[node] {
            continue;
        }
        
        // Check all neighbors
        for edge in &graph.nodes[node] {
            let next = State {
                cost: cost + edge.weight,
                node: edge.to,
            };
            
            // If we found a shorter path, update it
            if next.cost < distances[edge.to] {
                distances[edge.to] = next.cost;
                previous[edge.to] = Some(node);
                heap.push(next);
            }
        }
    }
    
    // Reconstruct path if target was reached
    if distances[target] == u32::MAX {
        None
    } else {
        let mut path = vec![];
        let mut current = Some(target);
        
        while let Some(node) = current {
            path.push(node);
            current = previous[node];
        }
        
        path.reverse();
        
        Some(PathResult {
            distance: distances[target],
            path,
        })
    }
}

fn create_sample_graph() -> Graph {
    // Create the same graph as in the x Language implementation
    //     1 --- 2
    //    / \   / \
    //   0   \ /   3
    //    \   4   /
    //     \ / \ /
    //      5   
    
    let mut graph = Graph::new(6);
    
    // Add all edges (undirected)
    graph.add_undirected_edge(0, 1, 4);
    graph.add_undirected_edge(0, 5, 8);
    graph.add_undirected_edge(1, 2, 8);
    graph.add_undirected_edge(1, 4, 11);
    graph.add_undirected_edge(1, 5, 7);
    graph.add_undirected_edge(2, 3, 2);
    graph.add_undirected_edge(2, 4, 4);
    graph.add_undirected_edge(3, 4, 9);
    graph.add_undirected_edge(4, 5, 1);
    
    graph
}

fn main() {
    println!("Dijkstra's Algorithm Demo");
    println!("========================\n");
    
    let graph = create_sample_graph();
    
    // Test cases from the x Language implementation
    let test_cases = vec![
        (0, 3, 14, vec![0, 1, 2, 3]),  // 0 -> 3: distance 14
        (0, 4, 9, vec![0, 5, 4]),       // 0 -> 4: distance 9
        (2, 5, 5, vec![2, 4, 5]),       // 2 -> 5: distance 5
        (1, 1, 0, vec![1]),             // 1 -> 1: distance 0
    ];
    
    for (start, target, expected_dist, expected_path) in test_cases {
        print!("Finding shortest path from {} to {}: ", start, target);
        
        match dijkstra(&graph, start, target) {
            Some(result) => {
                println!("Distance = {}", result.distance);
                println!("  Path: {:?}", result.path);
                
                // Verify the result
                let correct = result.distance == expected_dist && result.path == expected_path;
                println!("  Test: {}", if correct { "✓ PASSED" } else { "✗ FAILED" });
                
                if !correct {
                    println!("  Expected: distance = {}, path = {:?}", expected_dist, expected_path);
                }
            }
            None => {
                println!("No path found!");
                println!("  Test: ✗ FAILED");
            }
        }
        println!();
    }
    
    // Test unreachable node
    println!("Testing unreachable node:");
    let mut disconnected = Graph::new(3);
    disconnected.add_undirected_edge(0, 1, 1);
    // Node 2 is isolated
    
    print!("Finding path from 0 to 2 in disconnected graph: ");
    match dijkstra(&disconnected, 0, 2) {
        Some(_) => println!("Found path (unexpected!)"),
        None => println!("No path found ✓"),
    }
    
    println!("\nVisualizing the graph:");
    println!("     1 --- 2");
    println!("    /4\\8  /8\\2");
    println!("   0   \\ /   3");
    println!("  8 \\11 4  9/");
    println!("     \\ /1\\ /");
    println!("      5");
    println!("\nEdge weights are shown on the edges.");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dijkstra_basic() {
        let graph = create_sample_graph();
        
        // Test 0 -> 3
        let result = dijkstra(&graph, 0, 3).unwrap();
        assert_eq!(result.distance, 14);
        assert_eq!(result.path, vec![0, 1, 2, 3]);
        
        // Test 0 -> 4
        let result = dijkstra(&graph, 0, 4).unwrap();
        assert_eq!(result.distance, 9);
        assert_eq!(result.path, vec![0, 5, 4]);
        
        // Test 2 -> 5
        let result = dijkstra(&graph, 2, 5).unwrap();
        assert_eq!(result.distance, 5);
        assert_eq!(result.path, vec![2, 4, 5]);
    }
    
    #[test]
    fn test_dijkstra_same_node() {
        let graph = create_sample_graph();
        let result = dijkstra(&graph, 1, 1).unwrap();
        assert_eq!(result.distance, 0);
        assert_eq!(result.path, vec![1]);
    }
    
    #[test]
    fn test_dijkstra_unreachable() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1);
        graph.add_edge(1, 0, 1);
        // Node 2 is isolated
        
        assert!(dijkstra(&graph, 0, 2).is_none());
    }
}