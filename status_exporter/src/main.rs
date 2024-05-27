use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;
use clap::Parser;
use ovpn_status_lib::process_status_file;
use clokwerk::{Scheduler, TimeUnits};
use lazy_static::lazy_static;
use prometheus::{GaugeVec, register_gauge_vec, Encoder};

lazy_static!{
    static ref A_INT_GAUGE_VEC: GaugeVec = register_gauge_vec!("vpn_client_connected_since", "Time connected in epoch time", &["common_name", "virtual_address"]).unwrap();
    static ref BYTES_RECEIVED_GAUGE_VEC: GaugeVec = register_gauge_vec!("vpn_bytes_received", "Bytes received from client since connection", &["common_name", "virtual_address"]).unwrap();
    static ref BYTES_SENT_GAUGE_VEC: GaugeVec = register_gauge_vec!("vpn_bytes_sent", "Bytes sent to client since connection", &["common_name", "virtual_address"]).unwrap();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File path of status file
    filename: String,
    #[arg(short,long,default_value_t = String::from("ovpn_metrics"))]
    output_filename: String
}

fn process_status(filename: &str){
    let status_info = process_status_file(filename);
    println!("{:#?}", status_info.client_list);
    for client in status_info.client_list {
        A_INT_GAUGE_VEC.with_label_values(&[&*client.common_name, &*client.virtual_address.to_string()]).set(client.connected_since_time_t as f64);
        BYTES_RECEIVED_GAUGE_VEC.with_label_values(&[&*client.common_name, &*client.virtual_address.to_string()]).set(client.bytes_received as f64);
        BYTES_SENT_GAUGE_VEC.with_label_values(&[&*client.common_name, &*client.virtual_address.to_string()]).set(client.bytes_sent as f64);
    }

}

fn main() {
    let cli = Cli::parse();
    let mut buffer = Vec::new();
    let encoder = prometheus::TextEncoder::new();
    let mut scheduler = Scheduler::new();
    scheduler.every(1.minutes()).run(move || process_status(&cli.filename));
    let _thread_handle = scheduler.watch_thread(Duration::from_millis(100));
    loop {
        thread::sleep(Duration::from_millis(1000));
        let metric_families = prometheus::gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let mut file = File::create("ovpn_metrics").unwrap();
        file.write_all(String::from_utf8(buffer.clone()).unwrap().as_bytes()).unwrap();
        buffer.clear();
    }
}

