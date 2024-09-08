use serde::{Deserialize, Serialize};
use std::process::Command;
use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio;
use reqwest::Client;

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
    vsz_kb: u64,
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

fn kill_container(container_id: &str) {
    let output = Command::new("sudo")
        .arg("docker")
        .arg("stop")
        .arg(container_id)
        .output()
        .expect("Error al ejecutar el comando docker");

    if output.status.success() {
        println!("Contenedor eliminado exitosamente: {}", container_id);
    } else {
        eprintln!(
            "Error al eliminar el contenedor {}: {}",
            container_id,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn remove_specific_cronjob(script_path: &str) {
    let output = Command::new("crontab")
        .arg("-l")
        .output()
        .expect("Failed to list cronjobs");

    if !output.status.success() {
        eprintln!("Error al listar cronjobs: {:?}", output.status);
        return;
    }

    let cronjobs = String::from_utf8_lossy(&output.stdout);
    let filtered_cronjobs: Vec<&str> = cronjobs
        .lines()
        .filter(|line| !line.contains(script_path))
        .collect();

    if filtered_cronjobs.len() == cronjobs.lines().count() {
        println!("No se encontró ningún cronjob relacionado con {}", script_path);
        return;
    }

    let new_cronjobs = filtered_cronjobs.join("\n");
    let mut apply_cron = Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to update cronjobs");

    if let Some(ref mut stdin) = apply_cron.stdin {
        use std::io::Write;
        stdin
            .write_all(new_cronjobs.as_bytes())
            .expect("Failed to write to cron stdin");
    }

    let status = apply_cron.wait().expect("Failed to wait on cronjob process");
    if status.success() {
        println!("Cronjob relacionado con {} eliminado exitosamente.", script_path);
    } else {
        eprintln!("Error al eliminar el cronjob: {:?}", status);
    }
}

async fn send_process_logs(client: &Client, log_process_list: &[LogProcess]) -> Result<(), Box<dyn std::error::Error>> {
    for log_process in log_process_list {
        let response = client.post("http://localhost:8000/log/process")
            .json(log_process)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            println!("Process log sent successfully: {:?}", log_process);
        } else {
            // Leer el cuerpo de la respuesta después de obtener el estado
            let response_text = response.text().await?;
            eprintln!("Failed to send process log. Status: {}, Response: {}", status, response_text);
        }
    }
    Ok(())
}

fn get_system_info() -> SystemInfo {
    let output = Command::new("cat")
        .arg("/proc/sysinfo_201700841")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        panic!("Error al leer el archivo de sistema");
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_proc_to_struct(&json_str).expect("Failed to parse JSON")
}

fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;
    Ok(system_info)
}

async fn analyzer(system_info: &SystemInfo, client: &Client) {
    let mut log_proc_list: Vec<LogProcess> = Vec::new();
    
    let mut processes_list: Vec<Process> = system_info.processes.clone();
    sort_processes(&mut processes_list);

    println!("--- Lista completa de contenedores (ordenada) ---");
    for process in &processes_list {
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

    for process in &processes_list {
        let log_process = LogProcess {
            pid: process.pid,
            container_id: process.get_container_id().to_string(),
            name: process.name.clone(),
            vsz_kb: process.vsz_kb,
            rss_kb: process.rss_kb,
            memory_usage: process.memory_usage,
            cpu_usage: process.cpu_usage,
        };

        log_proc_list.push(log_process);
    }

    println!("------------------------------");

    println!("--- Contenedores enviados ---");
    for log_process in &log_proc_list {
        println!(
            "PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}",
            log_process.pid,
            log_process.name,
            log_process.container_id,
            log_process.vsz_kb,
            log_process.rss_kb,
            log_process.memory_usage,
            log_process.cpu_usage
        );
    }

    println!("------------------------------");

    if let Err(e) = send_process_logs(client, &log_proc_list).await {
        eprintln!("Error al enviar los logs de procesos: {}", e);
    }

    println!("------------------------------");
    let num_processes = processes_list.len();
    if num_processes < 5 {
        println!("No hay suficientes contenedores para aplicar análisis de alto/bajo consumo.");
        return;
    }

    let high_consumption = &processes_list[..2];
    let low_consumption = &processes_list[num_processes - 3..];
    let to_kill = &processes_list[2..num_processes - 3];

    // Definir los contenedores que no deben ser eliminados
    let protected_containers: Vec<&str> = vec![
        "16418f21d19bc31aae8ea1d55b98932d65d3c6ea76604e93d125843329bf914b"
    ];

    println!("--- Contenedores de bajo consumo ---");
    for process in low_consumption {
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

    println!("--- Contenedores con alto consumo ---");
    for process in high_consumption {
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

    println!("--- Contenedores a eliminar ---");
    for process in to_kill {
        let container_id = process.get_container_id();
        if !protected_containers.contains(&container_id) {
            println!(
                "PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}",
                process.pid,
                process.name,
                container_id,
                process.vsz_kb,
                process.rss_kb,
                process.memory_usage,
                process.cpu_usage
            );

            kill_container(container_id);
        } else {
            println!(
                "Contenedor protegido (no eliminado): {}",
                container_id
            );
        }
    }

    println!("------------------------------");
}

#[tokio::main]
async fn main() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();
    ctrlc::set_handler(move || {
        stop_flag_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let client = Client::new();

    while !stop_flag.load(Ordering::SeqCst) {
        let system_info = get_system_info();
        analyzer(&system_info, &client).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }

    println!("Programa detenido por señal Ctrl-C.");
}
