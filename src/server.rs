use actix_web::{dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{Ok, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[post("/")]
async fn homepage(body: web::Bytes) -> impl Responder {
    println!("Got: {:?}", String::from_utf8(body.to_vec()));
    HttpResponse::Ok().body("Hello World")
}

pub struct BotServer {
    ip: &'static str,
    port: u32,
    worker: Server,
}

/// Create the key with
/// $ openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
/// -days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"
impl BotServer {
    pub fn new(ip: &'static str, port: u32) -> Self {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file("YOURPRIVATE.key", SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file("YOURPUBLIC.pem")
            .unwrap();
        let server = HttpServer::new(|| App::new().service(homepage))
            .bind_openssl(format!("{}:{}", ip, port), builder)
            .unwrap()
            .run();
        BotServer {
            ip: ip,
            port: port,
            worker: server,
        }
    }

    pub async fn start(self) -> Result<()> {
        self.worker.await?;
        Ok(())
    }
}
