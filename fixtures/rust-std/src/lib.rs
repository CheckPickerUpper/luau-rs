//! Standard-library fixture for the end-to-end compiler contract.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[used]
static HEAP_PADDING: [u8; 131_072] = [0; 131_072];

#[derive(Default)]
struct ConstantHasher;

impl Hasher for ConstantHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

/// Runs the standard-library operations behind a non-inlined boundary so the
/// generated Luau keeps each Rust function within Luau's local-register limit.
#[inline(never)]
fn standard_library_body() -> i32 {
    let values = vec![3, 5, 7];
    let name = String::from("luau");
    let formatted = format!("{name}:{}", values.len());
    let key = formatted.len() as i32;

    let mut scores: HashMap<i32, i32, BuildHasherDefault<ConstantHasher>> =
        HashMap::with_capacity_and_hasher(1, BuildHasherDefault::default());
    scores.insert(key, 29);

    let lookup = scores.get(&key).copied().unwrap_or_default();
    lookup + values.iter().sum::<i32>()
}

/// Uses allocation, a growable collection, formatting, and a hash-map lookup.
#[unsafe(no_mangle)]
pub extern "C" fn standard_library_score() -> i32 {
    standard_library_body()
}
