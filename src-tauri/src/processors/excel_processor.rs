use crate::models::{ExcelProcessResult, ExcelRecord, GroupInfo, RiskInfo};
use anyhow::{Context, Result};
use calamine::{open_workbook, Reader, Xlsx};
use std::collections::HashMap;
use std::path::Path;

pub struct ExcelProcessor;

/// Excel原始数据结构
#[derive(Debug, Clone)]
pub struct RawExcelData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl ExcelProcessor {
    /// 读取Excel文件的原始数据（不进行去重和分组）
    pub fn read_excel_raw<P: AsRef<Path>>(excel_file: P) -> Result<RawExcelData> {
        let excel_file = excel_file.as_ref();
        log::info!("读取Excel文件原始数据: {:?}", excel_file);

        // 打开Excel文件
        let mut workbook: Xlsx<_> = open_workbook(excel_file)
            .with_context(|| format!("无法打开Excel文件: {:?}", excel_file))?;

        // 获取第一个工作表
        let sheet_name = workbook
            .sheet_names()
            .first()
            .context("Excel文件中没有工作表")?
            .clone();

        let range = workbook
            .worksheet_range(&sheet_name)
            .context("无法读取工作表")?;

        log::info!("工作表尺寸: {:?}", range.get_size());

        // 转换为行数据
        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| {
                row.iter()
                    .map(|cell| cell.to_string())
                    .collect()
            })
            .collect();

        if rows.is_empty() {
            anyhow::bail!("Excel文件为空");
        }

        if rows.len() <= 1 {
            anyhow::bail!("Excel文件只有表头，没有数据行");
        }

        // 第一行是表头
        let headers = rows[0].clone();
        let data_rows = rows[1..].to_vec();

        log::info!("表头列数: {}, 数据行数: {}", headers.len(), data_rows.len());

        Ok(RawExcelData {
            headers,
            rows: data_rows,
        })
    }

    /// 合并多个Excel文件的原始数据，验证表头一致性
    pub fn merge_excel_files<P: AsRef<Path>>(excel_files: &[P]) -> Result<RawExcelData> {
        if excel_files.is_empty() {
            anyhow::bail!("没有提供Excel文件");
        }

        log::info!("开始合并 {} 个Excel文件", excel_files.len());

        // 读取第一个文件作为基准
        let first_data = Self::read_excel_raw(&excel_files[0])?;
        let mut merged_rows = first_data.rows.clone();
        let reference_headers = first_data.headers.clone();

        log::info!("基准表头: {:?}", reference_headers);

        // 逐个读取并合并其他文件
        for (index, excel_file) in excel_files.iter().enumerate().skip(1) {
            let current_data = Self::read_excel_raw(excel_file)?;

            // 验证表头是否一致
            if current_data.headers.len() != reference_headers.len() {
                anyhow::bail!(
                    "文件 {} 的表头列数({})与第一个文件({})不一致",
                    excel_file.as_ref().display(),
                    current_data.headers.len(),
                    reference_headers.len()
                );
            }

            // 验证每一列的表头内容是否一致
            for (i, (current_header, reference_header)) in current_data.headers.iter()
                .zip(reference_headers.iter())
                .enumerate()
            {
                if current_header.trim() != reference_header.trim() {
                    anyhow::bail!(
                        "文件 {} 的第{}列表头(\"{}\")与第一个文件(\"{}\")不一致",
                        excel_file.as_ref().display(),
                        i + 1,
                        current_header,
                        reference_header
                    );
                }
            }

            // 表头一致，合并数据行
            log::info!("文件 {} 表头验证通过，合并 {} 行数据", index + 1, current_data.rows.len());
            merged_rows.extend(current_data.rows);
        }

        log::info!("合并完成！总数据行数: {}", merged_rows.len());

        Ok(RawExcelData {
            headers: reference_headers,
            rows: merged_rows,
        })
    }

    /// 从合并后的原始数据处理为结构化结果
    pub fn process_raw_data(raw_data: RawExcelData) -> Result<ExcelProcessResult> {
        log::info!("开始处理合并后的数据");

        let rows = raw_data.rows;

        // 创建列名（A-P）
        let column_count = if !rows.is_empty() {
            rows[0].len()
        } else {
            0
        };
        let column_names: Vec<String> = (0..column_count)
            .map(|i| format!("{}", (b'A' + i as u8) as char))
            .collect();

        log::info!("列数: {}, 列名: {:?}", column_count, column_names);

        // 转换为记录格式
        let mut records: Vec<HashMap<String, Option<String>>> = Vec::new();

        for row in &rows {
            let mut record = HashMap::new();
            for (i, value) in row.iter().enumerate() {
                if i < column_names.len() {
                    let col_name: &String = &column_names[i];
                    let cleaned = value.trim();
                    record.insert(
                        col_name.clone(),
                        if cleaned.is_empty() {
                            None
                        } else {
                            Some(cleaned.to_string())
                        },
                    );
                }
            }
            records.push(record);
        }

        log::info!("转换后记录数: {}", records.len());

        // 基于前7列（A-G）去重
        let before_dedup = records.len();
        records = Self::deduplicate_records(&records, &column_names[..7.min(column_names.len())]);
        let after_dedup = records.len();

        log::info!("去重前记录数: {}, 去重后记录数: {}", before_dedup, after_dedup);

        // 按B列和D列分组
        let grouped_data = Self::group_data_by_columns(&records, "B", "D");

        // 创建结构化结果
        let result = Self::create_structured_result(grouped_data, records.len());

        log::info!(
            "处理完成！总记录数: {}, 分组数: {}",
            result.total_records,
            result.total_groups
        );

        Ok(result)
    }

    /// 处理Excel文件并返回结构化结果（保留向后兼容）
    pub fn process_excel_to_json<P: AsRef<Path>>(excel_file: P) -> Result<ExcelProcessResult> {
        // 读取原始数据
        let raw_data = Self::read_excel_raw(excel_file)?;
        // 处理原始数据
        Self::process_raw_data(raw_data)
    }

    /// 基于指定列去重
    fn deduplicate_records(
        records: &[HashMap<String, Option<String>>],
        check_columns: &[String],
    ) -> Vec<HashMap<String, Option<String>>> {
        let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut unique_records = Vec::new();

        for record in records {
            // 创建组合键
            let key: String = check_columns
                .iter()
                .map(|col| {
                    record
                        .get(col)
                        .and_then(|v| v.as_ref())
                        .map(|s| s.as_str())
                        .unwrap_or("")
                })
                .collect::<Vec<&str>>()
                .join("|");

            if seen_keys.insert(key) {
                unique_records.push(record.clone());
            }
        }

        unique_records
    }

    /// 按指定列分组数据
    fn group_data_by_columns(
        records: &[HashMap<String, Option<String>>],
        col_b: &str,
        col_d: &str,
    ) -> HashMap<String, Vec<HashMap<String, Option<String>>>> {
        let mut grouped: HashMap<String, Vec<HashMap<String, Option<String>>>> = HashMap::new();

        for record in records {
            let key_b = record
                .get(col_b)
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let key_d = record
                .get(col_d)
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let group_key = format!("{}|{}", key_b, key_d);

            grouped
                .entry(group_key)
                .or_insert_with(Vec::new)
                .push(record.clone());
        }

        grouped
    }

    /// 创建结构化结果
    fn create_structured_result(
        grouped_data: HashMap<String, Vec<HashMap<String, Option<String>>>>,
        total_records: usize,
    ) -> ExcelProcessResult {
        // 创建每个组的结构化数据
        let mut grouped_structured: Vec<(String, GroupInfo, i32)> = Vec::new();

        for (group_key, records) in grouped_data {
            let parts: Vec<&str> = group_key.split('|').collect();
            let b_value = parts.get(0).unwrap_or(&"").to_string();
            let d_value = parts.get(1).unwrap_or(&"").to_string();

            let risk_info = RiskInfo::from_severity(&d_value);

            let group_info = GroupInfo {
                b_column: b_value,
                d_column: d_value,
                record_count: records.len(),
                records: records
                    .into_iter()
                    .map(|data| ExcelRecord { data })
                    .collect(),
            };

            grouped_structured.push((group_key, group_info, risk_info.priority));
        }

        // 按风险等级和记录数排序
        grouped_structured.sort_by(|a, b| {
            match a.2.cmp(&b.2) {
                std::cmp::Ordering::Equal => b.1.record_count.cmp(&a.1.record_count),
                other => other,
            }
        });

        // 移除优先级信息
        let grouped_data: Vec<(String, GroupInfo)> = grouped_structured
            .into_iter()
            .map(|(key, info, _)| (key, info))
            .collect();

        ExcelProcessResult {
            total_groups: grouped_data.len(),
            total_records,
            grouped_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_from_severity() {
        use crate::models::RiskLevel;

        assert_eq!(RiskLevel::from_severity("高危"), RiskLevel::High);
        assert_eq!(RiskLevel::from_severity("中危"), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_severity("低危"), RiskLevel::Low);
        assert_eq!(RiskLevel::from_severity("未知"), RiskLevel::Unknown);
    }
}
