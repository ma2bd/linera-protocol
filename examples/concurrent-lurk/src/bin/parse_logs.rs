use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    pub total_time: Duration,
    pub count: usize,
    pub mean_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
}

impl TimingStats {
    fn new() -> Self {
        Self {
            total_time: Duration::new(0, 0),
            count: 0,
            mean_time: Duration::new(0, 0),
            min_time: Duration::MAX,
            max_time: Duration::new(0, 0),
        }
    }

    fn add_timing(&mut self, duration: Duration) {
        self.total_time += duration;
        self.count += 1;

        if duration < self.min_time {
            self.min_time = duration;
        }

        if duration > self.max_time {
            self.max_time = duration;
        }

        self.mean_time = self.total_time / self.count as u32;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBenchmark {
    pub client_id: usize,
    pub operations: HashMap<String, TimingStats>,
}

impl ClientBenchmark {
    fn new(client_id: usize) -> Self {
        Self {
            client_id,
            operations: HashMap::new(),
        }
    }

    pub fn generate_markdown_table(&self) -> String {
        let mut markdown = String::new();

        // Table header
        markdown.push_str("| Operation | Total Time | Count | Mean Time | Min Time | Max Time |\n");
        markdown.push_str("|-----------|------------|-------|-----------|----------|----------|\n");

        // Table rows
        let mut sorted_ops: Vec<(&String, &TimingStats)> = self.operations.iter().collect();
        sorted_ops.sort_by(|a, b| a.0.cmp(b.0));

        for (operation, stats) in sorted_ops {
            markdown.push_str(&format!(
                "| {} | {:.2}s | {} | {:.2}s | {:.2}s | {:.2}s |\n",
                operation,
                stats.total_time.as_secs_f64(),
                stats.count,
                stats.mean_time.as_secs_f64(),
                stats.min_time.as_secs_f64(),
                stats.max_time.as_secs_f64()
            ));
        }

        markdown
    }
}

fn parse_logs_and_update_benchmark(
    log_content: &str,
    benchmark: &mut ClientBenchmark,
) -> Result<(), String> {
    let timestamp_pattern =
        Regex::new(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+INFO.*>>> (START|END) (\w+)")
            .map_err(|e| e.to_string())?;

    let mut operation_start_times: HashMap<String, DateTime<Utc>> = HashMap::new();

    for line in log_content.lines() {
        if let Some(captures) = timestamp_pattern.captures(line) {
            let timestamp_str = captures.get(1).unwrap().as_str();
            let action = captures.get(2).unwrap().as_str();
            let operation = captures.get(3).unwrap().as_str().to_string();

            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .map_err(|e| format!("Failed to parse timestamp: {}", e))?
                .with_timezone(&Utc);

            match action {
                "START" => {
                    operation_start_times.insert(operation, timestamp);
                }
                "END" => {
                    if let Some(start_time) = operation_start_times.remove(&operation) {
                        let duration = timestamp
                            .signed_duration_since(start_time)
                            .to_std()
                            .map_err(|e| format!("Duration conversion error: {}", e))?;

                        // Update benchmark stats
                        let stats = benchmark
                            .operations
                            .entry(operation)
                            .or_insert_with(TimingStats::new);
                        stats.add_timing(duration);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <log_file_path> <benchmark_markdown_path> [client_id]",
            args[0]
        );
        std::process::exit(1);
    }

    let log_file_path = &args[1];
    let benchmark_markdown_path = &args[2];
    let client_id = args
        .get(3)
        .map(|s| s.parse::<usize>().unwrap_or(1))
        .unwrap_or(1);

    // Read log file
    let log_content = fs::read_to_string(log_file_path)?;

    // Create or read existing benchmark
    let mut benchmark = ClientBenchmark::new(client_id);

    // Update benchmark with parsed logs
    parse_logs_and_update_benchmark(&log_content, &mut benchmark)?;

    // Write updated benchmark back to file
    let markdown = benchmark.generate_markdown_table();
    fs::write(benchmark_markdown_path, markdown)?;

    println!("Successfully updated benchmark at {}", benchmark_markdown_path);

    Ok(())
}
