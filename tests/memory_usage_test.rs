//! Memory usage tests to detect memory leaks
//!
//! These tests verify that repeated POT token generation does not cause
//! excessive memory accumulation.

use bgutil_ytdlp_pot_provider::{config::Settings, session::SessionManager, types::PotRequest};

/// Get current process memory usage in bytes
fn get_memory_usage() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/self/status
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    // VmRSS is in kB, convert to bytes
                    return parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
                }
            }
        }
        None
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Fallback: use ps command for cross-platform support
        use std::process::Command;

        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let rss_kb = output_str.trim().parse::<usize>().ok()?;
            // RSS is in kB, convert to bytes
            Some(rss_kb * 1024)
        } else {
            None
        }
    }
}

/// Format bytes to human-readable format
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[tokio::test]
async fn test_memory_usage_over_multiple_requests() {
    // Initialize tracing for debugging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let settings = Settings::default();
    let manager = SessionManager::new(settings);

    // Video ID from the issue: dQw4w9WgXcQ
    let video_id = "dQw4w9WgXcQ";

    // Get initial memory usage
    let mem_start = get_memory_usage();
    println!("Initial memory: {:?}", mem_start.map(format_bytes));

    // Warmup: Generate first token (this may allocate initial structures)
    let request = PotRequest::new().with_content_binding(video_id);
    let result = manager.generate_pot_token(&request).await;
    assert!(result.is_ok(), "First token generation should succeed");

    // Force garbage collection if possible
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mem_after_warmup = get_memory_usage();
    println!(
        "Memory after warmup: {:?}",
        mem_after_warmup.map(format_bytes)
    );

    // Now test repeated generations with DIFFERENT content bindings to avoid cache hits
    let num_iterations = 10;
    let mut memory_samples = Vec::new();

    for i in 0..num_iterations {
        // Use bypass_cache to force new token generation every time
        let request = PotRequest::new()
            .with_content_binding(video_id)
            .with_bypass_cache(true);

        let result = manager.generate_pot_token(&request).await;
        assert!(result.is_ok(), "Token generation {} should succeed", i + 1);

        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Sample memory usage
        if let Some(mem) = get_memory_usage() {
            memory_samples.push(mem);
            println!("Iteration {}: Memory = {}", i + 1, format_bytes(mem));
        }
    }

    // Get final memory usage
    let mem_end = get_memory_usage();
    println!("Final memory: {:?}", mem_end.map(format_bytes));

    // Analyze memory growth
    if let (Some(start), Some(end)) = (mem_after_warmup, mem_end) {
        let growth = end as i64 - start as i64;
        let growth_mb = growth as f64 / (1024.0 * 1024.0);

        println!("\n=== Memory Analysis ===");
        println!("Start (after warmup): {}", format_bytes(start));
        println!(
            "End (after {} iterations): {}",
            format_bytes(end),
            num_iterations
        );
        println!("Growth: {:.2} MB", growth_mb);

        // Calculate average growth per iteration
        if !memory_samples.is_empty() {
            let first = memory_samples[0];
            let last = memory_samples[memory_samples.len() - 1];
            let iter_growth = (last as i64 - first as i64) as f64 / memory_samples.len() as f64;
            let iter_growth_mb = iter_growth / (1024.0 * 1024.0);
            println!("Average growth per iteration: {:.2} MB", iter_growth_mb);

            // If memory growth is excessive (more than 10MB per iteration), the test should fail
            // This threshold is conservative - the issue reported ~300MB after a few runs
            let max_growth_per_iter_mb = 10.0;
            assert!(
                iter_growth_mb < max_growth_per_iter_mb,
                "Memory growth per iteration ({:.2} MB) exceeds threshold ({:.2} MB). \
                 This indicates a memory leak!",
                iter_growth_mb,
                max_growth_per_iter_mb
            );
        }

        // Overall growth should be reasonable (less than 100MB for 10 iterations)
        let max_total_growth_mb = 100.0;
        assert!(
            growth_mb < max_total_growth_mb,
            "Total memory growth ({:.2} MB) exceeds threshold ({:.2} MB). \
             This indicates a memory leak!",
            growth_mb,
            max_total_growth_mb
        );
    } else {
        println!("Warning: Could not measure memory usage on this platform");
        // Don't fail the test if we can't measure memory
    }
}

#[tokio::test]
async fn test_memory_usage_with_cache_invalidation() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let settings = Settings::default();
    let manager = SessionManager::new(settings);

    let video_id = "dQw4w9WgXcQ";

    // Get initial memory
    let mem_start = get_memory_usage();
    println!("Initial memory: {:?}", mem_start.map(format_bytes));

    // Generate tokens and periodically invalidate caches
    let num_iterations = 10;
    let mut memory_samples = Vec::new();

    for i in 0..num_iterations {
        let request = PotRequest::new().with_content_binding(video_id);
        let result = manager.generate_pot_token(&request).await;
        assert!(result.is_ok(), "Token generation {} should succeed", i);

        // Every 3 iterations, invalidate caches to test cleanup
        if i % 3 == 2 {
            manager.invalidate_caches().await.ok();
            println!("Caches invalidated at iteration {}", i + 1);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        if let Some(mem) = get_memory_usage() {
            memory_samples.push(mem);
            println!("Iteration {}: Memory = {}", i + 1, format_bytes(mem));
        }
    }

    let mem_end = get_memory_usage();
    println!("Final memory: {:?}", mem_end.map(format_bytes));

    // With cache invalidation, memory should be more stable
    if let (Some(start), Some(end)) = (mem_start, mem_end) {
        let growth = end as i64 - start as i64;
        let growth_mb = growth as f64 / (1024.0 * 1024.0);

        println!("\n=== Memory Analysis (with cache invalidation) ===");
        println!("Growth: {:.2} MB", growth_mb);

        // With regular cache cleanup, growth should be even more controlled
        let max_growth_mb = 80.0;
        assert!(
            growth_mb < max_growth_mb,
            "Memory growth with cache cleanup ({:.2} MB) exceeds threshold ({:.2} MB)",
            growth_mb,
            max_growth_mb
        );
    }
}

#[tokio::test]
#[ignore] // This is a stress test, run manually with: cargo test --test memory_usage_test -- --ignored
async fn test_memory_usage_stress() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .try_init();

    let settings = Settings::default();
    let manager = SessionManager::new(settings);

    let video_id = "dQw4w9WgXcQ";

    println!("Starting stress test with 50 iterations...");
    let mem_start = get_memory_usage();
    println!("Initial memory: {:?}", mem_start.map(format_bytes));

    // Stress test with many iterations
    let num_iterations = 50;

    for i in 0..num_iterations {
        let request = PotRequest::new()
            .with_content_binding(video_id)
            .with_bypass_cache(true);

        let result = manager.generate_pot_token(&request).await;
        assert!(result.is_ok(), "Token generation {} should succeed", i);

        if (i + 1) % 10 == 0 {
            if let Some(mem) = get_memory_usage() {
                println!("After {} iterations: {}", i + 1, format_bytes(mem));
            }
        }

        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }

    let mem_end = get_memory_usage();
    println!("Final memory: {:?}", mem_end.map(format_bytes));

    if let (Some(start), Some(end)) = (mem_start, mem_end) {
        let growth = end as i64 - start as i64;
        let growth_mb = growth as f64 / (1024.0 * 1024.0);

        println!("\n=== Stress Test Results ===");
        println!(
            "Total growth: {:.2} MB over {} iterations",
            growth_mb, num_iterations
        );
        println!(
            "Average per iteration: {:.2} MB",
            growth_mb / num_iterations as f64
        );

        // For 50 iterations, we should not see more than 200MB growth
        // (The issue reported ~300MB after "a few runs", so this is a conservative limit)
        assert!(
            growth_mb < 200.0,
            "Memory growth in stress test ({:.2} MB) indicates a leak",
            growth_mb
        );
    }
}
