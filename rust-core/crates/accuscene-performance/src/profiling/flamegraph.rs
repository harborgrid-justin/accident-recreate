//! Flamegraph generation support

use std::collections::HashMap;
use std::time::Duration;

/// Flamegraph sample
#[derive(Debug, Clone)]
pub struct FlameSample {
    /// Stack trace (from root to leaf)
    pub stack: Vec<String>,
    /// Duration in microseconds
    pub duration_us: u64,
}

/// Flamegraph generator
pub struct FlamegraphGenerator {
    samples: Vec<FlameSample>,
}

impl FlamegraphGenerator {
    /// Create a new flamegraph generator
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    /// Add a sample
    pub fn add_sample(&mut self, stack: Vec<String>, duration: Duration) {
        self.samples.push(FlameSample {
            stack,
            duration_us: duration.as_micros() as u64,
        });
    }

    /// Generate folded format (for flamegraph.pl)
    pub fn to_folded(&self) -> String {
        let mut folded_map: HashMap<String, u64> = HashMap::new();

        for sample in &self.samples {
            let stack_str = sample.stack.join(";");
            *folded_map.entry(stack_str).or_insert(0) += sample.duration_us;
        }

        let mut lines: Vec<String> = folded_map
            .into_iter()
            .map(|(stack, count)| format!("{} {}", stack, count))
            .collect();

        lines.sort();
        lines.join("\n")
    }

    /// Generate SVG (simplified version)
    pub fn to_svg(&self) -> String {
        // This is a simplified placeholder
        // In production, you'd use a proper flamegraph library
        format!(
            "<svg>
    <text>Flamegraph with {} samples</text>
</svg>",
            self.samples.len()
        )
    }

    /// Get aggregated tree
    pub fn aggregate_tree(&self) -> FlameNode {
        let mut root = FlameNode::new("root".to_string());

        for sample in &self.samples {
            root.add_sample(&sample.stack, sample.duration_us);
        }

        root
    }

    /// Get total duration
    pub fn total_duration(&self) -> Duration {
        let total_us: u64 = self.samples.iter().map(|s| s.duration_us).sum();
        Duration::from_micros(total_us)
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Clear all samples
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\nFlamegraph Summary:");
        println!("Total samples: {}", self.sample_count());
        println!("Total duration: {:?}", self.total_duration());

        let tree = self.aggregate_tree();
        tree.print(0);
    }
}

impl Default for FlamegraphGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Flamegraph tree node
#[derive(Debug, Clone)]
pub struct FlameNode {
    /// Function name
    pub name: String,
    /// Self time in microseconds
    pub self_time_us: u64,
    /// Total time in microseconds
    pub total_time_us: u64,
    /// Child nodes
    pub children: HashMap<String, FlameNode>,
}

impl FlameNode {
    /// Create a new flame node
    pub fn new(name: String) -> Self {
        Self {
            name,
            self_time_us: 0,
            total_time_us: 0,
            children: HashMap::new(),
        }
    }

    /// Add a sample to the tree
    pub fn add_sample(&mut self, stack: &[String], duration_us: u64) {
        self.total_time_us += duration_us;

        if stack.is_empty() {
            self.self_time_us += duration_us;
            return;
        }

        let child_name = &stack[0];
        let child = self
            .children
            .entry(child_name.clone())
            .or_insert_with(|| FlameNode::new(child_name.clone()));

        child.add_sample(&stack[1..], duration_us);
    }

    /// Print the tree
    pub fn print(&self, depth: usize) {
        let indent = "  ".repeat(depth);
        println!(
            "{}{}  self: {}µs  total: {}µs",
            indent, self.name, self.self_time_us, self.total_time_us
        );

        let mut children: Vec<_> = self.children.values().collect();
        children.sort_by(|a, b| b.total_time_us.cmp(&a.total_time_us));

        for child in children {
            child.print(depth + 1);
        }
    }

    /// Get self percentage
    pub fn self_percent(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            (self.self_time_us as f64 / self.total_time_us as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flamegraph_generator() {
        let mut generator = FlamegraphGenerator::new();

        generator.add_sample(
            vec!["main".to_string(), "foo".to_string()],
            Duration::from_micros(100),
        );

        generator.add_sample(
            vec!["main".to_string(), "bar".to_string()],
            Duration::from_micros(200),
        );

        assert_eq!(generator.sample_count(), 2);
        assert_eq!(generator.total_duration(), Duration::from_micros(300));
    }

    #[test]
    fn test_folded_format() {
        let mut generator = FlamegraphGenerator::new();

        generator.add_sample(
            vec!["main".to_string(), "foo".to_string()],
            Duration::from_micros(100),
        );

        let folded = generator.to_folded();
        assert!(folded.contains("main;foo"));
    }

    #[test]
    fn test_flame_tree() {
        let mut root = FlameNode::new("root".to_string());

        root.add_sample(&vec!["foo".to_string(), "bar".to_string()], 100);
        root.add_sample(&vec!["foo".to_string(), "baz".to_string()], 200);

        assert_eq!(root.total_time_us, 300);
        assert_eq!(root.children.len(), 1);

        let foo = &root.children["foo"];
        assert_eq!(foo.total_time_us, 300);
        assert_eq!(foo.children.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut generator = FlamegraphGenerator::new();

        generator.add_sample(vec!["main".to_string()], Duration::from_micros(100));
        assert_eq!(generator.sample_count(), 1);

        generator.clear();
        assert_eq!(generator.sample_count(), 0);
    }
}
