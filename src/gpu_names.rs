//! pulse: library-specific GPU names lookup with discrete-first sort.
//!
//! Kept app-specific because library's `query_gpu_names` returns a flat
//! `Vec<String>` without pulse's "discrete GPUs first" reordering.

/// Sorts GPU names placing discrete models (NVIDIA, RTX, RX, GTX) first
pub fn sort_gpu_names(mut names: Vec<String>) -> Vec<String> {
    names.sort_by(|a, b| {
        let is_discrete_a = a.contains("RX") || a.contains("RTX") || a.contains("GTX") || a.contains("NVIDIA");
        let is_discrete_b = b.contains("RX") || b.contains("RTX") || b.contains("GTX") || b.contains("NVIDIA");
        is_discrete_b.cmp(&is_discrete_a)
    });
    names
}

pub fn get_gpu_names_sorted() -> Vec<String> {
    let gpu_names = crate::backend::sys_info::query_gpu_names();
    sort_gpu_names(gpu_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_gpu_names() {
        let gpus = vec![
            "Intel(R) UHD Graphics".to_string(),
            "NVIDIA GeForce RTX 4070 Laptop GPU".to_string(),
            "AMD Radeon Graphics".to_string(),
            "AMD Radeon RX 6800 XT".to_string(),
        ];
        let sorted = sort_gpu_names(gpus);
        assert_eq!(sorted[0], "NVIDIA GeForce RTX 4070 Laptop GPU");
        assert_eq!(sorted[1], "AMD Radeon RX 6800 XT");
        assert_eq!(sorted[2], "Intel(R) UHD Graphics");
        assert_eq!(sorted[3], "AMD Radeon Graphics");
    }
}
