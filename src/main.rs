use batched_fn::batched_fn;
use env_logger::Env;
use log::{error, info};
use rust_bert::pipelines::generation::{GPT2Generator, GenerateConfig, LanguageGenerator};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tch::{Cuda, Device};
use warp::{http::StatusCode, reject::Reject, Filter, Rejection};

#[derive(Debug, Deserialize, Serialize)]
struct Context {
    text: String,
}

async fn generate(context: Context) -> Result<impl warp::Reply, Rejection> {
    info!("Received input context: {:?}", context);

    // Using the `batched_fn` macro, we run the model in a separate thread and
    // batch input `Context`s together to make better use of the GPU.
    //
    // NOTE: this is only more efficient if you have a GPU. If serving the model
    // on CPU this just adds overhead.
    let batched_generate = batched_fn! {
        handler = |batch: Vec<Context>, model: &GPT2Generator| -> Vec<String> {
            info!("Running batch of size {}", batch.len());
            model.generate(
                Some(batch.iter().map(|c| &c.text[..]).collect()),
                None,
            )
        };
        config = {
            max_batch_size: if Cuda::cudnn_is_available() { 4 } else { 1 },
            max_delay: 100,
            channel_cap: Some(20),
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

    batched_generate(context)
        .await
        .map(|output| {
            // Remove new lines in the generated text to make more readable.
            let formatted = output.replace('\n', " ");
            info!("Generated output: '{}'", formatted);
            formatted
        })
        .map_err(|e| match e {
            batched_fn::Error::Full => {
                error!("At capacity!");
                warp::reject::custom(CapacityFullError)
            }
            _ => {
                // This should only happen if the handler thread crashed.
                panic!("{:?}", e);
            }
        })
}

#[derive(Debug)]
struct CapacityFullError;

impl Reject for CapacityFullError {}

async fn handle_rejection(err: Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(CapacityFullError) = err.find() {
        code = StatusCode::SERVICE_UNAVAILABLE;
        message = "AT_CAPACITY";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    Ok(warp::reply::with_status(message, code))
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
        .and_then(generate)
        .recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
