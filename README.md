English | [中文](./README_CN.md)

# dify-openai-apis

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Description

OpenAI-compatible APIs for Dify platform services.  
This crate provides a set of APIs that are compatible with OpenAI's GPT-3 API, and can be used to interact with Dify's platform services and tools.

**Note:** The app currently does not support OpenAI's [Legacy Completions API](https://platform.openai.com/docs/api-reference/completions/create). Please use the [Chat Completion API](https://platform.openai.com/docs/api-reference/chat/create) instead.

## Config

Configurations can be set via .env file or environment variables:

- `HOST`: The host to bind the server to. Default: `127.0.0.1`
- `PORT`: The port to bind the server to. Default: `3000`
- `DIFY_BASE_URL`: The base URL of Dify's API. Default: `https://api.dify.ai`
- `DIFY_API_KEY`: Your API key for Dify's API. Default: `your_api_key`
- `DIFY_TIMEOUT`: The timeout for requests to Dify's API. Default: `10`
- `WORKERS_NUM`: The number of worker threads to use. Default: `4`
- `RUST_LOG`: The log level for the server. Default: `error`

**Note:**

- `DIFY_API_KEY` is the default API key. If a user provides an API key via Bearer Token when requesting the API `/v1/chat/completions`, it will override this default value.
- `RUST_LOG` is the log level, with a default value of `error`, meaning only error logs will be output. If you want to debug, it is recommended to set it to `debug` or `trace`.

## Install

Please download the precompiled binary from : [Release page](https://github.com/rming/dify-openai-apis/releases)

You can also install it using the `cargo` command.

```sh
# require cargo installed
cargo install dify-openai-apis
```

## Usage

To start the server, run:

```sh
# require cargo bin directory in PATH
# export PATH=$HOME/.cargo/bin:$PATH
dify-openai-apis
```
