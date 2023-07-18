use chrono::prelude::*;
use std::process::Command;

fn main() {
    let local: DateTime<Local> = Local::now();
    let day = local.weekday().number_from_monday();
    let backup_file = format!("db-{}.back", day);
    let output = Command::new("pg_dump")
        .arg("-f")
        .arg(backup_file)
        .arg("www")
        .output()
        .expect("Failed to execute command");

    println!(" day {} backup ok", day)
}
