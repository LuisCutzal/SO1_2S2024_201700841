from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from datetime import datetime
import json
import os
import matplotlib.pyplot as plt

app = FastAPI()

# Ruta del volumen compartido
LOGS_PATH = "/logs"
MEMORY_LOG_FILE = os.path.join(LOGS_PATH, "memory_logs.json")
PROCESS_LOG_FILE = os.path.join(LOGS_PATH, "process_logs.json")


# Modelos para recibir los logs
class MemoryLog(BaseModel):
    total_kb: int
    free_kb: int
    used_kb: int
    timestamp: str = datetime.now().isoformat()

class ProcessLog(BaseModel):
    pid: int
    name: str
    container_id: str
    vsz_kb: int
    rss_kb: int
    memory_usage: float
    cpu_usage: float
    timestamp: str = datetime.now().isoformat()


# Función para guardar los logs en un archivo JSON
def save_log(log, file_path):
    if os.path.exists(file_path):
        with open(file_path, "r") as f:
            logs = json.load(f)
    else:
        logs = []

    logs.append(log)

    with open(file_path, "w") as f:
        json.dump(logs, f, indent=4)


# Endpoint para recibir logs de memoria
@app.post("/log/memory")
def log_memory(memory_log: MemoryLog):
    save_log(memory_log.dict(), MEMORY_LOG_FILE)
    return {"status": "Memory log saved successfully"}


# Endpoint para recibir logs de procesos
@app.post("/log/process")
def log_process(process_log: ProcessLog):
    save_log(process_log.dict(), PROCESS_LOG_FILE)
    return {"status": "Process log saved successfully"}


# Endpoint para generar gráficas
@app.get("/generate-graphs")
def generate_graphs():
    if not os.path.exists(MEMORY_LOG_FILE) or not os.path.exists(PROCESS_LOG_FILE):
        raise HTTPException(status_code=404, detail="Log files not found")

    # Cargar los logs de memoria
    with open(MEMORY_LOG_FILE, "r") as f:
        memory_logs = json.load(f)
    
    # Cargar los logs de procesos
    with open(PROCESS_LOG_FILE, "r") as f:
        process_logs = json.load(f)

    # Generar gráficos de memoria y procesos
    generate_memory_graph(memory_logs)
    generate_process_graph(process_logs)

    return {"status": "Graphs generated successfully"}


# Función para generar la gráfica de memoria
def generate_memory_graph(memory_logs):
    timestamps = [log["timestamp"] for log in memory_logs]
    used_kb = [log["used_kb"] for log in memory_logs]
    free_kb = [log["free_kb"] for log in memory_logs]

    plt.figure(figsize=(10, 6))
    plt.plot(timestamps, used_kb, label='Used Memory (KB)')
    plt.plot(timestamps, free_kb, label='Free Memory (KB)')
    plt.xticks(rotation=45, ha='right')
    plt.title("Memory Usage Over Time")
    plt.xlabel("Timestamp")
    plt.ylabel("Memory (KB)")
    plt.legend()
    plt.tight_layout()
    plt.savefig(os.path.join(LOGS_PATH, "memory_usage_graph.png"))


# Función para generar la gráfica de procesos
def generate_process_graph(process_logs):
    container_ids = [log["container_id"] for log in process_logs]
    cpu_usages = [log["cpu_usage"] for log in process_logs]

    plt.figure(figsize=(10, 6))
    plt.bar(container_ids, cpu_usages)
    plt.xticks(rotation=45, ha='right')
    plt.title("CPU Usage by Container")
    plt.xlabel("Container ID")
    plt.ylabel("CPU Usage (%)")
    plt.tight_layout()
    plt.savefig(os.path.join(LOGS_PATH, "cpu_usage_graph.png"))