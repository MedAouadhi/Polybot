use anyhow::Result;
use async_trait::async_trait;
use llm_chain::{executor, parameters, prompt};
use llm_chain_openai::chatgpt::Executor;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn request(&self, req: &str) -> Result<String>;
    async fn chain_requests(&self, steps: Vec<&str>) -> Result<String>;
    async fn map_reduce_chain(&self, steps: Vec<&str>) -> Result<String>;
}

pub struct OpenAiModel {
    _api_token: Option<String>,
    executor: Executor,
}

impl OpenAiModel {
    pub fn new() -> Self {
        Self {
            _api_token: None,
            executor: executor!().unwrap(),
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
}
