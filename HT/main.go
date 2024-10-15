package main

import (
    "encoding/json"
    "fmt"
    "log"
    "net/http"
)

type Student struct {
    Name     string `json:"student"`
    Age      int    `json:"age"`
    Faculty  string `json:"faculty"`
    Discipline int  `json:"discipline"`
}

func handler(w http.ResponseWriter, r *http.Request) {
    var student Student

    err := json.NewDecoder(r.Body).Decode(&student)
    if err != nil {
        http.Error(w, "Bad request", http.StatusBadRequest)
        return
    }

    fmt.Fprintf(w, "Received: %+v\n", student)
    log.Printf("Received student info: %+v\n", student)
}

func main() {
    http.HandleFunc("/", handler)
    log.Println("Listening on port 8080...")
    log.Fatal(http.ListenAndServe(":8080", nil))
}
