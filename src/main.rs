use batched_fn::batched_fn;
use exitfailure::ExitFailure;
use rust_bert::pipelines::generation::{GPT2Generator, LanguageGenerator};
use std::path::Path;
use tch::{Cuda, Device};

struct ModelContext {
    model: GPT2Generator,
}

impl Default for ModelContext {
    fn default() -> Self {
        let device = Device::cuda_if_available();
        Self {
            model: GPT2Generator::new(
                Path::new("/home/epwalsh/rustbert/gpt2/vocab.txt"),
                Path::new("/home/epwalsh/rustbert/gpt2/merges.txt"),
                Path::new("/home/epwalsh/rustbert/gpt2/config.json"),
                Path::new("/home/epwalsh/rustbert/gpt2/model.ot"),
                device,
            )
            .unwrap(),
        }
    }
}

async fn generate(context: String) -> String {
    let batched_generate = batched_fn! {
        |batch: Vec<String>, ctx: &ModelContext| -> Vec<String> {
            let output = ctx.model.generate(
                Some(batch.iter().map(|c| &c[..]).collect()),
                0,
                30,
                true,
                false,
                5,
                1.2,
                0,
                0.9,
                1.0,
                1.0,
                3,
                1,
                None,
            );
            println!("Processed batch of size {}", output.len());
            output
        },
        max_batch_size = 4,
        delay = 50,
    };
    batched_generate(context).await
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    println!("Cuda available? {}", Cuda::cudnn_is_available());

    let mut handles = vec![];

    handles.push(tokio::spawn(async move {
        let output = generate("The dog".into()).await;
        println!("--> {}", output);
    }));

    handles.push(tokio::spawn(async move {
        let output = generate("The cat was".into()).await;
        println!("--> {}", output);
    }));

    handles.push(tokio::spawn(async move {
        let output = generate("Hello there".into()).await;
        println!("--> {}", output);
    }));

    handles.push(tokio::spawn(async move {
        let output = generate("Penguins tend to".into()).await;
        println!("--> {}", output);
    }));

    handles.push(tokio::spawn(async move {
        let output = generate("Does a bear shit in the woods?".into()).await;
        println!("--> {}", output);
    }));

    // Wait for spawn tasks to finish.
    for join_handle in handles {
        join_handle.await?;
    }

    Ok(())
}
