use std::time::UNIX_EPOCH;

pub fn now_unix() -> u64 {
    UNIX_EPOCH.elapsed().expect("Time went backwards").as_secs()
}
