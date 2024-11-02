package main

import (
	"context"
	pb "go-client/proto"
	"log"
	"time"

	"github.com/gofiber/fiber/v2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// var (
// 	servers = map[int]string{
// 		1: "golang-server-natacion-service:50051",  // Para disciplina 1
// 		2: "golang-server-atletismo-service:50052", // Para disciplina 2
// 		3: "golang-server-boxeo-service:50053",     // Para disciplina 3
// 	}
// )

var (
	servers = map[int]string{
		1: "http://localhost:50051", // Para disciplina 1
		2: "http://localhost:50052", // Para disciplina 2
		3: "http://localhost:50053", // Para disciplina 3
	}
)

type Student struct {
	Name       string `json:"name"`
	Age        int    `json:"age"`
	Faculty    string `json:"faculty"`
	Discipline int    `json:"discipline"`
}

func sendData(fiberCtx *fiber.Ctx) error {
	var body Student
	if err := fiberCtx.BodyParser(&body); err != nil {
		return fiberCtx.Status(400).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Verificamos si la disciplina está en el mapa de servidores
	addr, ok := servers[body.Discipline]
	if !ok {
		return fiberCtx.Status(400).JSON(fiber.Map{
			"error": "discipline debe ser 1, 2 o 3 para enviar al servidor gRPC",
		})
	}

	// Conectar al servidor gRPC
	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Printf("Error de conexión: %v", err)
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error": "Error de conexión al servidor gRPC",
		})
	}
	defer conn.Close()

	c := pb.NewStudentClient(conn)

	// Crear un canal para recibir la respuesta y el error
	responseChan := make(chan *pb.StudentResponse)
	errorChan := make(chan error)

	go func() {
		// Conectar servidor y obtener su respuesta
		ctx, cancel := context.WithTimeout(context.Background(), time.Second*10)
		defer cancel()
		r, err := c.GetStudent(ctx, &pb.StudentRequest{
			Name:       body.Name,
			Age:        int32(body.Age),
			Faculty:    body.Faculty,
			Discipline: pb.Discipline(body.Discipline),
		})
		if err != nil {
			errorChan <- err
			return
		}
		responseChan <- r
	}()

	select {
	case response := <-responseChan:
		return fiberCtx.JSON(fiber.Map{
			"message": response.GetSuccess(),
		})
	case err := <-errorChan:
		log.Printf("Error al obtener respuesta del servidor gRPC: %v", err)
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error": "Error al obtener respuesta del servidor gRPC",
		})
	case <-time.After(10 * time.Second):
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error": "timeout",
		})
	}
}

func main() {
	app := fiber.New()
	app.Post("/Agronomia", sendData)

	err := app.Listen(":8080")
	if err != nil {
		log.Println(err)
		return
	}
}
