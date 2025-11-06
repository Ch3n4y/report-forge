# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ReportForge is a Tauri-based desktop application for generating code audit reports. It processes Excel files containing security findings and generates formatted Word documents with statistics and detailed reports. The application is built with:
- **Frontend**: Vue 3 + TypeScript + Element Plus + Vite
- **Backend**: Rust (Tauri) with calamine for Excel processing and docx-rs for Word generation

## Development Commands

### Running the Application
```bash
# Start development server with hot reload
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Frontend Development
```bash
# Start Vite dev server only (without Tauri)
pnpm dev

# Type check
pnpm build  # Runs vue-tsc --noEmit && vite build
```

### Rust Development
```bash
# From src-tauri directory
cd src-tauri

# Check Rust code
cargo check

# Build Rust backend only
cargo build

# Clean build artifacts
cargo clean
```

## Architecture

### Application Flow
1. **User Input**: User selects Excel files and configures report parameters (identifier, test date, code version, etc.)
2. **Excel Processing**: Rust backend reads Excel files using calamine, deduplicates data based on first 7 columns, groups by problem type (column B) and severity (column D)
3. **Data Merging**: Multiple Excel files are merged into a single result set
4. **Word Generation**: docx-rs generates a Word document with statistics table and detailed findings
5. **Progress Tracking**: Real-time progress updates and logs via Tauri state management

### Rust Backend Structure

**Module Organization:**
- `src-tauri/src/lib.rs` - Application entry point, registers Tauri commands and initializes state
- `src-tauri/src/commands/report_commands.rs` - Tauri commands exposed to frontend, manages AppState for logs and progress
- `src-tauri/src/models/mod.rs` - Data structures (ReportConfig, ExcelProcessResult, GroupInfo, RiskInfo, etc.)
- `src-tauri/src/processors/excel_processor.rs` - Excel file parsing, deduplication, and grouping logic
- `src-tauri/src/processors/word_generator.rs` - Word document generation with tables and formatting

**State Management:**
The `AppState` struct (in report_commands.rs) maintains:
- `logs: Mutex<Vec<LogMessage>>` - Application logs with levels (Info, Warning, Error, Success)
- `progress: Mutex<Option<ProgressInfo>>` - Current progress state (current/total/percentage/message)

### Frontend Structure

**Single-page application in `src/App.vue`:**
- File upload component using Element Plus `el-upload`
- Form configuration for report parameters
- Real-time log display panel
- Progress tracking with percentage display
- Tauri command invocation using `@tauri-apps/api`

### Data Models

**ReportConfig** - Configuration for report generation:
- `excel_files`: Array of Excel file paths
- `output_dir`: Output directory for generated report
- `identifier_tag`: Problem identifier prefix (e.g., "SZ25QT9B00WT")
- `wt_add`: Problem number offset
- `ceshi_time`: Test date
- `code_version`: Code version being tested
- `ceshi_user`: Tester name

**ExcelProcessResult** - Result from Excel processing:
- `grouped_data`: Vec of (key, GroupInfo) tuples maintaining insertion order
- Each GroupInfo contains: problem name (B column), severity (D column), record count, and records

**RiskLevel** - Enum with priority ordering:
- High (priority 1) → "高危"
- Medium (priority 2) → "中危"
- Low (priority 3) → "低危"
- Unknown (priority 999)

### Tauri Commands

All commands are async and return `Result<T, String>`:
- `process_excel_file(file_path)` - Process single Excel file, returns ExcelProcessResult
- `generate_report(config)` - Main report generation, processes all files and generates Word doc
- `get_logs()` - Retrieve accumulated logs
- `get_progress()` - Get current progress state
- `clear_logs()` - Clear log history
- `clear_progress()` - Reset progress state

## Key Implementation Details

### Excel Processing Logic
- Deduplication is based on first 7 columns only (columns A-G)
- Grouping uses composite key: `{B_column}_{D_column}` (problem name + severity)
- Results are sorted by: risk level priority (High → Medium → Low) then record count (descending)

### Word Document Generation
- Creates statistics table first with: sequence number, problem name, severity level, count
- Then generates detailed findings for each group
- Uses checkbox symbols (☑/☐) for risk level indicators
- Formats identifier tags as: `{identifier_tag}-WT-{padded_number}`

### Asynchronous Operations
- All Tauri commands are async (using tokio runtime)
- Excel processing is CPU-bound but wrapped in async for consistency
- Progress updates happen synchronously within async context via Mutex-locked state

## Common Development Patterns

### Adding New Tauri Commands
1. Define command function in `src-tauri/src/commands/report_commands.rs` with `#[tauri::command]` attribute
2. Register in `src-tauri/src/lib.rs` using `tauri::generate_handler![]`
3. Call from frontend using `import { invoke } from '@tauri-apps/api/core'`

### Modifying Data Models
- Update structs in `src-tauri/src/models/mod.rs`
- Ensure all structs derive `Serialize` and `Deserialize` for Tauri IPC
- TypeScript types are inferred automatically by Tauri

### Error Handling
- Rust uses `anyhow::Result` internally, converts to `Result<T, String>` for Tauri commands
- Frontend catches errors in try-catch blocks and displays via Element Plus notifications
- All errors are logged to AppState with LogLevel::Error
