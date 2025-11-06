# ReportForge - 代码审计报告生成器

基于 Tauri + Rust + Vue + Element Plus 的代码审计报告生成工具

## 项目简介

ReportForge 是一个将 Python 数据处理应用迁移到 Tauri + Rust 技术栈的GUI应用，专门用于生成代码审计报告。

### 技术栈

**前端:**
- Tauri 2.x
- Vue 3 + TypeScript
- Element Plus UI 框架
- Vite 构建工具

**后端 (Rust):**
- `calamine` - Excel文件解析
- `docx-rs` - Word文档生成
- `serde` - 序列化/反序列化
- `tokio` - 异步运行时
- `anyhow` - 错误处理

## 项目结构

```
report-forge/
├── src/                    # Vue前端源码
│   ├── App.vue            # 主应用组件
│   └── main.ts            # 入口文件
├── src-tauri/             # Rust后端源码
│   ├── src/
│   │   ├── commands/      # Tauri命令
│   │   │   ├── mod.rs
│   │   │   └── report_commands.rs
│   │   ├── models/        # 数据模型
│   │   │   └── mod.rs
│   │   ├── processors/    # 处理模块
│   │   │   ├── mod.rs
│   │   │   ├── excel_processor.rs
│   │   │   └── word_generator.rs
│   │   ├── lib.rs         # 库入口
│   │   └── main.rs        # 程序入口
│   ├── Cargo.toml         # Rust依赖配置
│   └── tauri.conf.json    # Tauri配置
├── package.json           # 前端依赖配置
└── README.md              # 项目文档
```

## 核心功能

### 1. Excel 数据处理
- 读取 .xlsx 文件
- 自动去重（基于前7列）
- 按问题类型和严重性分组
- 风险等级自动识别

### 2. Word 报告生成
- 自动生成统计表格
- 创建详细的问题报告
- 格式化输出
- 支持多文件合并

### 3. 用户界面
- 拖拽式文件上传
- 表单配置
- 实时进度显示
- 日志输出面板

### 4. 风险等级管理
- 高危、中危、低危自动识别
- 按风险等级和数量排序
- 可视化风险标记

## 快速开始

### 环境要求
- Node.js 18+
- Rust 1.70+
- pnpm (推荐) 或 npm

### 安装依赖

```bash
# 进入项目目录
cd report-forge

# 安装前端依赖
pnpm install

# Rust依赖会在构建时自动下载
```

### 开发模式

```bash
# 启动开发服务器（热重载）
pnpm tauri dev
```

### 生产构建

```bash
# 构建生产版本
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/`

## 使用说明

### 1. 准备输入文件
- Excel文件（.xlsx格式）
- 确保Excel格式符合要求

### 2. 配置参数
- **Excel文件**: 选择一个或多个Excel文件
- **输出目录**: 选择报告输出位置
- **标识号前缀**: 如 `SZ25QT9B00WT`
- **问题编号起始**: 起始编号偏移量
- **测试时间**: 测试日期
- **代码版本**: 如 `V1.0`
- **测试人员**: 测试人员姓名

### 3. 生成报告
- 点击"开始生成报告"按钮
- 查看实时进度和日志
- 生成完成后自动打开报告文件

## 数据流程

```
Excel输入文件
    ↓
[Excel处理器]
    ├─ 读取数据
    ├─ 去重处理
    └─ 分组统计
    ↓
[数据结构化]
    ├─ 风险等级识别
    ├─ 优先级排序
    └─ 数据整合
    ↓
[Word生成器]
    ├─ 统计表格
    ├─ 详细报告
    └─ 格式化
    ↓
输出Word文档
```

## API 命令

### Tauri Commands

#### `process_excel_file`
处理单个Excel文件

```typescript
invoke('process_excel_file', {
  filePath: string
}) => Promise<ExcelProcessResult>
```

#### `generate_report`
生成完整报告

```typescript
invoke('generate_report', {
  config: ReportConfig
}) => Promise<string>
```

#### `get_logs`
获取日志

```typescript
invoke('get_logs') => Promise<LogMessage[]>
```

#### `get_progress`
获取进度

```typescript
invoke('get_progress') => Promise<ProgressInfo | null>
```

## 配置说明

### ReportConfig 结构

```typescript
interface ReportConfig {
  excel_files: string[];      // Excel文件路径列表
  template_file: string;      // 模板文件路径（暂未使用）
  output_dir: string;         // 输出目录
  identifier_tag: string;     // 标识号前缀
  wt_add: number;            // 问题编号偏移
  ceshi_time: string;        // 测试时间
  code_version: string;      // 代码版本
  ceshi_user: string;        // 测试人员
}
```

## 开发指南

### 添加新功能

1. **后端 (Rust)**
   - 在 `models/` 添加数据模型
   - 在 `processors/` 添加处理逻辑
   - 在 `commands/` 添加Tauri命令

2. **前端 (Vue)**
   - 在 `src/` 添加组件
   - 使用 `invoke()` 调用后端命令

### 错误处理

- 使用 `anyhow::Result` 处理Rust错误
- 前端使用 `try-catch` 捕获异常
- 通过日志系统记录错误

## 性能优化

- 异步文件处理（tokio）
- 增量进度更新
- 内存高效的数据结构
- 批量处理优化

## 故障排除

### 编译错误
```bash
# 清理构建缓存
cd src-tauri
cargo clean

# 重新构建
cargo build
```

### 前端问题
```bash
# 清理node_modules
rm -rf node_modules
pnpm install
```

## 推荐IDE配置

- [VS Code](https://code.visualstudio.com/)
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 贡献

欢迎提交 Issue 和 Pull Request

## 许可证

MIT License

## 联系方式

- 项目地址: ReportForge
- 作者: Chen
