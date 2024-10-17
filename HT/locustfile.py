from locust import HttpUser, task
import random

class User(HttpUser):
    @task
    def send_student_data(self):
        student_data = {
            'student': f'Luis {random.randint(1, 100)}',  # Generar un nombre único
            'age': random.randint(18, 30),  # Edad aleatoria entre 18 y 30
            'faculty': 'Ingeniería',
            'discipline': random.randint(1, 5)  # Disciplina aleatoria entre 1 y 5
        }
        # Enviar datos a la API
        self.client.post("/", json=student_data)
