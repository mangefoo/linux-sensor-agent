use std::fs;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use regex::Regex;

pub fn get_cpu_data() -> HashMap<String, String> {
    let mut proc_stat = unsafe { get_proc_stat() };
    proc_stat.extend(get_proc_cpuinfo());

    return proc_stat;
}

lazy_static! {
    static ref PROC_STAT_PREVIOUS: Mutex<HashMap<String, (u64, u64)>> = {
        Mutex::new(HashMap::new())
    };
}

// Extract CPU frequency values to cpu_core_frequency_<core number>
fn get_proc_cpuinfo() -> HashMap<String, String> {
    let mut cpu_values = HashMap::<String, String>::new();
    let proc_cpuinfo = fs::read_to_string("/proc/cpuinfo").expect("Failed to read /proc/cpuinfo");

    let lines: Vec<(&str, &str)> = proc_cpuinfo.split("\n")
        .filter(|line| line.contains(":"))
        .map(|line| {
            return line.split_once(":")
                .map(|key_value| (key_value.0.trim(), key_value.1.trim()))
                .unwrap();
        })
        .collect();

    let mut core_id = 0;
    for line in lines {
        match line.0 {
            "core id" => core_id = line.1.parse::<u32>().unwrap(),
            "cpu MHz" => {
                let key = format!("cpu_core_frequency_{}", core_id + 1).to_string();
                if !cpu_values.contains_key(&key) {
                    cpu_values.insert(key, line.1.to_string());
                }
            }
            _ => {}
        }
    }

    return cpu_values;
}

unsafe fn get_proc_stat() -> HashMap<String, String> {
    let mut state: HashMap<String, (u64, u64)> = HashMap::new();
    let mut utilization: HashMap<String, u64> = HashMap::new();
    let mut previous_values = PROC_STAT_PREVIOUS.lock().unwrap();

    let proc_stat = fs::read_to_string("/proc/stat").expect("Failed to read /proc/stat");
    let split_lines: Vec<String> = proc_stat.split("\n").map(|v| v.to_string()).collect();

    let cpu_line: Vec<Vec<String>> = proc_stat.split("\n")
        .map(|v| v.to_string())
        .filter(|line| line.starts_with("cpu "))
        .map(|line| String::from({
            let re = Regex::new(r"[ ]+").unwrap();
            re.replace_all(&line, " ")
        }))
        .map(|line| {
            return line.split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
        })
        .collect();

    let vcpu_lines: Vec<Vec<String>> = split_lines.into_iter()
        .filter(|line| line.starts_with("cpu") && !line.starts_with("cpu "))
        .map(|line| String::from({
            let re = Regex::new(r"[ ]+").unwrap();
            re.replace_all(&line, " ")
        }))
        .map(|line|
            line.split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>())
        .collect();

    for line in vcpu_lines {
        let mut total: u64 = 0;
        let (key, values) = line.split_first().unwrap();
        for parts in values {
            total += parts.to_string().parse::<u64>().unwrap();
        }

        let idle: u64 = values[3].parse().unwrap();

        state.insert(key.to_string(), (total, idle));
        if previous_values.contains_key(key) {
            let total_delta = total - previous_values[key].0;
            let idle_delta = idle - previous_values[key].1;
            utilization.insert(key.to_string(), 100 - (idle_delta * 100) / (total_delta));
        }
    }

    let (_, values) = cpu_line.first().unwrap().split_first().unwrap();
    let mut total = 0;
    for parts in values {
        total += parts.parse::<u64>().unwrap();
    }
    let idle = values[3].parse::<u64>().unwrap();

    state.insert("cpu".to_string(), (total, idle));

    if previous_values.contains_key("cpu") {
        let total_delta = total - previous_values["cpu"].0;
        let idle_delta = idle - previous_values["cpu"].1;
        utilization.insert("cpu".to_string(), 100 - (idle_delta * 100) / total_delta);
    }

    previous_values.clear();
    previous_values.extend(state);

    let mut sensor_values = HashMap::<String, String>::new();
    for key in 0..=15 {
        if utilization.contains_key(&format!("cpu{}", key * 2)) {
            let virtual_1 = utilization[&format!("cpu{}", key * 2)].clone();
            let virtual_2 = utilization[&format!("cpu{}", key * 2 + 1)].clone();

            let core_average = (virtual_1 + virtual_2) / 2;
            sensor_values.insert(format!("cpu_core_load_{}", key + 1), core_average.to_string());
        }
    }

    if utilization.contains_key("cpu") {
        sensor_values.insert("cpu_utilization".to_string(), utilization["cpu"].to_string());
    }

    return sensor_values;
}