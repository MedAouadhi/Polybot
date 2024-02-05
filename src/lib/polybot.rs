use crate::plant::PlantServer;
use crate::server::BotServer;
use crate::utils::{generate_certificate, get_ip};
use crate::{Bot, Config};
use anyhow::Result;
use std::path::PathBuf;
use std::{sync::Arc, time::Duration};
use tokio::select;
use tokio::sync::Notify;
use tracing::{debug, error, info};


pub struct Polybot<B: Bot> {
    bot: Arc<B>,
    config: Config,
    webhook_monitor: Option<Duration>,
}

impl<B: Bot> Polybot<B> {
    pub fn new(config: Config) -> Self {
        Self {
            bot: Arc::new(B::new(config.clone().bot)),
            config,
            webhook_monitor: None,
        }
    }

    /// Monitors each "timeout" period of time, the webhook ip address and the bot's
    /// current ip address, generates a new certificate with the updated ip, updating
    /// the webhook right after.
    pub fn with_webhook_monitoring(mut self, timeout: Duration) -> Self {
        self.webhook_monitor = Some(timeout);
        self
    }

    /// Starts the main loop of the bot, starts the server, and the webhook monitoring
    /// if enabled.
    pub async fn start_loop(&self) -> Result<()> {
        let bot_clone = self.bot.clone();
        let conf_clone = self.config.clone();
        let config_changed = Arc::new(Notify::new());
        let config_changed_clone = config_changed.clone();

        if let Some(timeout) = self.webhook_monitor {
            tokio::spawn(async move {
                loop {
                    // explicity handle the result as we are in async block
                    if let Ok(current_ip) = get_ip().await {
                        debug!("Current ip = {:?}", current_ip);
                        if let Ok(configured) = bot_clone.is_webhook_configured(&current_ip).await {
                            if !configured {
                                info!("Certificate is not correclty configured, configuring ...");
                            } else {
                                // the webhook is already set
                                tokio::time::sleep(timeout).await;
                                continue;
                            }
                        } else {
                            error!("Issue with getting the webhook status.");
                        }

                        // generate new certificate
                        if generate_certificate(
                            PathBuf::from(conf_clone.clone().server.pubkey_path),
                            PathBuf::from(conf_clone.clone().server.privkey_path),
                            &current_ip,
                            "Polybot",
                        )
                        .await
                        .is_ok()
                        {
                            if bot_clone
                                .update_webhook_cert(
                                    PathBuf::from(conf_clone.clone().server.pubkey_path),
                                    &current_ip,
                                )
                                .await
                                .is_err()
                            {
                                error!("failed to upload the certificate!");
                            } else {
                                // notify the server that a new certificate has been uploaded
                                config_changed_clone.notify_one();
                            }
                        } else {
                            error!("The certificate generation failed!");
                        }
                    }
                    tokio::time::sleep(timeout).await;
                }
            });
        }
        loop {
            let mut server = BotServer::new(self.config.server.clone(), self.bot.clone());
            let plant = PlantServer::new(
                "192.168.2.214",
                &self.config.bot.chat_id,
                3333,
                &self.config.bot.db_token,
            );

            // the flow will block here, until one of the branches terminates, which is due to:
            // - The server terminates by itself (e.g crash ..)
            // - The system's IP has changed
            select! {
                _ = server.start() => {break;},
                // A server restart needs to happen as the certificate has been changed.
                _ = config_changed.notified() => {
                    debug!("Received certificate update notification, restarting server ...");
                    server.stop().await;
                    continue;
                }
                e = plant.start(self.bot.clone()) => {
                    tracing::error!("Plant Server exited {:?}", e);
                    continue;
                }
            }
        }
        Ok(())
    }
}
