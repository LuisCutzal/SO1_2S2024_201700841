use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use studentgrpc::student_client::StudentClient;
use studentgrpc::StudentRequest;
use serde::{Deserialize, Serialize};
use tokio; // Asegúrate de tener `tokio` en tu archivo `Cargo.toml`

pub mod studentgrpc {
    tonic::include_proto!("student");
}

// Cambia `name` a `student` para que coincida con el JSON de entrada
#[derive(Deserialize, Serialize)]
struct StudentData {
    #[serde(rename = "name")]
    name: String,     // Se mapea "student" del JSON a "name" en Rust
    age: i32,
    faculty: String,
    discipline: i32,
}

// Mapa de servidores según la disciplina

const SERVERS: [&str; 3] = [
    "http://localhost:50051", //http://go-server-natacion-service:50051", // Para disciplina 1
    "http://localhost:50052",//http://go-server-atletismo-service:50052", // Para disciplina 2
    "http://localhost:50053", //"http://go-server-boxeo-service:50053", // Para disciplina 3
];

async fn handle_student(student: web::Json<StudentData>) -> impl Responder {
    // Verificamos si la disciplina está dentro de los límites
    if student.discipline < 1 || student.discipline > 3 {
        return HttpResponse::BadRequest().body("Discipline must be 1, 2, or 3");
    }

    // Seleccionamos la dirección del servidor según la disciplina
    let server_addr = SERVERS[(student.discipline - 1) as usize];

    // Creamos un hilo para la conexión gRPC
    let student_name = student.name.clone();
    let student_age = student.age;
    let student_faculty = student.faculty.clone();
    let student_discipline = student.discipline;

    // Llamamos a Tokio para ejecutar el hilo
    tokio::spawn(async move {
        // Intentamos conectar al servidor gRPC
        let mut client = match StudentClient::connect(server_addr).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Failed to connect to gRPC server: {}", e);
                return;
            }
        };

        // Creamos la solicitud
        let request = tonic::Request::new(StudentRequest {
            name: student_name,
            age: student_age,
            faculty: student_faculty,
            discipline: student_discipline,
        });

        // Realizamos la llamada al servidor gRPC
        match client.get_student(request).await {
            Ok(response) => {
                println!("RESPONSE={:?}", response);
            },
            Err(e) => eprintln!("gRPC call failed: {}", e),
        }
    });

    // Responder inmediatamente con un mensaje de éxito
    HttpResponse::Accepted().body("Request is being processed")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at -> http://0.0.0.0:8081");
    HttpServer::new(|| {
        App::new()
            .route("/Ingenieria", web::post().to(handle_student))
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
