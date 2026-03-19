use std::io::Write;

pub fn init_logger() {
    let mut builder = env_logger::Builder::new();

    if let Ok(level) = std::env::var("UQ_LOG_LEVEL") {
        builder.parse_filters(&level);
    } else if let Ok(level) = std::env::var("RUST_LOG") {
        builder.parse_filters(&level);
    } else {
        builder.parse_filters("warn");
    }

    builder.format(|buf, record| {
        writeln!(
            buf,
            "[{} {:>5}] {}",
            buf.timestamp_millis(),
            record.level(),
            record.args()
        )
    });

    let _ = builder.try_init();
}
