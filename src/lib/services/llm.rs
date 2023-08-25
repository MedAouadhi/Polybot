use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use llm_chain::{chains::conversation::Chain, executor, parameters, prompt, step::Step};
use llm_chain_openai::chatgpt::Executor;
use tokio::sync::Mutex;
use tracing::debug;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn request(&self, req: &str) -> Result<String>;
    async fn conversation(&self, req: &str, chain: Arc<Mutex<Chain>>) -> Result<String>;
    async fn chain_requests(&self, steps: Vec<&str>) -> Result<String>;
    async fn map_reduce_chain(&self, steps: Vec<&str>) -> Result<String>;
}

pub struct OpenAiModel {
    _api_token: Option<String>,
    executor: Executor,
}

impl OpenAiModel {
    pub fn try_new() -> Result<Self> {
        // check if the OPENAI_API_KEY variable exists
        if let Ok(token) = std::env::var("OPENAI_API_KEY") {
            if !token.is_empty() {
                debug!("OPENAI_API_KEY found!");
                Ok(Self {
                    _api_token: None,
                    executor: executor!().unwrap(),
                })
            } else {
                bail!("OPENAI_API_KEY variable is empty");
            }
        } else {
            bail!("OPENAI_API_KEY not found in env variables!");
        }
    }
}

#[async_trait]
impl Agent for OpenAiModel {
    async fn request(&self, req: &str) -> Result<String> {
        let res = prompt!(
            "You are a clever assistant that understands something about everything, 
            and particulary good with explaining things, you will try to make your answers
            as brief as possible",
            req
        )
        .run(&parameters!(), &self.executor)
        .await?;
        Ok(res.to_string())
    }
    async fn chain_requests(&self, _steps: Vec<&str>) -> Result<String> {
        todo!()
    }
    async fn map_reduce_chain(&self, _steps: Vec<&str>) -> Result<String> {
        todo!()
    }

    async fn conversation(&self, req: &str, chain: Arc<Mutex<Chain>>) -> Result<String> {
        let step = Step::for_prompt_template(prompt!(user: req));
        Ok(chain
            .lock()
            .await
            .send_message(step, &parameters!(), &self.executor)
            .await?
            .to_immediate()
            .await?
            .to_string())
    }
}
