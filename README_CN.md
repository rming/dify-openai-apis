[English](./README.md) | 中文

# dify-openai-apis

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Description

适用于 Dify 平台服务的 OpenAI 兼容 API。  
这个库提供了一套与 OpenAI 的 GPT-3 API 兼容的 API，可以用来与 Dify 的平台服务和工具进行交互。

## Config

配置可以通过 .env 文件或环境变量进行设置：

- `HOST`：绑定服务器的主机。默认值：`127.0.0.1`
- `PORT`：绑定服务器的端口。默认值：`3000`
- `DIFY_BASE_URL`：Dify API 的基础 URL。默认值：`https://api.dify.ai`
- `DIFY_API_KEY`：Dify API 的 API 密钥。默认值：`your_api_key`
- `DIFY_TIMEOUT`：向 Dify API 发送请求的超时时间。默认值：`10`
- `WORKERS_NUM`：要使用的工作线程数量。默认值：`4`
- `RUST_LOG`：服务器的日志级别。默认值：`error`

## Install

```sh
# require cargo installed
cargo install dify-openai-apis
```

## Usage

要启动服务器，请运行：

```sh
# require cargo bin directory in PATH
# export PATH=$HOME/.cargo/bin:$PATH
dify-openai-apis
```