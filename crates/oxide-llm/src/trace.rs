use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::{AgentError, Result};

/// Configuration and state manager for tracing logs.
///
/// Tracing 日志的配置与状态管理器。
pub struct TraceState {
    log_dir: PathBuf,
    enabled: bool,
    turn_counter: AtomicUsize,
    block_counter: AtomicUsize,
}

impl TraceState {
    fn new(log_dir: PathBuf, enabled: bool) -> Self {
        Self {
            log_dir,
            enabled,
            turn_counter: AtomicUsize::new(0),
            block_counter: AtomicUsize::new(0),
        }
    }
}

static TRACE_STATE: RwLock<Option<TraceState>> = RwLock::new(None);

/// Initializes the tracing directory and configures logging state.
///
/// 初始化 tracing 目录并配置日志状态。
pub fn init_trace_dir(dir: impl AsRef<Path>, clean_existing: bool) -> Result<()> {
    let path = dir.as_ref();
    if clean_existing && path.exists() {
        fs::remove_dir_all(path).map_err(AgentError::Io)?;
    }
    fs::create_dir_all(path).map_err(AgentError::Io)?;

    let mut state_guard = TRACE_STATE
        .write()
        .map_err(|e| AgentError::Trace(e.to_string()))?;
    *state_guard = Some(TraceState::new(path.to_path_buf(), true));
    Ok(())
}

/// Advances to a new conversation turn and resets the block counter.
///
/// 切换至新一轮对话并重置 block 计数器。
pub fn start_new_turn() {
    if let Ok(guard) = get_or_init_state() {
        if let Some(state) = guard.as_ref() {
            state.turn_counter.fetch_add(1, Ordering::SeqCst);
            state.block_counter.store(0, Ordering::SeqCst);
        }
    }
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

    let guard = get_or_init_state()?;

    let state = guard
        .as_ref()
        .ok_or_else(|| AgentError::Trace("TraceState unavailable".to_string()))?;

    if !state.enabled {
        return Err(AgentError::Trace("Tracing disabled".to_string()));
    }

    let turn_idx = state.turn_counter.load(Ordering::SeqCst);
    let block_idx = state.block_counter.fetch_add(1, Ordering::SeqCst) + 1;

    let sse_blocks_dir = state
        .log_dir
        .join(format!("turn_{:03}", turn_idx))
        .join("sse_blocks");
    if !sse_blocks_dir.exists() {
        fs::create_dir_all(&sse_blocks_dir).map_err(AgentError::Io)?;
    }

    let file_path = sse_blocks_dir.join(format!("{:04}.log", block_idx));
    fs::write(&file_path, block).map_err(AgentError::Io)?;

    Ok(file_path)
}

fn get_or_init_state() -> Result<std::sync::RwLockReadGuard<'static, Option<TraceState>>> {
    let guard = TRACE_STATE
        .read()
        .map_err(|e| AgentError::Trace(e.to_string()))?;

    if guard.is_none() {
        drop(guard);
        let mut write_guard = TRACE_STATE
            .write()
            .map_err(|e| AgentError::Trace(e.to_string()))?;
        if write_guard.is_none() {
            let path = PathBuf::from("./tracing_logs");
            let _ = fs::create_dir_all(&path);
            *write_guard = Some(TraceState::new(path, true));
        }
        drop(write_guard);
        return TRACE_STATE
            .read()
            .map_err(|e| AgentError::Trace(e.to_string()));
    }
    Ok(guard)
}
