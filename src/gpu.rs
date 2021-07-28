use std::collections::HashMap;
use std::fs::read_to_string;
use crate::utils::extract_values;

pub fn get_gpu_data() -> HashMap<String, String>{

    let pm_info = read_to_string("/sys/kernel/debug/dri/0/amdgpu_pm_info");
    return match pm_info {
        Ok(pm_info) => parse_pm_info(pm_info),
        Err(_) => HashMap::new()
    }
}

fn parse_pm_info(pm_info: String) -> HashMap<String, String> {
    let mut value_map = HashMap::<&str, &str>::new();
    let mut postprocessors = HashMap::<&str, Box<dyn Fn(String) -> String>>::new();

    value_map.insert("gpu_mem_frequency", "\\s*([0-9]+) MHz \\(MCLK\\).*");
    value_map.insert("gpu_frequency", "\\s*([0-9]+) MHz \\(SCLK\\).*");
    value_map.insert("gpu_voltage", "\\s*([0-9.]+) mV \\(VDDGFX\\).*");
    value_map.insert("gpu_power", "\\s*([0-9.]+) W \\(average GPU\\).*");
    value_map.insert("gpu_utilization", "GPU Load:\\s*([0-9.]+) %");

    postprocessors.insert("gpu_voltage", Box::from(|str: String| -> String {
        let val = str.parse::<f64>().unwrap();
        return (val / 1000.0).to_string();
    }));

    extract_values(pm_info, value_map, postprocessors)
}