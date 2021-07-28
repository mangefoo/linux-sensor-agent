use std::fs;
use std::collections::HashMap;
use crate::utils::extract_values;

pub fn get_memory_data() -> HashMap<String, String> {
    let proc_meminfo = fs::read_to_string("/proc/meminfo");

    return match proc_meminfo {
        Ok(proc_meminfo) => parse_meminfo(proc_meminfo),
        Err(_) => HashMap::new()
    }
}

fn parse_meminfo(proc_meminfo: String) -> HashMap<String, String> {
    let mut value_map = HashMap::<&str, &str>::new();
    let mut postprocessors = HashMap::<&str, Box<dyn Fn(String) -> String>>::new();

    value_map.insert("mem_total", "MemTotal:\\s*([0-9]+) kB.*");
    value_map.insert("mem_free", "MemFree:\\s*([0-9]+) kB.*");
    value_map.insert("mem_available", "MemAvailable:\\s*([0-9]+) kB.*");

    let kb_to_gb = |str: String| -> String {
        let val = str.parse::<f64>().unwrap();
        return (val / 1024.0 / 1024.0).to_string();
    };

    postprocessors.insert("mem_total", Box::from(kb_to_gb));
    postprocessors.insert("mem_free", Box::from(kb_to_gb));
    postprocessors.insert("mem_available", Box::from(kb_to_gb));

    return extract_values(proc_meminfo, value_map, postprocessors);
}