use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 风险等级枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
    Unknown,
}

impl RiskLevel {
    /// 从严重性字符串获取风险等级
    pub fn from_severity(severity: &str) -> Self {
        if severity.contains("高危") || severity.contains("高") {
            RiskLevel::High
        } else if severity.contains("中危") || severity.contains("中") {
            RiskLevel::Medium
        } else if severity.contains("低危") || severity.contains("低") {
            RiskLevel::Low
        } else {
            RiskLevel::Unknown
        }
    }

    /// 获取风险等级优先级（用于排序）
    pub fn priority(&self) -> i32 {
        match self {
            RiskLevel::High => 1,
            RiskLevel::Medium => 2,
            RiskLevel::Low => 3,
            RiskLevel::Unknown => 999,
        }
    }

    /// 获取风险等级文本（带复选框）
    pub fn text(&self) -> String {
        match self {
            RiskLevel::High => "☑ 高危风险  ☐ 中危风险  ☐ 低危风险".to_string(),
            RiskLevel::Medium => "☐ 高危风险  ☑ 中危风险  ☐ 低危风险".to_string(),
            RiskLevel::Low => "☐ 高危风险  ☐ 中危风险  ☑ 低危风险".to_string(),
            RiskLevel::Unknown => "☐ 高危风险  ☐ 中危风险  ☐ 低危风险".to_string(),
        }
    }
}

/// 风险信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskInfo {
    pub level: RiskLevel,
    pub text: String,
    pub priority: i32,
}

impl RiskInfo {
    pub fn from_severity(severity: &str) -> Self {
        let level = RiskLevel::from_severity(severity);
        let text = level.text();
        let priority = level.priority();

        RiskInfo {
            level,
            text,
            priority,
        }
    }
}

/// 报告配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub excel_files: Vec<String>,
    pub template_file: String,
    pub output_dir: String,
    pub identifier_tag: String,
    pub wt_add: i32,
    pub ceshi_time: String,
    pub code_version: String,
    pub ceshi_user: String,
}

/// Excel记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelRecord {
    pub data: HashMap<String, Option<String>>,
}

/// 分组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub b_column: String,       // 问题名称
    pub d_column: String,       // 严重性级别
    pub record_count: usize,
    pub records: Vec<ExcelRecord>,
}

/// Excel处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelProcessResult {
    pub total_groups: usize,
    pub total_records: usize,
    pub grouped_data: Vec<(String, GroupInfo)>,  // 保持顺序的分组数据
}

/// 统计项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticItem {
    pub seq_num: usize,
    pub problem_name: String,
    pub severity_level: String,
    pub problem_count: usize,
}

/// 进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub percentage: f32,
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// 日志消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
}
