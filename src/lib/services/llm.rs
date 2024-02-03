use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;

#[allow(unused)]
use llm_chain::document_stores::document_store::DocumentStore;
#[allow(unused)]
use llm_chain::tools::tools::VectorStoreTool;
use llm_chain::{chains::conversation::Chain, executor, parameters, prompt, step::Step};
#[allow(unused)]
use llm_chain::{
    schema::{Document, EmptyMetadata},
    traits::{Embeddings, VectorStore},
};
use llm_chain_openai::chatgpt::Executor;
use llm_chain_qdrant::Qdrant;
use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig},
};
use tokio::sync::Mutex;
use tracing::debug;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn request(&self, req: &str) -> Result<String>;
    async fn conversation(&self, req: &str, chain: Arc<Mutex<Chain>>) -> Result<String>;
    async fn chain_requests(&self, steps: Vec<&str>) -> Result<String>;
    async fn map_reduce_chain(&self, steps: Vec<&str>) -> Result<String>;
    async fn retrieval(&self, collection: &str, req: &str) -> Result<String>;
}

pub struct OpenAiModel {
    _api_token: Option<String>,
    executor: Executor,
}

impl OpenAiModel {
    const EMBEDDING_SIZE: u64 = 1536;
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

    async fn retrieval(&self, collection: &str, req: &str) -> Result<String> {
        let collection_name = collection.to_string();

        let db_config = QdrantClientConfig::from_url("http://localhost:6334");
        let client = Arc::new(QdrantClient::new(Some(db_config))?);
        let embeddings = llm_chain_openai::embeddings::Embeddings::default();
        if !client.has_collection(collection_name.clone()).await? {
            client
                .create_collection(&CreateCollection {
                    collection_name: collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                            VectorParams {
                                size: Self::EMBEDDING_SIZE,
                                distance: Distance::Cosine.into(),
                                hnsw_config: None,
                                quantization_config: None,
                                on_disk: None,
                            },
                        )),
                    }),
                    ..Default::default()
                })
                .await?;
        }

        // Store the documents
        let qdrant: Qdrant<llm_chain_openai::embeddings::Embeddings, EmptyMetadata> = Qdrant::new(
            client.clone(),
            collection_name.clone(),
            embeddings,
            None,
            None,
            None,
        );

        let doc_dog_definition = r#"The dog (Canis familiaris[4][5] or Canis lupus familiaris[5]) is a domesticated descendant of the wolf. Also called the domestic dog, it is derived from the extinct Pleistocene wolf,[6][7] and the modern wolf is the dog's nearest living relative.[8] Dogs were the first species to be domesticated[9][8] by hunter-gatherers over 15,000 years ago[7] before the development of agriculture.[1] Due to their long association with humans, dogs have expanded to a large number of domestic individuals[10] and gained the ability to thrive on a starch-rich diet that would be inadequate for other canids.[11]
                The dog has been selectively bred over millennia for various behaviors, sensory capabilities, and physical attributes.[12] Dog breeds vary widely in shape, size, and color. They perform many roles for humans, such as hunting, herding, pulling loads, protection, assisting police and the military, companionship, therapy, and aiding disabled people. Over the millennia, dogs became uniquely adapted to human behavior, and the human–canine bond has been a topic of frequent study.[13] This influence on human society has given them the sobriquet of "man's best friend"."#.to_string();
        let doc_woodstock_sound = r#"Sound for the concert was engineered by sound engineer Bill Hanley. "It worked very well", he says of the event. "I built special speaker columns on the hills and had 16 loudspeaker arrays in a square platform going up to the hill on 70-foot [21 m] towers. We set it up for 150,000 to 200,000 people. Of course, 500,000 showed up."[48] ALTEC designed marine plywood cabinets that weighed half a ton apiece and stood 6 feet (1.8 m) tall, almost 4 feet (1.2 m) deep, and 3 feet (0.91 m) wide. Each of these enclosures carried four 15-inch (380 mm) JBL D140 loudspeakers. The tweeters consisted of 4×2-Cell & 2×10-Cell Altec Horns. Behind the stage were three transformers providing 2,000 amperes of current to power the amplification setup.[49][page needed] For many years this system was collectively referred to as the Woodstock Bins.[50] The live performances were captured on two 8-track Scully recorders in a tractor trailer back stage by Edwin Kramer and Lee Osbourne on 1-inch Scotch recording tape at 15 ips, then mixed at the Record Plant studio in New York.[51]"#.to_string();
        let doc_reddit_creep_shots = r#"A year after the closure of r/jailbait, another subreddit called r/CreepShots drew controversy in the press for hosting sexualized images of women without their knowledge.[34] In the wake of this media attention, u/violentacrez was added to r/CreepShots as a moderator;[35] reports emerged that Gawker reporter Adrian Chen was planning an exposé that would reveal the real-life identity of this user, who moderated dozens of controversial subreddits, as well as a few hundred general-interest communities. Several major subreddits banned links to Gawker in response to the impending exposé, and the account u/violentacrez was deleted.[36][37][38] Moderators defended their decisions to block the site from these sections of Reddit on the basis that the impending report was "doxing" (a term for exposing the identity of a pseudonymous person), and that such exposure threatened the site's structural integrity.[38]"#.to_string();

        let doc_ids = qdrant
            .add_documents(
                vec![
                    doc_dog_definition,
                    doc_woodstock_sound,
                    doc_reddit_creep_shots,
                ]
                .into_iter()
                .map(Document::new)
                .collect(),
            )
            .await?;

        debug!("Documents stored under IDs: {:?}", doc_ids);
        let response: String = qdrant
            .similarity_search(req.to_string(), 1)
            .await?
            .iter()
            .map(|x| x.page_content.to_string())
            .collect();

        let res = prompt!("Given this text as the context: {{txt}}, can you try to answer briefly this question {{question}}."
        )
        .run(&parameters!("txt"=>response, "question"=>req), &self.executor)
        .await?;
        Ok(res.to_string())
    }
}
