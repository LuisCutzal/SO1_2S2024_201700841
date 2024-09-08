use std::process::Command;
use std::sync::{Arc, atomic::{AtomicBool, Ordering as AtomicOrdering}};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::{self, Read}, path::Path, time::Duration};
use std::thread;

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

enum SortCriteria {
    CpuUsage,
    MemoryUsage,
    VszKb,
    RssKb,
}

impl Process {
    fn get_container_id(&self) -> &str {
        let parts: Vec<&str> = self.cmd_line.split_whitespace().collect();
        if let Some(last_part) = parts.last() {
            if last_part.len() == 64 {
                return last_part;
            }
        }
        "N/A"
    }
}

// Ordenamiento completo en función de múltiples criterios
fn multi_criteria_sort(processes_list: &mut Vec<Process>) {
    processes_list.sort_by(|a, b| {
        let criteria_cmp = |a: &Process, b: &Process, criteria: &SortCriteria| match criteria {
            SortCriteria::CpuUsage => b.cpu_usage.partial_cmp(&a.cpu_usage),
            SortCriteria::MemoryUsage => b.memory_usage.partial_cmp(&a.memory_usage),
            SortCriteria::VszKb => b.vsz_kb.cmp(&a.vsz_kb),
            SortCriteria::RssKb => b.rss_kb.cmp(&a.rss_kb),
        };

        let mut comparison = criteria_cmp(a, b, &SortCriteria::CpuUsage);
        if comparison == Some(std::cmp::Ordering::Equal) {
            comparison = criteria_cmp(a, b, &SortCriteria::MemoryUsage);
        }
        if comparison == Some(std::cmp::Ordering::Equal) {
            comparison = criteria_cmp(a, b, &SortCriteria::VszKb);
        }
        if comparison == Some(std::cmp::Ordering::Equal) {
            comparison = criteria_cmp(a, b, &SortCriteria::RssKb);
        }

        comparison.unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn kill_container(id: &str) -> std::process::Output {
    let output = Command::new("sudo")
        .arg("docker")
        .arg("stop")
        .arg(id)
        .output()
        .expect("Failed to execute docker stop");

    println!("Matando contenedor con id: {}", id);
    output
}

fn log_to_container(log_message: &str, log_container_id: &str) {
    let client = Client::new();
    let url = format!("http://localhost:8080/logs/{}", log_container_id);
    let response = client.post(&url)
        .body(log_message.to_string())
        .send();

    match response {
        Ok(_) => println!("Log enviado al contenedor de logs."),
        Err(e) => println!("Error enviando log: {}", e),
    }
}

fn create_log_container() -> Result<String, String> {
    let output = Command::new("sudo")
        .arg("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg("log_container")
        .arg("log_image")
        .output()
        .expect("Error al crear el contenedor de logs");

    if output.status.success() {
        let container_id = String::from_utf8_lossy(&output.stdout).to_string();
        println!("Contenedor de logs creado con ID: {}", container_id);
        Ok(container_id.trim().to_string())
    } else {
        Err(format!("Error creando contenedor de logs: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

fn analyzer(system_info: &SystemInfo, log_container_id: &str) {
    let mut log_proc_list: Vec<LogProcess> = Vec::new();
    let mut processes_list = system_info.processes.clone();

    // Ordenar procesos usando todos los criterios en cascada
    multi_criteria_sort(&mut processes_list);

    // Dividir la lista en alto consumo y bajo consumo
    let (lowest_list, highest_list) = processes_list.split_at(processes_list.len() / 2);

    println!("Bajo consumo");
    for process in lowest_list {
        println!(
            "PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}",
            process.pid,
            process.name,
            process.get_container_id(),
            process.vsz_kb,
            process.rss_kb,
            process.memory_usage,
            process.cpu_usage
        );
    }

    println!("------------------------------");

    println!("Alto consumo");
    for process in highest_list {
        println!(
            "PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}",
            process.pid,
            process.name,
            process.get_container_id(),
            process.vsz_kb,
            process.rss_kb,
            process.memory_usage,
            process.cpu_usage
        );
    }

    // Eliminar procesos con bajo consumo (los que están más abajo en la lista)
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

    // Eliminar procesos con alto consumo (los que están más arriba en la lista)
    if highest_list.len() > 2 {
        for process in highest_list.iter().take(highest_list.len() - 2) {
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

    println!("Contenedores matados");
    for process in log_proc_list {
        let log_message = format!(
            "PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}",
            process.pid,
            process.name,
            process.container_id,
            process.vsz_k,
            process.rss_kb,
            process.memory_usage,
            process.cpu_usage
        );
        log_to_container(&log_message, log_container_id);
    }
}

fn read_file(file_path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_system_info(contents: &str) -> Result<SystemInfo, serde_json::Error> {
    serde_json::from_str(contents)
}

fn loop_system_analyzer() {
    let log_container_id = create_log_container().expect("Error creando el contenedor de logs");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let _handle = thread::spawn(move || {
        while r.load(AtomicOrdering::SeqCst) {
            let file_path = Path::new("/tmp/system_info.json");

            match read_file(file_path) {
                Ok(contents) => match parse_system_info(&contents) {
                    Ok(system_info) => analyzer(&system_info, &log_container_id),
                    Err(e) => eprintln!("Error parsing system info: {}", e),
                },
                Err(e) => eprintln!("Error reading file: {}", e),
            }

            thread::sleep(Duration::from_secs(60));
        }
    });

    // Simulación de parada de la aplicación
    thread::sleep(Duration::from_secs(30)); // Mantener corriendo por 1 hora para demostrar
    running.store(false, AtomicOrdering::SeqCst);
}

fn main() {
    loop_system_analyzer();
}
