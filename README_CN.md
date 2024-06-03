[English](./README.md) | 中文

# dify-openai-apis

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Description

适用于 Dify 平台服务的 OpenAI 兼容 API。  
这个库提供了一套与 OpenAI 的 GPT-3 API 兼容的 API，可以用来与 Dify 的平台服务和工具进行交互。

**注意：** 该应用程序目前不支持 OpenAI 的[Legacy Completions API](https://platform.openai.com/docs/api-reference/completions/create)。请使用[Chat Completion API](https://platform.openai.com/docs/api-reference/chat/create)。

## Config

配置可以通过 .env 文件或环境变量进行设置：

- `HOST`：绑定服务器的主机。默认值：`127.0.0.1`
- `PORT`：绑定服务器的端口。默认值：`3000`
- `DIFY_BASE_URL`：Dify API 的基础 URL。默认值：`https://api.dify.ai`
- `DIFY_API_KEY`：Dify API 的 API 密钥。默认值：`your_api_key`
- `DIFY_TIMEOUT`：向 Dify API 发送请求的超时时间。默认值：`10`
- `WORKERS_NUM`：要使用的工作线程数量。默认值：`4`
- `RUST_LOG`：服务器的日志级别。默认值：`error`

**注意：**

- `DIFY_API_KEY` 是默认 API 密钥，如果用户在请求 API `/v1/chat/completions` 时通过 Bearer Token 传递了 API 密钥，则将覆盖此默认值。
- `RUST_LOG` 是日志级别，默认值为 `error`，即只输出错误日志。如果要调试运行，建议设置为 `debug` 或 `trace`。

## Install

请到发布页面下载预编译版本：[Release page](https://github.com/rming/dify-openai-apis/releases)

也可以通过 `cargo` 命令安装

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
