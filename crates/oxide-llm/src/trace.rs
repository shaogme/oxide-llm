use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crate::error::{AgentError, Result};

/// Configuration and state manager for tracing logs.
///
/// Tracing 日志的配置与状态管理器。
pub struct TraceState {
    log_dir: OnceLock<PathBuf>,
    enabled: AtomicBool,
    turn_counter: AtomicUsize,
    block_counter: AtomicUsize,
}

impl TraceState {
    /// Creates a new `TraceState` instance.
    ///
    /// 创建一个新的 `TraceState` 实例。
    pub const fn new() -> Self {
        Self {
            log_dir: OnceLock::new(),
            enabled: AtomicBool::new(true),
            turn_counter: AtomicUsize::new(0),
            block_counter: AtomicUsize::new(0),
        }
    }

    /// Returns whether tracing is enabled.
    ///
    /// 返回 tracing 是否已启用。
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Sets whether tracing is enabled.
    ///
    /// 设置 tracing 是否启用。
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }

    /// Returns the current turn counter.
    ///
    /// 返回当前对话轮次计数。
    pub fn turn_counter(&self) -> usize {
        self.turn_counter.load(Ordering::Acquire)
    }

    /// Advances to a new conversation turn and resets the block counter.
    ///
    /// 递增对话轮次计数并重置数据块计数。
    pub fn start_new_turn(&self) {
        self.turn_counter.fetch_add(1, Ordering::AcqRel);
        self.block_counter.store(0, Ordering::Release);
    }

    /// Increments and returns the next block index.
    ///
    /// 递增并返回下一个数据块索引。
    pub fn next_block_index(&self) -> usize {
        self.block_counter.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Returns or initializes the log directory.
    ///
    /// 获取或初始化日志目录。
    pub fn log_dir(&self) -> &PathBuf {
        self.log_dir.get_or_init(|| {
            let path = PathBuf::from("./tracing_logs");
            let _ = fs::create_dir_all(&path);
            path
        })
    }
}

impl Default for TraceState {
    fn default() -> Self {
        Self::new()
    }
}

static TRACE_STATE: TraceState = TraceState::new();

/// Initializes the tracing directory and configures logging state.
///
/// 初始化 tracing 目录并配置日志状态。
pub fn init_trace_dir(dir: impl AsRef<Path>, clean_existing: bool) -> Result<()> {
    let path = dir.as_ref();
    if clean_existing && path.exists() {
        fs::remove_dir_all(path).map_err(AgentError::Io)?;
    }
    fs::create_dir_all(path).map_err(AgentError::Io)?;

    let _ = TRACE_STATE.log_dir.set(path.to_path_buf());
    TRACE_STATE.set_enabled(true);
    Ok(())
}

/// Advances to a new conversation turn and resets the block counter.
///
/// 切换至新一轮对话并重置 block 计数器。
pub fn start_new_turn() {
    TRACE_STATE.start_new_turn();
}

/// Dumps an SSE raw block into a separate file and returns the file path.
///
/// 将 SSE 原始数据块 Dump 保存为独立文件，并返回对应的文件路径。
pub fn dump_sse_raw_block(block: &str) -> Result<PathBuf> {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return Err(AgentError::Trace(
            "Tracing subscriber not initialized or DEBUG level disabled".to_string(),
        ));
    }

    if !TRACE_STATE.is_enabled() {
        return Err(AgentError::Trace("Tracing disabled".to_string()));
    }

    let log_dir = TRACE_STATE.log_dir();
    let turn_idx = TRACE_STATE.turn_counter();
    let block_idx = TRACE_STATE.next_block_index();

    let sse_blocks_dir = log_dir
        .join(format!("turn_{:03}", turn_idx))
        .join("sse_blocks");
    if !sse_blocks_dir.exists() {
        fs::create_dir_all(&sse_blocks_dir).map_err(AgentError::Io)?;
    }

    let file_path = sse_blocks_dir.join(format!("{:04}.log", block_idx));
    fs::write(&file_path, block).map_err(AgentError::Io)?;

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_state() {
        start_new_turn();
        assert!(TRACE_STATE.turn_counter() >= 1);
    }
}
