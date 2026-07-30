use serde::{Deserialize, Serialize};

use super::request::CreateInteractionRequest;

/// Trigger resource in Gemini Interactions API.
///
/// Gemini Interactions API 中的 Trigger 资源。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    /// Required. Unique identifier for the trigger.
    ///
    /// 必填。Trigger 的唯一标识符。
    pub id: String,
    /// Required. Cron schedule in standard cron format.
    ///
    /// 必填。标准 cron 格式的定时调度表达式。
    pub schedule: String,
    /// Required. Time zone in which schedule is interpreted.
    ///
    /// 必填。解释调度的时区。
    pub time_zone: String,
    /// Required. Interaction request template.
    ///
    /// 必填。要执行的 Interaction 请求模板。
    pub interaction: CreateInteractionRequest,
    /// Display name of trigger.
    ///
    /// Trigger 的显示名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Environment ID for trigger execution.
    ///
    /// Trigger 执行的环境 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    /// Execution timeout in seconds.
    ///
    /// 执行超时时间（秒）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_timeout_seconds: Option<i32>,
    /// Max consecutive failures allowed before automatic pause.
    ///
    /// 自动暂停前允许的最大连续失败次数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_consecutive_failures: Option<i32>,
    /// Status of trigger ('active', 'paused', 'error').
    ///
    /// Trigger 状态（'active', 'paused', 'error'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TriggerStatus>,
    /// Time when trigger was created.
    ///
    /// Trigger 创建时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    /// Time when trigger was last updated.
    ///
    /// Trigger 最近更新时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
    /// Time when trigger was last run.
    ///
    /// Trigger 上次运行时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_time: Option<String>,
    /// Time when trigger is scheduled to run next.
    ///
    /// Trigger 下次预计运行时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run_time: Option<String>,
    /// ID of last interaction created by trigger.
    ///
    /// 该 Trigger 创建的最新 Interaction 的 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_interaction_id: Option<String>,
}

/// Trigger status ('active', 'paused', 'error').
///
/// Trigger 状态（'active', 'paused', 'error'）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TriggerStatus {
    Active,
    Paused,
    Error,
}

/// Parameters for creating a trigger.
///
/// 创建 Trigger 的参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCreateParams {
    /// Required. Cron schedule in standard cron format.
    ///
    /// 必填。标准 cron 格式的定时调度表达式。
    pub schedule: String,
    /// Required. Time zone in which schedule is interpreted.
    ///
    /// 必填。解释调度的时区。
    pub time_zone: String,
    /// Required. Interaction request template to execute.
    ///
    /// 必填。要执行的 Interaction 请求模板。
    pub interaction: CreateInteractionRequest,
    /// Display name of trigger.
    ///
    /// Trigger 的显示名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Environment ID for execution.
    ///
    /// 执行的环境 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    /// Execution timeout in seconds.
    ///
    /// 执行超时时间（秒）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_timeout_seconds: Option<i32>,
    /// Max consecutive failures allowed.
    ///
    /// 允许的最大连续失败次数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_consecutive_failures: Option<i32>,
}

/// Parameters for updating a trigger.
///
/// 更新 Trigger 的参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriggerUpdate {
    /// Display name of trigger.
    ///
    /// Trigger 的显示名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Status of trigger ('active', 'paused', 'error').
    ///
    /// Trigger 状态（'active', 'paused', 'error'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TriggerStatus>,
}

/// Execution instance of a trigger.
///
/// Trigger 的执行实例。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerExecution {
    /// ID of the trigger execution.
    ///
    /// Trigger 执行的 ID。
    pub id: String,
    /// ID of trigger that created execution.
    ///
    /// 创建此执行的 Trigger 的 ID。
    pub trigger_id: String,
    /// Status of execution ('in_progress', 'completed', 'failed', 'skipped', 'timed_out').
    ///
    /// 执行状态（'in_progress', 'completed', 'failed', 'skipped', 'timed_out'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ExecutionStatus>,
    /// Interaction ID created by execution.
    ///
    /// 此执行创建的 Interaction ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_id: Option<String>,
    /// Environment ID used for execution.
    ///
    /// 用于执行的环境 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    /// Error message if execution failed.
    ///
    /// 执行失败时的错误消息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Scheduled run time.
    ///
    /// 计划运行时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_time: Option<String>,
    /// Execution start time.
    ///
    /// 执行开始时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Execution end time.
    ///
    /// 执行结束时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// Status of trigger execution.
///
/// Trigger 执行的状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    InProgress,
    Completed,
    Failed,
    Skipped,
    TimedOut,
}

/// Response for listing triggers.
///
/// 列出 Trigger 的响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTriggersResponse {
    /// Array of triggers.
    ///
    /// Trigger 列表。
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    /// Next page pagination token.
    ///
    /// 下一页的分页 Token。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Response for listing trigger executions.
///
/// 列出 Trigger 执行的响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTriggerExecutionsResponse {
    /// Array of trigger executions.
    ///
    /// Trigger 执行列表。
    #[serde(default)]
    pub trigger_executions: Vec<TriggerExecution>,
    /// Next page pagination token.
    ///
    /// 下一页的分页 Token。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}
