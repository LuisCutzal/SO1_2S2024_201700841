use serde::{Deserialize, Serialize};
use std::process::Command;
use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    // Ordenar procesos por cpu_usage, memory_usage, rss_kb, y vsz_kb
    processes.sort();
    //println!("Procesos ordenados (debug):");
    // for process in processes {
    //     println!("PID: {}, CPU Usage: {}, Memory Usage: {}, RSS: {}, VSZ: {}",
    //         process.pid, process.cpu_usage, process.memory_usage, process.rss_kb, process.vsz_kb);
    // }
}

fn kill_container(contenedor_id: &str) {
    let output = Command::new("sudo")
        .arg("docker")
        .arg("stop")
        //.arg("-f")
        .arg(contenedor_id)
        .output()
        .expect("Error al ejecutar el comando docker");

    if output.status.success() {
        println!("Contenedor eliminado exitosamente: {}", contenedor_id);
    } else {
        eprintln!(
            "Error al eliminar el contenedor {}: {}",
            contenedor_id,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn remove_specific_cronjob(script_path: &str) {
    // Obtener la lista actual de cronjobs
    let output = Command::new("crontab")
        .arg("-l")
        .output()
        .expect("Failed to list cronjobs");

    if !output.status.success() {
        eprintln!("Error al listar cronjobs: {:?}", output.status);
        return;
    }

    // Convertir la salida a string
    let cronjobs = String::from_utf8_lossy(&output.stdout);

    // Filtrar los cronjobs que no coincidan con el script_path
    let filtered_cronjobs: Vec<&str> = cronjobs
        .lines()
        .filter(|line| !line.contains(script_path))
        .collect();

    // Verificar si había un cronjob asociado al script
    if filtered_cronjobs.len() == cronjobs.lines().count() {
        println!("No se encontró ningún cronjob relacionado con {}", script_path);
        return;
    }

    // Escribir los cronjobs filtrados de vuelta al crontab
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


fn analyzer(system_info: &SystemInfo) {
    // Lista para almacenar los procesos eliminados
    let mut log_proc_list: Vec<LogProcess> = Vec::new();
    
    // Copiamos y ordenamos la lista de procesos según el criterio existente
    let mut processes_list: Vec<Process> = system_info.processes.clone();
    sort_processes(&mut processes_list);


    // Imprimimos todos los contenedores (ya ordenados)
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

    // Ahora seleccionamos los 2 contenedores de mayor consumo y los 3 de menor consumo
    let high_consumption = &processes_list[..2]; // Primeros 2 contenedores
    let low_consumption = &processes_list[processes_list.len() - 3..]; // Últimos 3 contenedores

    // Eliminamos todos los demás contenedores que no están ni en high_consumption ni en low_consumption
    let to_kill = &processes_list[2..processes_list.len() - 3]; // Todos los demás contenedores

    // Imprimimos los 3 contenedores de menor consumo
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

    // Imprimimos los 2 contenedores de mayor consumo
    println!("--- Contenedores de alto consumo ---");
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

    // Eliminamos los contenedores de consumo medio y los agregamos a log_proc_list
    //println!("--- Eliminando contenedores de consumo medio ---");
    for process in to_kill {
        let log_process = LogProcess {
            pid: process.pid,
            container_id: process.get_container_id().to_string(),
            name: process.name.clone(),
            vsz_k: process.vsz_kb,
            rss_kb: process.rss_kb,
            memory_usage: process.memory_usage,
            cpu_usage: process.cpu_usage,
        };

        // Guardamos el proceso en la lista log_proc_list
        log_proc_list.push(log_process.clone());

        // Simulamos el proceso de eliminación del contenedor
        let _output = kill_container(&process.get_container_id());
    }

    println!("------------------------------");

    // Imprimimos los contenedores que fueron eliminados
    println!("--- Contenedores eliminados ---");
    for log_process in log_proc_list {
        println!(
            "PID: {}, Nombre: {}, ContenedorID: {}, VSZ_KB: {}, RSS_KB: {}, Memory Usage: {}, CPU Usage: {}",
            log_process.pid,
            log_process.name,
            log_process.container_id,
            log_process.vsz_k,
            log_process.rss_kb,
            log_process.memory_usage,
            log_process.cpu_usage
        );
    }
    
    println!("------------------------------");
}



fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    let system_info: SystemInfo = serde_json::from_str(json_str)?;
    Ok(system_info)
}

fn main() {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    // Manejador para Ctrl+C
    ctrlc::set_handler(move || {
        println!("Ctrl+C recibido, eliminando cronjob...");
        remove_specific_cronjob("generate_containers.sh");
        stop_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error al configurar el manejador de Ctrl+C");

    while !stop.load(Ordering::SeqCst) {
        // Leer el archivo /proc/sysinfo_201700841 dentro del bucle para obtener datos actualizados
        let output = Command::new("cat")
            .arg("/proc/sysinfo_201700841")
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            eprintln!("Error executing command: {:?}", output.status);
            return;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse the system info dentro del bucle para obtener la información más reciente
        let system_info: Result<SystemInfo, serde_json::Error> = parse_proc_to_struct(&stdout);
        let system_info = match system_info {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Error parsing system info: {:?}", e);
                return;
            }
        };

        // Print system information
        println!("Información del sistema:");
        println!("Memoria Total (KB): {}", system_info.memoria_total_kb);
        println!("Memoria Libre (KB): {}", system_info.memoria_libre_kb);
        println!("Memoria Usada (KB): {}", system_info.memoria_usada_kb);
        println!("------------------------------");

        // Ejecutar el análisis en los datos más recientes
        analyzer(&system_info);

        // Sleep to avoid excessive CPU usage in the loop
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    println!("Servicio terminado.");
}
