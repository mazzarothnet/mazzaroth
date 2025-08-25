pub mod error;
pub mod file;
pub mod log;
pub mod secp;
pub mod sha256;
pub mod time;
pub mod mutex_log;

pub fn get_u8_vec_sum(vec: &[u8]) -> u64 {
    vec.iter().map(|x| u64::from(*x)).sum()
}

// #[cfg(not(target_env = "msvc"))]
// use jemallocator::Jemalloc;
// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;
