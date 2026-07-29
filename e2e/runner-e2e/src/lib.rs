/// End-to-End (E2E) Test suite for `oxide-llm` utilizing TypeScript Mock Server.
///
/// `oxide-llm` 使用 TypeScript Mock Server 的端到端 (E2E) 测试套件。
use std::{
    path::PathBuf,
    process::{Child, Command},
    thread::sleep,
    time::Duration,
};

pub mod non_stream;
pub mod stream;
pub mod tool_loop;

/// Process guard that launches and stops the TypeScript Mock Server automatically.
///
/// 自动启动和停止 TypeScript Mock Server 的进程守护结构体。
pub struct MockServerGuard {
    process: Child,
    pub base_url: String,
}

impl MockServerGuard {
    /// Launch the TypeScript mock server on the specified port.
    ///
    /// 在指定端口启动 TypeScript Mock Server。
    pub fn start(port: u16) -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let server_dir = manifest_dir
            .parent()
            .expect("Parent dir of runner_e2e")
            .join("server");

        #[cfg(target_os = "windows")]
        let mut cmd = Command::new("cmd");
        #[cfg(target_os = "windows")]
        cmd.args(["/C", "npx", "tsx", "src/index.ts"]);

        #[cfg(not(target_os = "windows"))]
        let mut cmd = Command::new("npx");
        #[cfg(not(target_os = "windows"))]
        cmd.args(["tsx", "src/index.ts"]);

        let process = cmd
            .current_dir(&server_dir)
            .env("PORT", port.to_string())
            .spawn()
            .expect("Failed to start TypeScript Mock Server process");

        let guard = Self {
            process,
            base_url: format!("http://127.0.0.1:{port}"),
        };

        // Wait briefly for server to listen
        sleep(Duration::from_millis(2000));
        guard
    }
}

impl Drop for MockServerGuard {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

use oxide_llm::{core::tool::Tool, macros::Schema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Schema)]
pub struct WeatherArgs {
    pub location: String,
    pub unit: Option<String>,
}

#[derive(Serialize)]
pub struct WeatherOutput {
    pub temperature: i32,
}

#[derive(Clone)]
pub struct WeatherTool;

impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";
    const DESCRIPTION: &'static str = "Get current weather";

    type Args = WeatherArgs;
    type Output = WeatherOutput;
    type Error = String;
    type Future = std::future::Ready<Result<Self::Output, Self::Error>>;

    fn run(&self, args: Self::Args) -> Self::Future {
        let _loc = args.location;
        let _unit = args.unit;
        std::future::ready(Ok(WeatherOutput { temperature: 25 }))
    }
}
