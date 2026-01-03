use crate::AppResult;
use local_ip_address::local_ip;
use std::env;
use std::process::Command;
use sysinfo::System;

/// System information structure
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub username: String,
    pub uptime: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub gpu_info: String,
    pub local_ip: String,
}

impl SystemInfo {
    /// Collect system information
    pub fn collect() -> AppResult<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Basic system information
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "Unknown".to_string());

        // Uptime
        let uptime_seconds = System::uptime();
        let uptime = format_uptime(uptime_seconds);

        // CPU information
        let cpu_model = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let cpu_cores = sys.cpus().len();

        // Memory information
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();

        // Disk information
        let disk_total = 0u64;
        let disk_used = 0u64;

        // GPU information
        let gpu_info = get_gpu_info();

        // Local IP address
        let local_ip = get_local_ip();

        Ok(Self {
            os_name,
            os_version,
            kernel_version,
            hostname,
            username,
            uptime,
            cpu_model,
            cpu_cores,
            memory_total,
            memory_used,
            disk_total,
            disk_used,
            gpu_info,
            local_ip,
        })
    }
}

/// Format uptime
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Format byte size
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_collection() {
        let info = SystemInfo::collect().unwrap();
        assert!(!info.os_name.is_empty());
        assert!(info.cpu_cores > 0);
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(3661), "1h 1m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
        assert_eq!(format_uptime(61), "1m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }
}

/// Get GPU information
fn get_gpu_info() -> String {
    // Use different commands to get GPU information based on the operating system
    if cfg!(target_os = "windows") {
        get_gpu_info_windows()
    } else if cfg!(target_os = "linux") {
        get_gpu_info_linux()
    } else if cfg!(target_os = "macos") {
        get_gpu_info_macos()
    } else {
        "Unknown GPU".to_string()
    }
}

/// Get GPU information on Windows system
fn get_gpu_info_windows() -> String {
    match Command::new("wmic")
        .args(&[
            "path",
            "win32_VideoController",
            "get",
            "name",
            "/format:value",
        ])
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("Name=") && !line.trim_end_matches("Name=").is_empty() {
                    return line.trim_start_matches("Name=").trim().to_string();
                }
            }
            "Unknown GPU".to_string()
        }
        Err(_) => "Unknown GPU".to_string(),
    }
}

/// Get GPU information on Linux system
fn get_gpu_info_linux() -> String {
    // Try using lspci command
    match Command::new("lspci").args(&["-mm"]).output() {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    // Parse lspci output and extract GPU name
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 6 {
                        return format!("{} {}", parts[3], parts[5]);
                    }
                }
            }
            "Unknown GPU".to_string()
        }
        Err(_) => "Unknown GPU".to_string(),
    }
}

/// Get GPU information on macOS system
fn get_gpu_info_macos() -> String {
    match Command::new("system_profiler")
        .args(&["SPDisplaysDataType", "-json"])
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Simple parsing to find GPU name
            if let Some(start) = output_str.find("\"_name\" : \"") {
                let start = start + 11;
                if let Some(end) = output_str[start..].find('"') {
                    return output_str[start..start + end].to_string();
                }
            }
            "Unknown GPU".to_string()
        }
        Err(_) => "Unknown GPU".to_string(),
    }
}

/// Get local IP address
fn get_local_ip() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "Unknown IP".to_string(),
    }
}
