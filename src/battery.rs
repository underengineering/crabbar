use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

pub struct BatteryInfo {
    pub capacity: i32,
    pub status: String,
}

fn read_to_string_buf<P: AsRef<Path>>(path: P, buffer: &mut String) -> io::Result<&mut String> {
    let mut file = File::open(path)?;

    buffer.clear();
    file.read_to_string(buffer)?;

    Ok(buffer)
}

// https://github.com/elkowar/eww/blob/dc3129aee2806823bdad87785f7ef80651d5245c/crates/eww/src/config/system_stats.rs#L118
// https://github.com/valpackett/systemstat/blob/cbd9c1638b792d1819479f0c2baa5840f65af727/src/platform/linux.rs#L584
pub fn get_batteries() -> HashMap<String, BatteryInfo> {
    let mut batteries = HashMap::new();

    let power_supply_dir = Path::new("/sys/class/power_supply");
    let power_supplies = power_supply_dir
        .read_dir()
        .expect("Failed to read /sys/class/power_supply/");

    let mut buffer = String::with_capacity(16);
    for entry in power_supplies {
        let entry = entry.expect("Failed to get power supply entry").path();

        // Skip non-batteries
        if read_to_string_buf(entry.join("type"), &mut buffer)
            .map(|t| t != "Battery\n")
            .unwrap_or(true)
        {
            continue;
        }

        let capacity = read_to_string_buf(entry.join("capacity"), &mut buffer)
            .expect("Failed to read battery capacity")
            .trim_end_matches('\n')
            .parse::<i32>()
            .expect("Failed to parse battery capacity");

        let status = {
            let mut str = read_to_string_buf(entry.join("status"), &mut buffer)
                .expect("Failed to read battery status")
                .to_owned();
            str.truncate(str.trim_end_matches('\n').len());
            str
        };

        batteries.insert(
            entry
                .file_name()
                .expect("Failed to get battery name")
                .to_string_lossy()
                .to_string(),
            BatteryInfo { capacity, status },
        );
    }

    batteries
}
