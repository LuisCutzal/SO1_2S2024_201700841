use studentgrpc::student_client::StudentClient;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use studentgrpc::StudentRequest;
use serde::{Deserialize, Serialize};
use tokio::task;

pub mod studentgrpc {
    tonic::include_proto!("student");
}

#[derive(Deserialize, Serialize)]
struct StudentData {
    name: String,
    age: i32,
    faculty: String,
    discipline: i32,
}

async fn handle_student(student: web::Json<StudentData>) -> impl Responder {
    // Crear un hilo asíncrono para manejar la llamada gRPC
    let student_data = student.into_inner();
    let handle = task::spawn(async move {
        let mut client = match StudentClient::connect("http://go-server-service:50051").await {
            Ok(client) => client,
            Err(e) => return Err(format!("Failed to connect to gRPC server: {}", e)),
        };

        let request = tonic::Request::new(StudentRequest {
            name: student_data.name,
            age: student_data.age,
            faculty: student_data.faculty,
            discipline: student_data.discipline,
        });

        match client.get_student(request).await {
            Ok(response) => Ok(format!("Student: {:?}", response)),
            Err(e) => Err(format!("gRPC call failed: {}", e)),
        }
    });

    // Esperar a que el hilo termine y manejar el resultado
    match handle.await {
        Ok(Ok(response)) => HttpResponse::Ok().json(response),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e),
        Err(e) => HttpResponse::InternalServerError().body(format!("Task panicked: {:?}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://go-server-service:8080");
    HttpServer::new(|| {
        App::new()
            .route("/faculty", web::post().to(handle_student))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

/* #[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = StudentClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(StudentRequest{
       name: "Alvaro Rust".into(),
       age: 25,
       faculty: "Ingenería".into(),
       discipline: 1, 
    });

    let response = client.send_student(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
} */