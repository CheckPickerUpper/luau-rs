//! Standard-library fixture for the end-to-end compiler contract.

use std::collections::HashMap;

/// Uses allocation, a growable collection, formatting, and a hash-map lookup.
#[unsafe(no_mangle)]
pub extern "C" fn standard_library_score() -> i32 {
    let values = vec![3, 5, 7];
    let name = String::from("luau");
    let formatted = format!("{name}:{}", values.len());

    let mut scores = HashMap::new();
    scores.insert(formatted, 29);

    let lookup = scores.get("luau:3").copied().unwrap_or_default();
    lookup + values.iter().sum::<i32>()
}
