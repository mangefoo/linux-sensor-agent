use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use reqwest::blocking;
use serde_json::json;

use crate::cpu::get_cpu_data;
use crate::gpu::get_gpu_data;
use crate::memory::get_memory_data;
use crate::network::get_network_data;
use crate::sensors::get_sensor_data;

mod sensors;
mod cpu;
mod gpu;
mod memory;
mod utils;
mod network;

fn main() {
    let mut last_network_data = HashMap::<String, String>::new();
    let mut last_sample = Instant::now();

    loop {
        let mut sensor_data = get_sensor_data();

        let request_url = format!("http://sensor-relay.int.mindphaser.se/publish");

        let cpu_data = get_cpu_data();
        sensor_data.extend(cpu_data);

        let gpu_data = get_gpu_data();
        sensor_data.extend(gpu_data);

        let memory_data = get_memory_data();
        sensor_data.extend(memory_data);

        let network_data = get_network_data();
        if last_network_data.keys().len() > 0 {
            let last_received = last_network_data.get("network_received_bytes_1").unwrap().parse::<i64>().unwrap();
            let last_sent = last_network_data.get("network_sent_bytes_1").unwrap().parse::<i64>().unwrap();
            let received  = network_data.get("network_received_bytes_1").unwrap().parse::<i64>().unwrap();
            let sent = network_data.get("network_sent_bytes_1").unwrap().parse::<i64>().unwrap();

            let received_delta = (received - last_received) * 1000 / last_sample.elapsed().as_millis() as i64;
            let sent_delta = (sent - last_sent) * 1000 / last_sample.elapsed().as_millis() as i64;

            sensor_data.insert("network_name_1".to_string(), "ethernet".to_string());
            sensor_data.insert("network_sent_bytes_1".to_string(), format!("{}", sent_delta));
            sensor_data.insert("network_received_bytes_1".to_string(), format!("{}", received_delta));
        }
        last_sample = Instant::now();
        last_network_data = network_data.clone();

        let register_body = json!({
            "reporter": "linux-sensor-agent",
            "sensors": sensor_data,
            "topic": "sensors"
        });

        let post_response = blocking::Client::new()
            .post(request_url)
            .json(&register_body)
            .send();

        if post_response.is_err() {
            println!("Failed to send update to server: {}", post_response.unwrap_err())
        }

        thread::sleep(Duration::from_secs(1));
    }
}
