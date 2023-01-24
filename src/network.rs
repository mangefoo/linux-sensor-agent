use std::fs;
use std::collections::HashMap;
use regex::Regex;

pub fn get_network_data() -> HashMap<String, String> {
    let proc_net_dev = fs::read_to_string("/proc/net/dev");

    return match proc_net_dev {
        Ok(proc_net_dev) => parse_net_dev(proc_net_dev),
        Err(_) => HashMap::new()
    }
}

fn parse_net_dev(proc_meminfo: String) -> HashMap<String, String> {
    let mut values = HashMap::<String, String>::new();

    // enp6s0: 1436460811 1044283    0    0    0     0          0      4504 24663248  226471    0    0    0     0       0          0
    let re = Regex::new("enp5s0:\\s+([^\\s]+)\\s+[^\\s]+\\s+[^\\s]+\\s+[^\\s]+\\s+[^\\s]+\\s+[^\\s]+\\s+[^\\s]+\\s+[^\\s]+\\s+([^\\s]+)").unwrap();
    for line in proc_meminfo.split("\n") {
        let capture = re.captures(line);
        if capture.is_some() {
            let captures = capture.unwrap();

            values.insert("network_name_1".to_string(), "ethernet".to_string());
            values.insert("network_received_bytes_1".to_string(), captures.get(1).unwrap().as_str().to_string());
            values.insert("network_sent_bytes_1".to_string(), captures.get(2).unwrap().as_str().to_string());
        }
    }

    return values;
}