use std::sync::Arc;
use crate::types::WeatherProvider;

use super::types::Update;
use super::telegram_ops::Bot;
use actix_web::{dev::Server, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{Ok, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web::middleware::Logger;

pub struct BotServer<B: Bot> {
    ip: &'static str,
    port: u32,
    worker: Server,
    pub bot: Arc<B>,
}

#[post("/")]
async fn handler(body: web::Bytes, bot: web::Data<Arc<dyn Bot>>) -> impl Responder {
    let update: Update = String::from_utf8(body.to_vec()).unwrap().into();
    if let Some(msg) = update.message {
        bot.into_inner().handle_message(msg).await.unwrap();
    } else {
        println!("Unsupported message format {:#?}", update);
    }
    HttpResponse::Ok()
}

impl<B: Bot> BotServer<B> {
    pub fn new(ip: &'static str, port: u32, bot: Arc<B>) -> Self {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file("/home/mohamed/personal/homebot/YOURPRIVATE.key", SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file("/home/mohamed/personal/homebot/YOURPUBLIC.pem")
            .unwrap();
        let bot_clone = bot.clone();
        let bot_object : Arc<dyn Bot> = bot_clone;

        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(bot_object.clone()))
                .service(handler)
                .wrap(Logger::default())
        })
        .bind_openssl(format!("{}:{}", ip, port), builder)
        .unwrap()
        .run();

        BotServer {
            ip: ip,
            port: port,
            bot: bot.clone(),
            worker: server,
        }
    }

    pub async fn start(self) -> Result<()> {
        self.worker.await?;
        Ok(())
    }
}
