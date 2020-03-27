# rust-dl-webserver

This project provides an example of a serving a deep learning model with batched prediction using Rust.
In particular it runs a GPT2 model to generate text based on input context.

## Requirements

In order to for the server to make use of your GPU (if you have one available) you'll need to compile it against the right
version of the C++ library: LibTorch.

You can download LibTorch from here: [https://pytorch.org/get-started/locally/](https://pytorch.org/get-started/locally/).

## Running the server

Once LibTorch is downloaded and unzipped, clone this repo and run the server with the path to unzipped LibTorch directory:

```bash
git clone https://github.com/epwalsh/rust-dl-webserver.git && cd rust-dl-webserver
make run LIBTORCH=/path/to/libtorch
```

Now in a separate terminal you can send send several requests in to the server at once:

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
