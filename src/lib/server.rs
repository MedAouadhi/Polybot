use crate::ServerConfig;

use super::telegrambot::Bot;
use super::types::Update;
use actix_ip_filter::IPFilter;
use actix_web::middleware::Logger;
use actix_web::{dev::Server, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{Ok, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

pub struct BotServer<B: Bot> {
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
    pub fn new(config: ServerConfig, bot: Arc<B>) -> Self {
        let mut priv_key = PathBuf::from(env::current_dir().unwrap());
        priv_key.push(config.privkey_path);
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(priv_key, SslFiletype::PEM)
            .unwrap();

        let mut pub_key = PathBuf::from(env::current_dir().unwrap());
        pub_key.push(config.pubkey_path);
        builder.set_certificate_chain_file(pub_key).unwrap();
        let bot_clone = bot.clone();
        let bot_object: Arc<dyn Bot> = bot_clone;

        let server = HttpServer::new(move || {
            let new_bot = bot_object.clone();
            App::new()
                .app_data(web::Data::new(new_bot.clone()))
                .service(handler)
                .wrap(Logger::default())
                .wrap(IPFilter::new().allow(new_bot.get_server_ips().unwrap()))
        })
        .shutdown_timeout(3)
        .bind_openssl(format!("{}:{}", config.ip, config.port), builder)
        .unwrap()
        .run();

        BotServer {
            bot: bot.clone(),
            worker: server,
        }
    }

    pub async fn start(self) -> Result<()> {
        self.worker.await?;
        Ok(())
    }
}
