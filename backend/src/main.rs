
mod page_gen;
extern crate actix_rt;
use actix_session::CookieSession;
use actix_identity::{ CookieIdentityPolicy, IdentityService };
use actix_web::{ web, HttpResponse, App, HttpServer, Responder, middleware };
use tera::ErrorKind;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1";
    let port = "8080";
    let log_lvl = "info";

    std::env::set_var("RUST_LOG", format!("actix_web={}", log_lvl));
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new().wrap(middleware::Logger::default())
            .wrap(
                CookieSession::signed(&[0; 32])
                    .name("post_session")
                    .path("/")
                    .secure(false)
                    .max_age(60 * 60i64)
            )
            .wrap(
                IdentityService::new(
                    CookieIdentityPolicy::new(&[0;32])
                        .name("admin")
                        .path("/admin")
                        .max_age(60 * 60i64)
                        .secure(false)
                )
            )
            .service(
                web::scope("/")
                     .service(web::resource("").route(web::get().to(hello)))
            )
    }).bind(format!("{}:{}", &addr, &port))?
        .run()
        .await;

    Ok(())
}

async fn hello() -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("name", &String::from("ben"));
    let template = page_gen::TEMPLATES.render("index.html", &ctx);
    match template {
        Ok(t) => HttpResponse::Ok().content_type("text/html").body(t),
        Err(e) => HttpResponse::NotImplemented().await.unwrap()
    }
}
 
