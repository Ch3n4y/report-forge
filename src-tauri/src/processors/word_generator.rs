use crate::models::{ExcelProcessResult, ReportConfig, RiskInfo, StatisticItem};
use anyhow::{Context, Result};
use docx_rs::*;
use std::path::Path;

pub struct WordGenerator;

impl WordGenerator {
    /// 生成完整报告
    pub fn generate_report(
        config: &ReportConfig,
        result_data: &ExcelProcessResult,
    ) -> Result<String> {
        log::info!("开始生成Word报告");

        // 创建文档
        let mut doc = Docx::new();

        // 生成统计表格
        let statistics = Self::generate_statistics(result_data);
        doc = Self::add_statistics_table(doc, &statistics)?;

        // 为每个分组生成报告内容
        let mut title_num = 1;
        for (group_key, group_info) in &result_data.grouped_data {
            // 生成报告编号
            let report_number = format!(
                "{}{}",
                config.identifier_tag,
                format!("{:04}", title_num + config.wt_add)
            );

            let parts: Vec<&str> = group_key.split('|').collect();
            let problem_name = parts.get(0).unwrap_or(&"");
            let severity = parts.get(1).unwrap_or(&"");

            let risk_info = RiskInfo::from_severity(severity);
            let title = format!("{}、{}", title_num, problem_name);

            // 生成相关代码文本
            let code_text = Self::generate_code_text(&group_info.records);
            let code_path_text = Self::generate_path_text(&group_info.records);

            // 获取第一条记录的详细信息
            let first_record = group_info.records.first();
            let phenomenon = first_record
                .and_then(|r| r.data.get("B"))
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");
            let vulnerability = first_record
                .and_then(|r| r.data.get("K"))
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");
            let suggestion = first_record
                .and_then(|r| r.data.get("N"))
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");

            // 添加报告内容
            doc = Self::add_report_section(
                doc,
                &report_number,
                &title,
                &config.code_version,
                &config.ceshi_user,
                &config.ceshi_time,
                &risk_info.text,
                phenomenon,
                &Self::clean_text(&code_path_text),
                &Self::clean_text(&code_text),
                vulnerability,
                suggestion,
            )?;

            log::info!(
                "已处理第 {}/{} 条记录",
                title_num,
                result_data.total_groups
            );
            title_num += 1;
        }

        // 生成输出文件路径
        let timestamp = chrono::Local::now().timestamp();
        let output_file = format!(
            "{}/{}_{}_{}.docx",
            config.output_dir,
            config.identifier_tag,
            config.code_version,
            timestamp
        );

        // 保存文档
        let path = Path::new(&output_file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("无法创建输出目录: {:?}", parent))?;
        }

        let file = std::fs::File::create(&output_file)
            .with_context(|| format!("无法创建输出文件: {}", output_file))?;

        doc.build()
            .pack(file)
            .with_context(|| "无法写入Word文档")?;

        log::info!("报告生成完成！文件: {}", output_file);
        Ok(output_file)
    }

    /// 生成统计信息
    fn generate_statistics(result_data: &ExcelProcessResult) -> Vec<StatisticItem> {
        let mut statistics = Vec::new();
        let mut seq_num = 1;

        for (_, group_info) in &result_data.grouped_data {
            let severity = if group_info.d_column.contains("高危") || group_info.d_column.contains("高") {
                "高"
            } else if group_info.d_column.contains("中危") || group_info.d_column.contains("中") {
                "中"
            } else if group_info.d_column.contains("低危") || group_info.d_column.contains("低") {
                "低"
            } else {
                "未知"
            };

            statistics.push(StatisticItem {
                seq_num,
                problem_name: group_info.b_column.clone(),
                severity_level: severity.to_string(),
                problem_count: group_info.record_count,
            });

            seq_num += 1;
        }

        statistics
    }

    /// 添加统计表格到文档
    fn add_statistics_table(mut doc: Docx, statistics: &[StatisticItem]) -> Result<Docx> {
        // 添加标题
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text("问题统计表格")
                        .size(32) // 小四 = 24, 这里用32表示16磅
                        .bold()
                        .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                )
                .align(AlignmentType::Center),
        );

        // 创建表头行 - 带样式
        let header_cells = vec![
            Self::create_header_cell("序号"),
            Self::create_header_cell("问题名称"),
            Self::create_header_cell("严重性级别"),
            Self::create_header_cell("问题个数"),
        ];

        // 创建表格，设置边框
        let mut table = Table::new(vec![TableRow::new(header_cells)])
            .set_grid(vec![1200, 4500, 1800, 1500]) // 调整列宽：序号窄，问题名称宽
            .align(TableAlignmentType::Center);

        // 添加数据行
        for stat in statistics {
            let row = TableRow::new(vec![
                Self::create_data_cell(&stat.seq_num.to_string()),
                Self::create_data_cell(&stat.problem_name),
                Self::create_data_cell(&stat.severity_level),
                Self::create_data_cell(&stat.problem_count.to_string()),
            ]);
            table = table.add_row(row);
        }

        doc = doc.add_table(table);
        doc = doc.add_paragraph(Paragraph::new()); // 空行

        Ok(doc)
    }

    /// 创建表头单元格 - 小四字体，宋体，加粗，居中
    fn create_header_cell(text: &str) -> TableCell {
        TableCell::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(text)
                            .size(24) // 小四 = 12磅 = 24半磅
                            .bold()
                            .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                    )
                    .align(AlignmentType::Center),
            )
            .vertical_align(VAlignType::Center)
            .shading(Shading::new().fill("D9E2F3")) // 浅蓝色背景
    }

    /// 创建数据单元格 - 小四字体，宋体，居中
    fn create_data_cell(text: &str) -> TableCell {
        TableCell::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(text)
                            .size(24) // 小四 = 12磅 = 24半磅
                            .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                    )
                    .align(AlignmentType::Center),
            )
            .vertical_align(VAlignType::Center)
    }

    /// 添加报告章节 - 使用指定的表格格式
    #[allow(clippy::too_many_arguments)]
    fn add_report_section(
        mut doc: Docx,
        report_number: &str,
        title: &str,
        code_version: &str,
        ceshi_user: &str,
        ceshi_time: &str,
        risk_text: &str,
        phenomenon: &str,
        code_path: &str,
        code: &str,
        vulnerability: &str,
        suggestion: &str,
    ) -> Result<Docx> {
        // 添加标题
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(title)
                        .size(28) // 标题字号稍大
                        .bold()
                        .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                )
                .style("Heading3"),
        );

        // 创建报告信息表格 (8行4列)
        let mut table = Table::new(vec![
            // 第1行：问题报告编号 | [编号] | 软件版本 | [版本]
            TableRow::new(vec![
                Self::create_label_cell("问题报告编号"),
                Self::create_content_cell(report_number),
                Self::create_label_cell("软件版本"),
                Self::create_content_cell(code_version),
            ]),
            // 第2行：测试人 | [测试人] | 测试时间 | [时间]
            TableRow::new(vec![
                Self::create_label_cell("测试人"),
                Self::create_content_cell(ceshi_user),
                Self::create_label_cell("测试时间"),
                Self::create_content_cell(ceshi_time),
            ]),
            // 第3行：问题描述 (跨3列)
            TableRow::new(vec![
                Self::create_label_cell("问题描述"),
                Self::create_multiline_cell(&format!(
                    "缺陷描述：\n{}\n\n{}",
                    phenomenon, code
                ))
                .grid_span(3),
            ]),
            // 第4行：问题严重性级别 (跨3列)
            TableRow::new(vec![
                Self::create_label_cell("问题严重性级别"),
                Self::create_content_cell(risk_text).grid_span(3),
            ]),
            // 第5行：相关文件路径 (跨3列)
            TableRow::new(vec![
                Self::create_label_cell("相关文件路径"),
                Self::create_multiline_cell(code_path).grid_span(3),
            ]),
            // 第6行：漏洞说明 (跨3列)
            TableRow::new(vec![
                Self::create_label_cell("漏洞说明"),
                Self::create_multiline_cell(vulnerability).grid_span(3),
            ]),
            // 第7行：整改建议 (跨3列)
            TableRow::new(vec![
                Self::create_label_cell("整改建议"),
                Self::create_multiline_cell(suggestion).grid_span(3),
            ]),
        ]);

        // 设置表格样式和列宽
        table = table
            .set_grid(vec![1800, 2800, 1800, 2800]) // 4列：标签-内容-标签-内容
            .align(TableAlignmentType::Center);

        doc = doc.add_table(table);

        // 添加空行作为分隔
        doc = doc.add_paragraph(Paragraph::new());

        Ok(doc)
    }

    /// 创建标签单元格 - 浅灰背景，加粗，居中
    fn create_label_cell(text: &str) -> TableCell {
        TableCell::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(text)
                            .size(24) // 小四
                            .bold()
                            .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                    )
                    .align(AlignmentType::Center),
            )
            .vertical_align(VAlignType::Center)
            .shading(Shading::new().fill("F2F2F2")) // 浅灰色背景
    }

    /// 创建内容单元格 - 普通文本，左对齐，垂直居中
    fn create_content_cell(text: &str) -> TableCell {
        TableCell::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(text)
                            .size(24) // 小四
                            .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                    )
                    .align(AlignmentType::Left),
            )
            .vertical_align(VAlignType::Center)
    }

    /// 创建多行内容单元格 - 支持换行，左对齐，顶部对齐
    fn create_multiline_cell(text: &str) -> TableCell {
        let mut cell = TableCell::new();

        // 处理换行符：Excel中的换行可能是\n, \r\n, 或 _x000D_
        let cleaned_text = text
            .replace("_x000D_", "\n")  // Excel特殊换行符
            .replace("\r\n", "\n")      // Windows换行符
            .replace('\r', "\n");       // Mac换行符

        // 按行分割文本
        let lines: Vec<&str> = cleaned_text.split('\n').collect();

        for (i, line) in lines.iter().enumerate() {
            // 跳过空行，但保留一些间距
            if line.trim().is_empty() && i > 0 {
                cell = cell.add_paragraph(Paragraph::new().add_run(Run::new().add_text(" ")));
                continue;
            }

            let mut para = Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(*line) // 解引用 &&str 为 &str
                        .size(24) // 小四
                        .fonts(RunFonts::new().east_asia("宋体").ascii("Times New Roman")),
                )
                .align(AlignmentType::Left);

            // 为段落间添加适当间距
            if i > 0 {
                para = para.line_spacing(LineSpacing::new().before(80).after(0));
            }

            cell = cell.add_paragraph(para);
        }

        cell.vertical_align(VAlignType::Top)
    }

    /// 生成相关代码文本
    fn generate_code_text(records: &[crate::models::ExcelRecord]) -> String {
        let mut code_text = String::new();
        for (i, record) in records.iter().enumerate() {
            let code = record
                .data
                .get("J")
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");
            code_text.push_str(&format!("缺陷{}相关代码如下：\r{}\r\n", i + 1, code));
        }
        code_text.trim().to_string()
    }

    /// 生成文件路径文本
    fn generate_path_text(records: &[crate::models::ExcelRecord]) -> String {
        let mut path_text = String::new();
        for (i, record) in records.iter().enumerate() {
            let path = record
                .data
                .get("I")
                .and_then(|v| v.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("")
                .trim_start_matches("root");
            path_text.push_str(&format!("缺陷{}文件路径：\r{}\r\n", i + 1, path));
        }
        path_text.trim().to_string()
    }

    /// 清理文本
    fn clean_text(text: &str) -> String {
        text.replace("_x000D_", "")
            .replace("      ", "    ")
            .trim()
            .to_string()
    }
}
