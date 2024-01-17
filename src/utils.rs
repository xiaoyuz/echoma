use std::io;
use std::time::Duration;

const TIMESTAMP_FORMAT: &str = "%Y/%m/%d %H:%M:%S%.3f %:z";

pub fn timestamp_local(io: &mut dyn io::Write) -> io::Result<()> {
    let now = chrono::Local::now().format(TIMESTAMP_FORMAT);
    write!(io, "{now}")
}

pub fn current_time_mills() -> i64 {
    chrono::Local::now().timestamp_millis()
}

pub async fn sleep(ms: u32) {
    tokio::time::sleep(Duration::from_millis(ms as u64)).await;
}
