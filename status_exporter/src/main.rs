use std::thread;
use std::time::Duration;
use clap::Parser;
use ovpn_status_lib::process_status_file;
use clokwerk::{Scheduler, TimeUnits};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File path of status file
    filename: String,
}

fn process_status(filename: &str){
    println!("{:?}", process_status_file(filename).snapshot_time.unwrap().snapshot_time_t)
}

fn main() {
    let cli = Cli::parse();
    let mut scheduler = Scheduler::new();
    scheduler.every(1.minutes()).run(move || process_status(&cli.filename));
    let _thread_handle = scheduler.watch_thread(Duration::from_millis(100));
    loop {
        thread::sleep(Duration::from_millis(200));
    }

    println!("Hello, world! {:?}", cli.filename);
}

