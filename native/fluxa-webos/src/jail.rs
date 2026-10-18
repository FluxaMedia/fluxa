use std::fs;

use serde_json::Value;

const BLOCKED_CHASSIS: [&str; 2] = ["k5lp", "k3lp"];

pub fn blocks_load(machine: &str, rtkmem_readable: bool) -> bool {
    let machine = machine.to_lowercase();
    !rtkmem_readable && BLOCKED_CHASSIS.iter().any(|chassis| machine.starts_with(chassis))
}

fn machine_name() -> String {
    let from_os_info = fs::read_to_string("/var/run/nyx/os_info.json")
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|json| json["machine_name"].as_str().map(str::to_owned));
    from_os_info
        .or_else(|| fs::read_to_string("/proc/sys/kernel/hostname").ok())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

pub fn refusal() -> Option<String> {
    let machine = machine_name();
    let readable = fs::File::open("/dev/rtkmem").is_ok();
    blocks_load(&machine, readable)
        .then(|| format!("this TV ({machine}) does not give apps /dev/rtkmem; run the Homebrew Channel jail repair"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realtek_chassis_without_rtkmem_is_refused() {
        assert!(blocks_load("k5lp-atsc", false));
        assert!(blocks_load("K3LP", false));
    }

    #[test]
    fn readable_rtkmem_or_other_chassis_is_allowed() {
        assert!(!blocks_load("k5lp-atsc", true));
        assert!(!blocks_load("m16p4", false));
    }
}
