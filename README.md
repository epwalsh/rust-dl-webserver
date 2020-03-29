# rust-dl-webserver

This project provides an example of serving a deep learning model with batched prediction using Rust.
In particular it runs a GPT2 model to generate text based on input context.

## Setup

You'll need to download the model files for GPT-2 through the [Rust Bert](https://github.com/guillaume-be/rust-bert) repository. This requires Python 3.

```bash
git clone https://github.com/guillaume-be/rust-bert && cd rust-bert
pip install -r requirements.txt
python utils/download-dependencies_gpt2.py
```

Also in order for the server to make use of your GPU (if you have one available) you'll need to compile it against the right
version of the LibTorch C++ library, which you can download from
[https://pytorch.org/get-started/locally/](https://pytorch.org/get-started/locally/). After downloading, unzip the file.

## Running the server

Once you've downloaded the model files and LibTorch, clone this repo and run the server with:

```bash
make run LIBTORCH=/path/to/libtorch
```

Now in a separate terminal you can send several requests in to the server at once:

```bash
curl -d '{"text":"Hello, World!"}' \
    -H "Content-Type: application/json" \
    -X POST \
    http://localhost:3030/generate &
curl -d '{"text":"Stay at home"}' \
    -H "Content-Type: application/json" \
    -X POST \
    http://localhost:3030/generate &
curl -d '{"text":"Wash your hands"}' \
    -H "Content-Type: application/json" \
    -X POST \
    http://localhost:3030/generate &
curl -d '{"text":"Do not touch your face"}' \
    -H "Content-Type: application/json" \
    -X POST \
    http://localhost:3030/generate &
```

The logs from the server should look something like this:

![server output](img/server_output.png)
