use actix_web::{get, post, web, Error, App, HttpResponse, HttpServer, Responder};
use actix_multipart::{
    form::{
        tempfile::TempFile,
        text::Text,
        MultipartForm,
    },
};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    tags: Text<String>,
    file: TempFile
}

#[get("/get")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/upload")]
async fn save_files(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    println!("{} ? {}", form.tags.as_str(), form.file.size);
    /*
    for f in form {
        let path = format!("./tmp/{}", f.file_name.unwrap());
        f.file.persist(path).unwrap();
    }
    */

    Ok(HttpResponse::Ok())
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(save_files)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("192.168.1.10", 8080))?
    .run()
    .await
}
