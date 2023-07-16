use std::sync::Arc;
use super::types::Update;
use super::bot::Bot;
use actix_web::{dev::Server, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{Ok, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

pub struct BotServer {
    ip: &'static str,
    port: u32,
    worker: Server,
    bot: Arc<Bot>,
}

#[post("/")]
async fn handler(body: web::Bytes, bot: web::Data<Arc<Bot>>) -> impl Responder {
    let update: Update = String::from_utf8(body.to_vec()).unwrap().into();
    if let Some(msg) = update.message {
        bot.handle_message(msg).await.unwrap();
    } else {
        println!("Unsupported message format {:#?}", update);
    }
    HttpResponse::Ok()
}

impl BotServer {
    pub fn new(ip: &'static str, port: u32, bot: Arc<Bot>) -> Self {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file("/home/mohamed/personal/homebot/YOURPRIVATE.key", SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file("/home/mohamed/personal/homebot/YOURPUBLIC.pem")
            .unwrap();
        let bot_clone = bot.clone();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(bot_clone.clone()))
                .service(handler)
        })
        .bind_openssl(format!("{}:{}", ip, port), builder)
        .unwrap()
        .run();
        BotServer {
            ip: ip,
            port: port,
            bot: bot,
            worker: server,
        }
    }

    pub async fn start(self) -> Result<()> {
        self.worker.await?;
        Ok(())
    }
}
