use crate::ServerConfig;

use crate::types::Bot;
use actix_ip_filter::IPFilter;
use actix_server::{Server, ServerHandle};
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use openssl::x509::{X509NameBuilder, X509};
use socket2::{Domain, Protocol, Socket, Type};
use std::env;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;

pub struct BotServer<B: Bot + Send + Sync> {
    worker: Option<Server>,
    handle: ServerHandle,
    pub bot: Arc<B>,
}

#[post("/")]
async fn handler(body: web::Bytes, bot: web::Data<Arc<dyn Bot>>) -> impl Responder {
    let update = if let Ok(msg) = String::from_utf8(body.to_vec()) {
        msg
    } else {
        error!("Wrong message format received! {:#?}", body.to_vec());
        return HttpResponse::BadRequest();
    };
    if bot.into_inner().handle_message(update).await.is_err() {
        error!("Failed to handle the message!");
        return HttpResponse::InternalServerError();
    }
    HttpResponse::Ok()
}

impl<B: Bot> BotServer<B> {
    const TIME_WAIT: u64 = 3;
    pub fn new(config: ServerConfig, bot: Arc<B>) -> Self {
        let mut priv_key = env::current_dir().unwrap();
        priv_key.push(config.clone().privkey_path);
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(priv_key, SslFiletype::PEM)
            .unwrap();

        let mut pub_key = env::current_dir().unwrap();
        pub_key.push(config.clone().pubkey_path);
        builder.set_certificate_chain_file(pub_key).unwrap();
        builder
            .check_private_key()
            .expect("failed to check the private key");
        let bot_clone = bot.clone();
        let bot_object: Arc<dyn Bot> = bot_clone;

        let addr: SocketAddr = format!("{}:{}", config.ip, config.port).parse().unwrap();

        // Setting up the socket
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();

        // We use SO_REUSEADDR to prevent the case where the system did not release the
        // binded port in time. Since we are stopping the previous server instance  before starting
        // a new one, we know we are safe.
        socket.set_reuse_address(true).unwrap();
        socket.bind(&addr.into()).unwrap();
        socket.listen(128).unwrap();
        let listener: TcpListener = socket.into();

        let server = HttpServer::new(move || {
            let new_bot = bot_object.clone();
            App::new()
                .app_data(web::Data::new(new_bot.clone()))
                .service(handler)
                .wrap(TracingLogger::default())
                .wrap(IPFilter::new().allow(new_bot.get_webhook_ips().unwrap()))
        })
        .shutdown_timeout(Self::TIME_WAIT)
        .listen_openssl(listener, builder)
        .unwrap()
        .run();

        BotServer {
            bot,
            handle: server.handle(),
            worker: Some(server),
        }
    }

    pub async fn generate_certificate(pubkey: PathBuf, privkey: PathBuf, ip: &str) -> Result<()> {
        let rsa = Rsa::generate(2048)?;
        let key_pair = PKey::from_rsa(rsa)?;

        let mut x509_name = X509NameBuilder::new()?;
        x509_name.append_entry_by_text("C", "DE")?;
        x509_name.append_entry_by_text("ST", "B")?;
        x509_name.append_entry_by_text("O", "homebot")?;
        x509_name.append_entry_by_text("CN", ip)?;
        let x509_name = x509_name.build();

        let mut cert_builder = X509::builder()?;
        cert_builder.set_version(2)?;
        let serial_number = {
            let mut serial = BigNum::new()?;
            serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
            serial.to_asn1_integer()?
        };
        cert_builder.set_serial_number(&serial_number)?;
        cert_builder.set_subject_name(&x509_name)?;
        cert_builder.set_issuer_name(&x509_name)?;
        cert_builder.set_pubkey(&key_pair)?;

        let not_before = Asn1Time::days_from_now(0)?;
        cert_builder.set_not_before(&not_before)?;

        let not_after = Asn1Time::days_from_now(365)?;
        cert_builder.set_not_after(&not_after)?;

        cert_builder.sign(&key_pair, openssl::hash::MessageDigest::sha256())?;
        let cert = cert_builder.build();

        fs::write(&pubkey, cert.to_pem()?).await?;
        fs::write(&privkey, &key_pair.private_key_to_pem_pkcs8()?).await?;

        let mut file = File::open(&pubkey).await?;
        file.flush().await?;
        file.sync_all().await?;

        let mut file = File::open(&privkey).await?;
        file.flush().await?;
        file.sync_all().await?;

        info!("Generated the keys !");
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting the server ...");
        // we take the server from the option so as to not take
        // ownership of "self", so that we can use the handle, to
        // stop the server at a later time.
        if let Some(worker) = self.worker.take() {
            // worker.await?;
            let task = tokio::spawn(worker);
            futures::future::join_all(vec![task]).await;
        }
        Ok(())
    }

    pub async fn stop(&self) {
        info!("Stopping the server ..");
        self.handle.stop(false).await;
    }
}
