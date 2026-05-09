use std::process::Command;
use std::time::Instant;

#[test]
fn test_search_performance_gate() {
    let binary_path = env!("CARGO_BIN_EXE_changeguard");

    // 1. Initial indexing
    let start = Instant::now();
    let status = Command::new(binary_path)
        .args(&["search", "struct", "--index", "--limit", "1"])
        .status()
        .expect("Failed to execute changeguard search");
    assert!(status.success());
    let indexing_duration = start.elapsed();
    println!("Indexing took: {:?}", indexing_duration);

    // 2. Ranked search performance
    let start = Instant::now();
    let output = Command::new(binary_path)
        .args(&["search", "TantivySearchEngine", "--limit", "10"])
        .output()
        .expect("Failed to execute changeguard search");
    assert!(output.status.success());
    let ranked_duration = start.elapsed();
    println!("Ranked search took: {:?}", ranked_duration);
    assert!(ranked_duration.as_millis() < 500, "Ranked search too slow: {:?}", ranked_duration);

    // 3. Regex search performance
    let start = Instant::now();
    let output = Command::new(binary_path)
        .args(&["search", "pub fn.*\\{", "--regex", "--limit", "10"])
        .output()
        .expect("Failed to execute changeguard search");
    assert!(output.status.success());
    let regex_duration = start.elapsed();
    println!("Regex search took: {:?}", regex_duration);
    assert!(regex_duration.as_millis() < 500, "Regex search too slow: {:?}", regex_duration);
}
