package main

import (
	"context"
	"flag"
	"log"
	"strings"

	"github.com/go-redis/redis/v8"
	"github.com/segmentio/kafka-go"
)

var (
	kafkaBroker = "my-cluster-kafka-bootstrap.kafka.svc.cluster.local:9092" // Kafka broker
	topic       = "losers"                                                  // Tópico a consumir
	redisAddr   = "redis-service.redis:6379"                                // Dirección del servidor Redis
	redisClient *redis.Client
)

// var (
// 	kafkaBroker = "localhost:9092" // Kafka broker
// 	topic       = "losers"         // Tópico a consumir
// 	redisAddr   = "localhost:6379" // Dirección del servidor Redis
// 	redisClient *redis.Client
// )

// Init Kafka reader
func initKafkaReader() *kafka.Reader {
	return kafka.NewReader(kafka.ReaderConfig{
		Brokers:  []string{kafkaBroker},
		Topic:    topic,
		GroupID:  "my-consumer-group-losers", // Establece un GroupID
		MinBytes: 10e3,                       // 10KB
		MaxBytes: 10e6,                       // 10MB
	})
}

// Init Redis client
func initRedisClient() {
	redisClient = redis.NewClient(&redis.Options{
		Addr: redisAddr,
	})
}

// Extract faculty from the message
func extractFaculty(message string) string {
	parts := strings.Split(message, ",")
	for _, part := range parts {
		if strings.Contains(part, "Faculty") {
			return strings.TrimSpace(strings.Split(part, ":")[1]) // Extract the faculty name
		}
	}
	return ""
}

// Extract discipline from the message
func extractDiscipline(message string) string {
	parts := strings.Split(message, ",")
	for _, part := range parts {
		if strings.Contains(part, "Discipline") {
			return strings.TrimSpace(strings.Split(part, ":")[1]) // Extract the discipline
		}
	}
	return ""
}

// Map discipline ID to name
func getDisciplineName(disciplineID string) string {
	switch disciplineID {
	case "1":
		return "NATACION"
	case "2":
		return "ATLETISMO"
	case "3":
		return "BOXEO"
	default:
		return ""
	}
}

// Consume messages from Kafka and send them to Redis
func consumeFromKafka(reader *kafka.Reader) {
	for {
		message, err := reader.ReadMessage(context.Background())
		if err != nil {
			log.Printf("Error reading message: %v", err)
			continue
		}

		msg := string(message.Value)
		log.Printf("Received message: %s", msg)

		// Extraer la facultad del mensaje
		faculty := extractFaculty(msg)
		if faculty == "" {
			log.Printf("Faculty not found in message: %s", msg)
			continue
		}

		// Incrementar el contador total de estudiantes de la facultad
		_, err = redisClient.HIncrBy(context.Background(), "Total-Estudiantes", faculty, 1).Result()
		if err != nil {
			log.Printf("Failed to increment student counter for faculty %s: %v", faculty, err)
		} else {
			log.Printf("Student counter for faculty %s incremented successfully", faculty)
		}

		// Extraer la disciplina del mensaje
		disciplineID := extractDiscipline(msg)
		if disciplineID == "" {
			log.Printf("Discipline not found in message: %s", msg)
			continue
		}

		// Obtener el nombre de la disciplina
		disciplineName := getDisciplineName(disciplineID)
		if disciplineName == "" {
			log.Printf("Unknown discipline ID %s in message: %s", disciplineID, msg)
			continue
		}

		// Incrementar el contador de perdedores por disciplina en Redis
		_, err = redisClient.HIncrBy(context.Background(), "Losers-Disciplinas", disciplineName, 1).Result()
		if err != nil {
			log.Printf("Failed to increment losers counter for discipline %s: %v", disciplineName, err)
		} else {
			log.Printf("Losers counter for discipline %s incremented successfully", disciplineName)
		}

		if err := reader.CommitMessages(context.Background(), message); err != nil {
			log.Printf("Failed to commit message: %v", err)
		} else {
			log.Println("Message committed successfully")
		}
	}
}

func main() {
	flag.Parse()

	// Inicializa el cliente de Redis
	initRedisClient()
	defer redisClient.Close() // Cierra el cliente de Redis al terminar

	// Inicializa el lector de Kafka
	reader := initKafkaReader()
	defer reader.Close() // Cierra el reader cuando la aplicación termina

	log.Printf("Kafka consumer for losers started, listening to topic %s", topic)

	// Consumir mensajes en un bucle
	consumeFromKafka(reader)
}
