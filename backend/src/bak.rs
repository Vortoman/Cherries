#[get("/images/pix")]
async fn serve_image() -> HttpResponse {
    let image_data = fs::read("./backend/data/pixelated.png").unwrap();
    HttpResponse::Ok()
        .content_type("image/png")
        .body(image_data)
}
#[post("/files/back/")]
async fn get_from_frontend(request: Json<File>) -> Result<HttpResponse, JsonPayloadError> {
    let out = request.into_inner();
    println!("got data {} and {}", out.name, out.data);
    Ok(HttpResponse::Ok().body("success!"))
}

#[get("/files/{name}/")]
async fn download(info: web::Path<String>) -> HttpResponse {
    let name = String::from(info.into_inner().as_str());
    HttpResponse::Ok()
        .content_type("application/json")
        .body(name)
}
