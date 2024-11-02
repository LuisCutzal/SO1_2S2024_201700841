package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"math/rand"
	"net"
	"time"

	pb "go-client/proto"

	"github.com/segmentio/kafka-go"
	"google.golang.org/grpc"
)

// var (
// 	port        = flag.Int("port", 50052, "The server port")
// 	kafkaBroker = "my-cluster-kafka-bootstrap.kafka.svc.cluster.local:9092" // Kafka broker
// 	topic       = "winners"                                                 // Tópico por defecto
// 	kafkaWriter *kafka.Writer
// )

var (
	port        = flag.Int("port", 50052, "The server port")
	kafkaBroker = "localhost:9092" // Kafka broker
	topic       = "winners"        // Tópico por defecto
	kafkaWriter *kafka.Writer
)

// Server is used to implement the gRPC server in the proto library
type server struct {
	pb.UnimplementedStudentServer
}

// Implement the GetStudent method
// Implement the GetStudent method
func (s *server) GetStudent(_ context.Context, in *pb.StudentRequest) (*pb.StudentResponse, error) {
	// Log the received data
	log.Printf("Received: %v", in)
	log.Printf("Student: %s", in.GetName())
	log.Printf("Student faculty: %s", in.GetFaculty())
	log.Printf("Student age: %d", in.GetAge())
	log.Printf("Student discipline: %d", in.GetDiscipline())

	// Generate winner using a random coin toss
	rand.Seed(time.Now().UnixNano())
	winner := rand.Intn(2) // 0 (loser) or 1 (winner)

	// Determine Kafka topic based on winner or loser
	if winner == 1 {
		topic = "winners"
	} else {
		topic = "losers"
	}

	// Create the message to send to Kafka
	result := "loser"
	if winner == 1 {
		result = "winner"
	}
	message := fmt.Sprintf("Student: %s, Age: %d, Faculty: %s, Discipline: %d, Result: %s",
		in.GetName(), in.GetAge(), in.GetFaculty(), in.GetDiscipline(), result)

	// Produce message to Kafka
	err := produceToKafka(topic, message)
	if err != nil {
		log.Printf("Failed to produce message to Kafka: %v", err)
		return nil, err
	}

	// Return the gRPC response
	return &pb.StudentResponse{
		Success: true,
	}, nil
}

// Init Kafka writer (producer) to reuse during the server's lifetime
func initKafkaWriter() {
	kafkaWriter = kafka.NewWriter(kafka.WriterConfig{
		Brokers:  []string{kafkaBroker},
		Balancer: &kafka.LeastBytes{},
	})
}

// Produce message to Kafka

func produceToKafka(topic string, message string) error {
	if topic == "" {
		return fmt.Errorf("topic must be specified")
	}

	// Produce the message to the specified topic
	err := kafkaWriter.WriteMessages(context.Background(),
		kafka.Message{
			Topic: topic,         // Especifica el tópico aquí
			Key:   []byte("key"), // Puedes cambiar la clave según tu lógica
			Value: []byte(message),
		},
	)

	if err != nil {
		return fmt.Errorf("failed to produce message: %s", err)
	}

	log.Printf("Message sent to topic %s: %s", topic, message)
	return nil
}

func main() {
	flag.Parse()
	lis, err := net.Listen("tcp", fmt.Sprintf(":%d", *port))
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	// Inicializa el escritor de Kafka solo una vez
	initKafkaWriter()
	defer kafkaWriter.Close() // Cierra el writer cuando la aplicación termina

	s := grpc.NewServer()
	pb.RegisterStudentServer(s, &server{})
	log.Printf("Server started on port %d", *port)

	if err := s.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
