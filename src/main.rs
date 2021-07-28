use serde_json::json;
use reqwest::blocking;
use std::thread;
use std::time::Duration;
use crate::sensors::get_sensor_data;
use crate::cpu::get_cpu_data;
use crate::gpu::get_gpu_data;
use crate::memory::get_memory_data;

mod sensors;
mod cpu;
mod gpu;
mod memory;
mod utils;

fn main() {
    loop {
        let mut sensor_data = get_sensor_data();

        let request_url = format!("http://sensor-relay.int.mindphaser.se/publish");

        let cpu_data = get_cpu_data();
        sensor_data.extend(cpu_data);

        let gpu_data = get_gpu_data();
        println!("GPU data: {:?}", gpu_data);
        sensor_data.extend(gpu_data);

        let memory_data = get_memory_data();
        println!("Memory data: {:?}", memory_data);
        sensor_data.extend(memory_data);

        let register_body = json!({
            "reporter": "linux-sensor-agent",
            "sensors": sensor_data,
            "topic": "sensors"
        });

        blocking::Client::new()
            .post(request_url)
            .json(&register_body)
            .send()
            .unwrap();

        thread::sleep(Duration::from_secs(1));
    }
}
