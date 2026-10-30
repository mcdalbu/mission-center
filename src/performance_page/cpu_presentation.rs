// SPDX-License-Identifier: GPL-3.0-or-later
//! Read-only CPU presentation helpers. No power settings are written here.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreType { Performance, Efficiency, Unknown }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreInfo { pub cpu: usize, pub kind: CoreType }

pub fn format_epp(raw: &str) -> String {
    let value = match raw.trim() {
        "performance" => Some(0),
        "balance_performance" => Some(128),
        "balance_power" => Some(192),
        "power" => Some(255),
        value => value.parse::<u8>().ok(),
    };
    let Some(value) = value else { return raw.to_owned() };
    let name = match value {
        0 => "Maximum Performance", 32 => "High Performance", 64 => "Performance",
        96 => "Responsive", 128 => "Balanced", 160 => "Efficient",
        192 => "Power Saving", 224 => "Maximum Savings", 255 => "Full Economy",
        _ => "Custom value",
    };
    format!("{value} · {name}")
}

pub fn parse_cpu_list(raw: &str) -> Option<Vec<usize>> {
    let mut result = BTreeSet::new();
    for group in raw.trim().split(',') {
        let (first, last) = match group.trim().split_once('-') {
            Some((first, last)) => (first.parse::<usize>().ok()?, last.parse::<usize>().ok()?),
            None => { let cpu = group.trim().parse::<usize>().ok()?; (cpu, cpu) },
        };
        if first > last || last > 65535 { return None; }
        result.extend(first..=last);
    }
    Some(result.into_iter().collect())
}

fn read_cpu_list(path: &Path) -> Vec<usize> {
    fs::read_to_string(path).ok().and_then(|text| parse_cpu_list(&text)).unwrap_or_default()
}

pub fn core_info(cpu_root: &Path, pmu_root: &Path, count: usize) -> Vec<CoreInfo> {
    let online = read_cpu_list(&cpu_root.join("online"));
    let ids: Vec<_> = if online.len() == count { online } else { (0..count).collect() };
    let p = read_cpu_list(&pmu_root.join("cpu_core/cpus"));
    let e = read_cpu_list(&pmu_root.join("cpu_atom/cpus"));
    ids.into_iter().map(|cpu| CoreInfo { cpu, kind: match (p.contains(&cpu), e.contains(&cpu)) {
        (true, false) => CoreType::Performance,
        (false, true) => CoreType::Efficiency,
        _ => CoreType::Unknown,
    }}).collect()
}

pub fn frequency_khz(cpu_root: &Path, cpu: usize) -> Option<u64> {
    fs::read_to_string(cpu_root.join(format!("cpu{cpu}/cpufreq/scaling_cur_freq")))
        .ok()?.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

pub fn core_label(core: &CoreInfo, khz: Option<u64>) -> String {
    let title = match core.kind {
        CoreType::Performance => format!("CPU {} · P-core", core.cpu),
        CoreType::Efficiency => format!("CPU {} · E-core", core.cpu),
        CoreType::Unknown => format!("CPU {}", core.cpu),
    };
    let frequency = khz.map(|value| format!("{:.2} GHz", value as f64 / 1_000_000.0))
        .unwrap_or_else(|| "—".to_owned());
    format!("{title}\n{frequency}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn epp_names_and_aliases_preserve_values() {
        for (raw, expected) in [
            ("0", "0 · Maximum Performance"), ("32", "32 · High Performance"),
            ("64", "64 · Performance"), ("96", "96 · Responsive"),
            ("128", "128 · Balanced"), ("160", "160 · Efficient"),
            ("192", "192 · Power Saving"), ("224", "224 · Maximum Savings"),
            ("255", "255 · Full Economy"), ("208", "208 · Custom value"),
            ("balance_power", "192 · Power Saving"), ("balance_performance", "128 · Balanced"),
            ("performance", "0 · Maximum Performance"), ("power", "255 · Full Economy"),
            ("Mixed", "Mixed"), ("default", "default"), ("256", "256"),
        ] { assert_eq!(format_epp(raw), expected); }
    }
    #[test]
    fn cpu_list_handles_ranges_holes_and_invalid_input() {
        assert_eq!(parse_cpu_list("0,2-3,6\n"), Some(vec![0,2,3,6]));
        assert_eq!(parse_cpu_list("3,0-1,1"), Some(vec![0,1,3]));
        for bad in ["", "3-1", "abc", "1,", "0-999999"] { assert_eq!(parse_cpu_list(bad), None); }
    }
    #[test]
    fn frequencies_use_decimal_ghz_and_unknown_is_not_zero() {
        assert_eq!(core_label(&CoreInfo {cpu: 0, kind: CoreType::Performance}, Some(3_250_000)), "CPU 0 · P-core\n3.25 GHz");
        assert_eq!(core_label(&CoreInfo {cpu: 4, kind: CoreType::Efficiency}, Some(400_000)), "CPU 4 · E-core\n0.40 GHz");
        assert_eq!(core_label(&CoreInfo {cpu: 7, kind: CoreType::Unknown}, None), "CPU 7\n—");
    }
    #[test]
    fn sysfs_topology_and_missing_frequency() {
        let root = std::env::temp_dir().join(format!("missioncenter-core-test-{}", std::process::id()));
        let cpu = root.join("cpu"); let pmu = root.join("pmu");
        fs::create_dir_all(cpu.join("cpu2/cpufreq")).unwrap();
        fs::create_dir_all(pmu.join("cpu_core")).unwrap();
        fs::create_dir_all(pmu.join("cpu_atom")).unwrap();
        fs::write(cpu.join("online"), "0,2-3").unwrap();
        fs::write(pmu.join("cpu_core/cpus"), "0,2").unwrap();
        fs::write(pmu.join("cpu_atom/cpus"), "3").unwrap();
        let info = core_info(&cpu,&pmu,3);
        assert_eq!(info, vec![CoreInfo{cpu:0,kind:CoreType::Performance},CoreInfo{cpu:2,kind:CoreType::Performance},CoreInfo{cpu:3,kind:CoreType::Efficiency}]);
        fs::write(cpu.join("cpu2/cpufreq/scaling_cur_freq"), "2500000\n").unwrap();
        assert_eq!(frequency_khz(&cpu,2), Some(2_500_000));
        assert_eq!(frequency_khz(&cpu,3), None);
        fs::write(pmu.join("cpu_atom/cpus"), "0,3").unwrap();
        assert_eq!(core_info(&cpu,&pmu,3)[0].kind, CoreType::Unknown);
        fs::remove_dir_all(root).unwrap();
    }
}
