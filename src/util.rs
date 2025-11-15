use time::{OffsetDateTime, macros::{format_description, offset}};

static DATETIME_FMT: &[time::format_description::FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

pub fn get_time() -> String {
      let utc = OffsetDateTime::now_utc();
      let local = utc.to_offset(offset!(+7));
      
      return local.format(DATETIME_FMT).unwrap().to_string();
}

pub fn log_something(scope_title: &str, message: &str) {
      println!("[{}] [{}] {}", get_time(), scope_title, message);
}