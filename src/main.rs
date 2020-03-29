use batched_fn::batched_fn;
use env_logger::Env;
use log::info;
use rust_bert::pipelines::generation::{GPT2Generator, GenerateConfig, LanguageGenerator};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tch::{Cuda, Device};
use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct Context {
    text: String,
}

async fn generate(context: Context) -> Result<impl warp::Reply, Infallible> {
    let batched_generate = batched_fn! {
        handler = |batch: Vec<Context>, model: &GPT2Generator| -> Vec<String> {
            info!("Running batch of size {}", batch.len());
            model.generate(
                Some(batch.iter().map(|c| &c.text[..]).collect()),
                None,
            )
        };
        config = {
            max_batch_size: 4,
            delay: 100,
        };
        context = {
            model: {
                info!("Loading model...");
                let device = Device::cuda_if_available();
                let generate_config = GenerateConfig {
                    max_length: 30,
                    do_sample: true,
                    num_beams: 5,
                    temperature: 1.1,
                    num_return_sequences: 1,
                    ..Default::default()
                };
                let home = dirs::home_dir().unwrap();
                let model = GPT2Generator::new(
                    &home.join("rustbert/gpt2/vocab.txt"),
                    &home.join("rustbert/gpt2/merges.txt"),
                    &home.join("rustbert/gpt2/config.json"),
                    &home.join("rustbert/gpt2/model.ot"),
                    generate_config,
                    device,
                ).unwrap();
                info!("...model loaded");
                model
            },
        };
    };

    info!("Received input context: {:?}", context);

    let output = batched_generate(context).await;

    info!("Generated output: '{}'", output.replace('\n', " "));

    Ok(output)
}

#[tokio::main]
async fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    info!("Cuda available? {}", Cuda::cudnn_is_available());

    // POST /generate/ {"context":"Hello, World!"}
    let routes = warp::post()
        .and(warp::path("generate"))
        // Only accept bodies smaller than 16kb.
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(generate);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
