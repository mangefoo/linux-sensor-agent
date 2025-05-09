use std::collections::HashMap;
use sensors::{Sensors, Chip, Feature, LibsensorsError};
use regex::Regex;

pub fn get_sensor_data(debug: bool) -> HashMap<String, String> {
    let mut sensor_map = HashMap::new();
    sensor_map.insert("amdgpu-pci-1000.*/edge/temp[0-9]{1}_input", "gpu_edge_temp");
    sensor_map.insert("amdgpu-pci-1000.*/junction/temp[0-9]{1}_input", "gpu_junction_temp");
    sensor_map.insert("amdgpu-pci-1000.*/mem/temp[0-9]{1}_input", "gpu_mem_temp");
    sensor_map.insert("k10temp-.*/Tctl/temp[0-9]{1}_input", "cpu_temp");
    sensor_map.insert("corsaircpro-.*/fan3 4pin/fan[0-9]{1}_input", "fan1_rpm");
    sensor_map.insert("corsaircpro-.*/fan4 4pin/fan[0-9]{1}_input", "fan2_rpm");
    sensor_map.insert("corsaircpro-.*/fan5 4pin/fan[0-9]{1}_input", "fan3_rpm");
    sensor_map.insert("corsaircpro-.*/fan1 4pin/fan[0-9]{1}_input", "fan4_rpm");
    sensor_map.insert("corsaircpro-.*/fan2 4pin/fan[0-9]{1}_input", "fan5_rpm");
    sensor_map.insert("corsaircpro-.*/temp1/temp[0-9]{1}_input", "pump_temp");
    sensor_map.insert("corsaircpro-.*/temp2/temp[0-9]{1}_input", "exhaust_temp");
    sensor_map.insert("corsaircpro-.*/temp3/temp[0-9]{1}_input", "front_intake_temp");
    sensor_map.insert("corsaircpro-.*/temp4/temp[0-9]{1}_input", "ambient_temp");
    sensor_map.insert("nct6798-isa-0290/fan2/fan[0-9]", "cpu_rpm");

    let sensor_values = get_sensor_values(&sensor_map, debug);

    return sensor_values;
}

fn get_sensor_values(sensor_map: &HashMap<&str, &str>, debug: bool) -> HashMap<String, String> {
    let sensors = Sensors::new();

    if debug {
        println!("Dump:");
        for chip in sensors {
            println!("  {}, {:?}", chip.get_name().unwrap(), chip);
            for feature in chip {
                println!("    {}, {:?}", feature.get_label().unwrap(), feature);
            }
        }
    }

    let mut sensor_values = HashMap::new();

    for map in sensor_map {
        let parts = map.0.split_once("/").unwrap();
        let re = Regex::new(parts.0).unwrap();
        for chip in sensors {
            if re.is_match(&chip.get_name().unwrap()) {
                sensor_values.insert(map.1.to_string(), get_chip_value(chip, parts.1));
            }
        }
    }

    return sensor_values;
}

fn get_chip_value(chip: Chip, path: &str) -> String {
    let parts = path.split_once("/").unwrap();
    let re = Regex::new(parts.0).unwrap();
    for feature in chip {
        if re.is_match(&feature.get_label().unwrap()) {
            return get_feature_value(feature, parts.1);
        }
    }

    return "".to_string();
}

fn get_feature_value(feature: Feature, path: &str) -> String {
    let re = Regex::new(path).unwrap();

    for subfeature in feature {
        if re.is_match(&subfeature.name()) {
            return format!("{:.2}", subfeature.get_value()
                .or_else(|err: LibsensorsError| {
                    println!("Failed to get value {}: {:?}", subfeature.name(), err);
                    return Ok::<f64, LibsensorsError>(0.0);
                })
                .unwrap());
        }
    }

    return "".to_string();
}
