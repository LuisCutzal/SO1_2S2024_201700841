use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SystemInfo {
    #[serde(rename = "MemoriaTotalKB")]
    memoria_total_kb: u64,
    #[serde(rename = "MemoriaLibreKB")]
    memoria_libre_kb: u64,
    #[serde(rename = "MemoriaUsadaKB")]
    memoria_usada_kb: u64,
    #[serde(rename = "Procesos")]
    processes: Vec<Process>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Process {
    #[serde(rename = "PID")]
    pid: u32,
    #[serde(rename = "Nombre")]
    name: String,
    #[serde(rename = "ContenedorID")]
    cmd_line: String,
    #[serde(rename = "VSZ_KB")]
    vsz_kb: u64,
    #[serde(rename = "RSS_KB")]
    rss_kb: u64,
    #[serde(rename = "PorcentajeMemoria")]
    memory_usage: f64,
    #[serde(rename = "PorcentajeCPU")]
    cpu_usage: f64,
}

#[derive(Debug, Serialize, Clone)]
struct LogProcess {
    pid: u32,
    container_id: String,
    name: String,
    vsz_k: u64,
    rss_kb: u64,
    memory_usage: f64,
    cpu_usage: f64,
}

impl Process {
    fn get_container_id(&self) -> &str {
        let parts: Vec<&str> = self.cmd_line.split_whitespace().collect();
        if let Some(last_part) = parts.last() {
            if last_part.len() == 64 { // Asumiendo que el ID tiene 64 caracteres
                return last_part;
            }
        }
        "N/A"
    }
}

impl Eq for Process {}

impl Ord for Process {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cpu_usage.partial_cmp(&other.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.memory_usage.partial_cmp(&other.memory_usage).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| self.rss_kb.cmp(&other.rss_kb))
            .then_with(|| self.vsz_kb.cmp(&other.vsz_kb))
    }
}

impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn sort_processes(processes: &mut Vec<Process>) {
    processes.sort();
}

fn kill_container(id: &str) -> std::process::Output {
    let output = Command::new("sudo")
        .arg("docker")
        .arg("stop")
        .arg(id)
        .output()
        .expect("failed to execute process");

    println!("Matando contenedor con id: {}", id);

    output
}

fn analyzer(system_info: &SystemInfo) {
    let mut log_proc_list: Vec<LogProcess> = Vec::new();
    let mut processes_list: Vec<Process> = system_info.processes.clone();

    sort_processes(&mut processes_list);

    let (lowest_list, highest_list) = processes_list.split_at(processes_list.len() / 2);

    println!("Bajo consumo");
    for process in lowest_list {
        println!("PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}", 
        process.pid, 
        process.name, 
        process.get_container_id(), 
        process.vsz_kb,
        process.rss_kb,
        process.memory_usage, 
        process.cpu_usage);
    }

    println!("------------------------------");

    println!("Alto consumo");
    for process in highest_list {
        println!("PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}", 
        process.pid, 
        process.name,
        process.get_container_id(),
        process.vsz_kb,
        process.rss_kb,
        process.memory_usage, 
        process.cpu_usage);
    }

    println!("------------------------------");

    if lowest_list.len() > 3 {
        for process in lowest_list.iter().skip(3) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.name.clone(),
                vsz_k: process.vsz_kb,
                rss_kb: process.rss_kb,
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage,
            };

            log_proc_list.push(log_process.clone());
            let _output = kill_container(&process.get_container_id());
        }
    } 

    if highest_list.len() > 2 {
        for process in highest_list.iter().take(highest_list.len() - 2) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.name.clone(),
                vsz_k: process.vsz_kb,
                rss_kb: process.rss_kb,
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage
            };

            log_proc_list.push(log_process.clone());
            let _output = kill_container(&process.get_container_id());
        }
    }

    println!("Contenedores matados");
    for process in log_proc_list {
        println!("PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {} ",
        process.pid,
        process.name,
        process.container_id,
        process.vsz_k,
        process.rss_kb,
        process.memory_usage,
        process.cpu_usage);
    }

    println!("------------------------------");
}


fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;
    Ok(system_info)
}

fn main() {
    let output = Command::new("cat")
        .arg("/proc/sysinfo_201700841")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Error executing command: {:?}", output.status);
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the system info once
    let system_info: Result<SystemInfo, serde_json::Error> = parse_proc_to_struct(&stdout);
    let system_info = match system_info {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error parsing system info: {:?}", e);
            return;
        }
    };

    // Print system information once
    println!("Información del sistema:");
    println!("Memoria Total (KB): {}", system_info.memoria_total_kb);
    println!("Memoria Libre (KB): {}", system_info.memoria_libre_kb);
    println!("Memoria Usada (KB): {}", system_info.memoria_usada_kb);
    println!("------------------------------");

    loop {
        analyzer(&system_info);

        // Sleep to avoid excessive CPU usage in the loop
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
