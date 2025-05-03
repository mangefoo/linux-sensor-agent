use clap::{Command, Arg};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use signal_hook::{consts::SIGUSR1, iterator::Signals};

use reqwest::blocking;
use serde_json::json;

use crate::config::Config;
use crate::cpu::get_cpu_data;
use crate::gpu::get_nvidia_gpu_data;
use crate::memory::get_memory_data;
use crate::network::get_network_data;
use crate::sensors::get_sensor_data;

mod config;
mod sensors;
mod cpu;
mod gpu;
mod memory;
mod utils;
mod network;

struct State {
    debug: bool,
    config: Config
}
fn main() {
    let matches = Command::new("Linux Sensor Agent")
        .args(&[Arg::new("configpath")
            .short('c')
            .long("configfile")
            .value_name("CONFIG_PATH")])
        .get_matches();

    let config_path = match matches.get_one::<String>("configpath") {
        None => "config.toml",
        Some(s) => s.as_str()
    };
    println!("Using config file: {}", config_path);
    let config = config::read_config(config_path);

    let mut last_network_data = HashMap::<String, String>::new();
    let mut last_sample = Instant::now();
    let state = Arc::new(Mutex::new(State { config: config, debug: false }));

    signals_init(&state);

    loop {
        let mut sensor_data = get_sensor_data(state.lock().unwrap().debug);

        let cpu_data = get_cpu_data();
        sensor_data.extend(cpu_data);

        let nvidia_gpu_data = get_nvidia_gpu_data();
        if nvidia_gpu_data.len() > 0 {
            sensor_data.insert("gpu_vendor".to_string(), "NVIDIA".to_string());
            sensor_data.insert("gpu_model".to_string(), "RTX 5080".to_string());
            sensor_data.extend(nvidia_gpu_data);
        }

        let amd_gpu_data = gpu::get_amd_gpu_data();
        if amd_gpu_data.len() > 0 {
            sensor_data.insert("gpu_vendor".to_string(), "Ryzen".to_string());
            sensor_data.insert("gpu_model".to_string(), "RX 6600".to_string());
            sensor_data.extend(amd_gpu_data);
        }

        let memory_data = get_memory_data();
        sensor_data.extend(memory_data);

        let network_data = get_network_data(state.lock().unwrap().config.network_interface.clone());
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
            "reporter": state.lock().unwrap().config.reporter,
            "sensors": sensor_data,
            "topic": "sensors"
        });

        let post_response = blocking::Client::new()
            .post(state.lock().unwrap().config.publish_url.clone())
            .json(&register_body)
            .timeout(Duration::from_secs(2))
            .send();

        if post_response.is_err() {
            println!("Failed to send update to server: {}", post_response.unwrap_err())
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn signals_init(state: &Arc<Mutex<State>>) {
    let mut signals = Signals::new(&[SIGUSR1]).unwrap();

    let c_state = state.clone();
    thread::spawn(move || {
        for _sig in signals.forever() {
            let mut state = c_state.lock().unwrap();
            state.debug = !state.debug;
            println!("Toggling USR1 debug");
        }
    });
}