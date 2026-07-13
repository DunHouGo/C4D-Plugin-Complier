//! HTTP 下载客户端的统一调优。

use std::time::Duration;

pub const DOWNLOAD_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 C4D-Plugin-Compiler/0.1";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(900);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);

/// 配置阻塞下载客户端，使 SDK 下载和浏览器网络路径更接近。
pub fn configure_blocking_client(
    builder: reqwest::blocking::ClientBuilder,
) -> reqwest::blocking::ClientBuilder {
    builder
        .user_agent(DOWNLOAD_USER_AGENT)
        .timeout(DOWNLOAD_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .http2_adaptive_window(true)
        .tcp_keepalive(KEEPALIVE_TIMEOUT)
        .tcp_nodelay(true)
}
