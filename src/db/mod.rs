pub mod actions;
pub mod models;
pub mod schema;

// https://stackoverflow.com/a/74491572
pub fn wrap_to_u64(x: i64) -> u64 {
    (x as u64).wrapping_add(u64::MAX / 2 + 1)
}
pub fn wrap_to_i64(x: u64) -> i64 {
    x.wrapping_sub(u64::MAX / 2 + 1) as i64
}
