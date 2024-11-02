from locust import HttpUser, TaskSet, task, between
import json
import random

# Listas de nombres y apellidos
first_names = [
    "Juan", "María", "José", "Ana", "Luis", "Carmen", "Carlos", "Laura", "Miguel", "Sofía",
    "Diego", "Isabella", "Fernando", "Valentina", "Pablo", "Gabriela", "Andrés", "Lucía", 
    "Javier", "Claudia", "Sergio", "Natalia", "Ricardo", "Elena", "David", "Marta", 
    "Rafael", "Teresa", "Gonzalo", "Alba", "Cristian", "Raquel", "Arturo", "Victoria"
]

last_names = [
    "García", "Pérez", "López", "Martínez", "González", "Sánchez", "Ramírez", 
    "Torres", "Flores", "Reyes", "Hernández", "Cruz", "Morales", "Ortiz", "Gutiérrez", 
    "Castro", "Vásquez", "Moreno", "Silva", "Córdoba", "Rojas", "Mejía", "Díaz", 
    "Camacho", "Ponce", "Luna", "Pacheco", "Serrano", "López", "Salazar", "Valdés"
]

# Generación de datos de estudiantes
students = [
    {
        "name": f"{random.choice(first_names)} {random.choice(last_names)}", 
        "age": random.randint(18, 30),
        "faculty": random.choice(["Ingenieria", "Agronomia"]),
        "discipline": random.randint(1, 3)
    } 
    for _ in range(10000)
]

class UserBehavior(TaskSet):
    @task
    def send_data(self):
        student = random.choice(students)
        # Enviar datos a Agronomía
        if student['faculty'] == "Agronomia":
            print("funciona agro")
            response = self.client.post("http://localhost:8080/Agronomia", data=json.dumps(student), headers={"Content-Type": "application/json"})
            if response.status_code != 200:
                print(f"Error al enviar datos a Agronomía: {response.status_code}")
        
        # Enviar datos a Ingeniería
        elif student['faculty'] == "Ingenieria":
            print("funciona inge")
            response = self.client.post("http://localhost:8081/Ingenieria", data=json.dumps(student), headers={"Content-Type": "application/json"})
            if response.status_code != 200:
                print(f"Error al enviar datos a Ingeniería: {response.status_code}")

class WebsiteUser(HttpUser):
    tasks = [UserBehavior]
    wait_time = between(1, 3)  # Espera entre 1 y 2 segundos entre las solicitudes
