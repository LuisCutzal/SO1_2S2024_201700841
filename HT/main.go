package main

import (
	"encoding/json"
	"net/http"
)

type Student struct {
	Student    string `json:"student"`
	Age        int    `json:"age"`
	Faculty    string `json:"faculty"`
	Discipline int    `json:"discipline"`
}

func handler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {
		var student Student
		// Decodificar el cuerpo de la solicitud JSON a la estructura Student
		if err := json.NewDecoder(r.Body).Decode(&student); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		// Responder con el objeto recibido
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(student)
		return
	}

	// Manejo de otras solicitudes (por ejemplo, GET)
	http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
}

func main() {
	http.HandleFunc("/", handler)
	http.ListenAndServe(":8080", nil)
}
