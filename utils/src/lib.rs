pub mod error;
pub mod file;
pub mod log;
pub mod sha256;
pub mod time;
pub mod secp;

pub fn get_u8_vec_sum(vec: &[u8]) -> u64 {
    vec.iter().map(|x| u64::from(*x)).sum()
}
