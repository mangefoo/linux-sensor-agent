use std::collections::HashMap;
use regex::Regex;

pub fn extract_values(content: String, value_map: HashMap<&str, &str>,
                  postprocessors: HashMap<&str, Box<dyn Fn(String) -> String>>) -> HashMap<String, String> {
    let mut values: HashMap<String, String> = HashMap::<String, String>::new();

    for line in content.split("\n") {
        for map_entry in &value_map {
            let re = Regex::new(map_entry.1).unwrap();
            let capture = re.captures(line);
            if capture.is_some() {
                values.insert(map_entry.0.to_string(), capture.unwrap().get(1).unwrap().as_str().to_string());
            }
        }
    }

    for postprocess in postprocessors {
        if values.contains_key(postprocess.0) {
            values.insert(postprocess.0.to_string(), postprocess.1(values[postprocess.0].to_string()));
        }
    }

    return values;
}