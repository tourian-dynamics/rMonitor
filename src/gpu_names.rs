//! pulse: library-specific GPU names lookup with discrete-first sort.
//!
//! Kept app-specific because library's `query_gpu_names` returns a flat
//! `Vec<String>` without pulse's "discrete GPUs first" reordering.

pub fn get_gpu_names_sorted() -> Vec<String> {
    let mut gpu_names = library::platform::native::sys_info::query_gpu_names();
    gpu_names.sort_by(|a, b| {
        let is_discrete_a = a.contains("RX") || a.contains("RTX") || a.contains("GTX") || a.contains("NVIDIA");
        let is_discrete_b = b.contains("RX") || b.contains("RTX") || b.contains("GTX") || b.contains("NVIDIA");
        is_discrete_b.cmp(&is_discrete_a)
    });
    gpu_names
}
