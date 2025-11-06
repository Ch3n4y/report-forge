<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  ElMessage,
  ElNotification,
} from "element-plus";
import {
  Folder,
} from "@element-plus/icons-vue";

// 类型定义
interface ReportConfig {
  excel_files: string[];
  template_file: string;
  output_dir: string;
  identifier_tag: string;
  wt_add: number;
  ceshi_time: string;
  code_version: string;
  ceshi_user: string;
}

// 响应式状态
const config = reactive<ReportConfig>({
  excel_files: [],
  template_file: "",
  output_dir: "",
  identifier_tag: "",
  wt_add: 0,
  ceshi_time: '',
  code_version: "",
  ceshi_user: "",
});

const isGenerating = ref(false);
const isDragging = ref(false);
let unlistenFileDrop: UnlistenFn | null = null;

// 设置文件拖拽监听
onMounted(async () => {
  const webview = getCurrentWebviewWindow();

  unlistenFileDrop = await webview.onDragDropEvent((event) => {
    if (event.payload.type === "over") {
      isDragging.value = true;
    } else if (event.payload.type === "leave") {
      isDragging.value = false;
    } else if (event.payload.type === "drop") {
      isDragging.value = false;
      const paths = event.payload.paths || [];

      // 过滤Excel文件
      const excelFiles = paths.filter((path: string) => {
        const ext = path.split(".").pop()?.toLowerCase();
        return ext === "xlsx" || ext === "xls";
      });

      if (excelFiles.length === 0) {
        ElMessage({
          message: "请拖拽Excel文件（.xlsx 或 .xls）",
          type: "warning",
          placement: "top-right",
          
        });
        return;
      }

      // 添加到已选文件列表，避免重复
      const newFiles = excelFiles.filter((path: string) => !config.excel_files.includes(path));
      config.excel_files.push(...newFiles);

      if (newFiles.length > 0) {
        ElMessage({
          message: `已添加 ${newFiles.length} 个文件`,
          type: "success",
          placement: "top-right",
          
        });
      }
    }
  });
});

onUnmounted(() => {
  if (unlistenFileDrop) {
    unlistenFileDrop();
  }
});

// 选择Excel文件
const selectExcelFiles = async () => {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [
        {
          name: "Excel",
          extensions: ["xlsx", "xls"],
        },
      ],
      title: "选择Excel文件",
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      config.excel_files = paths;
      ElMessage({
        message: `已选择 ${paths.length} 个文件`,
        type: "success",
        placement: "top-right",
        
      });
    }
  } catch (error) {
    console.error("选择文件失败:", error);
    ElMessage({
      message: `选择文件失败: ${error}`,
      type: "error",
      placement: "top-right",
      
    });
  }
};

const removeFile = (index: number) => {
  config.excel_files.splice(index, 1);
  ElMessage({
    message: "已移除文件",
    type: "success",
    placement: "top-right",
    
  });
};

// 选择输出目录
const selectOutputDir = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      placement: "top-right",
      title: "选择输出目录",
    });

    if (selected) {
      config.output_dir = selected as string;
      ElMessage({
        message: "已选择输出目录",
        type: "success",
        placement: "top-right",
        
      });
    }
  } catch (error) {
    console.error("选择目录失败:", error);
    ElMessage({
      message: `选择目录失败: ${error}`,
      type: "error",
      placement: "top-right",
      
    });
  }
};

// 生成报告
const generateReport = async () => {
  if (config.excel_files.length === 0) {
    ElMessage({
      message: "请至少选择一个Excel文件",
      type: "warning",
      placement: "top-right",
      
    });
    return;
  }

  if (!config.output_dir) {
    ElMessage({
      message: "请选择输出目录",
      type: "warning",
      placement: "top-right",
      
    });
    return;
  }

  if (!config.identifier_tag || !config.ceshi_user) {
    ElMessage({
      message: "请填写所有必填字段",
      type: "warning",
      placement: "top-right",
      
    });
    return;
  }

  isGenerating.value = true;

  try {
    const outputFile = await invoke<string>("generate_report", { config });

    ElNotification({
      title: "成功",
      message: `报告生成成功！\n文件: ${outputFile}`,
      type: "success",
      duration: 5000,
      position: "top-right",
      
    });

    // 尝试打开文件
    setTimeout(async () => {
      try {
        await invoke("plugin:opener|open", { path: outputFile });
      } catch (err) {
        console.error("打开文件失败:", err);
      }
    }, 500);
  } catch (error) {
    console.error("生成报告失败:", error);
    ElNotification({
      title: "错误",
      message: `生成报告失败: ${error}`,
      type: "error",
      duration: 5000,
      position: "top-right",
      
    });
  } finally {
    isGenerating.value = false;
  }
};
</script>

<template>
  <div class="app-container">
    <div class="background-decoration">
      <div class="decoration-circle decoration-1"></div>
      <div class="decoration-circle decoration-2"></div>
      <div class="decoration-circle decoration-3"></div>
    </div>

    <div class="content-wrapper">
      <div class="main-card">
        <!-- 文件上传区域 -->
        <div class="upload-section">
          <div class="section-header">
            <h2>数据源</h2>
            <span class="section-badge">必填</span>
          </div>

          <button
            class="upload-area"
            :class="{ 'dragging': isDragging, 'has-files': config.excel_files.length > 0 }"
            @click="selectExcelFiles"
          >
            <div class="upload-icon">
              <svg v-if="config.excel_files.length === 0" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M7 18a4.6 4.4 0 010-9 5 4.5 0 0111 2h1a3.5 3.5 0 010 7h-1" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M9 15l3-3 3 3m-3-3v10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              <el-icon v-else><Folder /></el-icon>
            </div>
            <div class="upload-text">
              <span class="upload-title">
                {{ config.excel_files.length > 0 ? `已选择 ${config.excel_files.length} 个文件` : '点击选择或拖拽Excel文件' }}
              </span>
              <span class="upload-hint">支持 .xlsx 和 .xls 格式，可多选</span>
            </div>
          </button>

          <transition name="fade">
            <div v-if="config.excel_files.length > 0" class="file-list">
              <div
                v-for="(file, index) in config.excel_files"
                :key="index"
                class="file-item"
              >
                <div class="file-icon">
                  <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </div>
                <span class="file-name">{{ file.split('\\').pop() || file.split('/').pop() }}</span>
                <button class="file-remove" @click.stop="removeFile(index)">
                  <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M6 18L18 6M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
              </div>
            </div>
          </transition>

          <div class="form-item">
            <label class="form-label">
              <span class="label-icon">📁</span>
              输出目录
            </label>
            <button class="select-button" @click="selectOutputDir">
              <el-icon><Folder /></el-icon>
              <span>{{ config.output_dir || '选择报告保存位置' }}</span>
            </button>
          </div>
        </div>

        <div class="divider"></div>

        <!-- 配置区域 -->
        <div class="config-section">
          <div class="section-header">
            <h2>报告配置</h2>
            <span class="section-badge">必填</span>
          </div>

          <div class="form-grid">
            <div class="form-item">
              <label class="form-label">
                <span class="label-icon">🏷️</span>
                标识号前缀
              </label>
              <el-input
                v-model="config.identifier_tag"
                placeholder="例如: SZ25QT9B00WT"
                class="custom-input"
              />
            </div>

            <div class="form-item">
              <label class="form-label">
                <span class="label-icon">🔢</span>
                问题编号起始
              </label>
              <el-input-number
                v-model="config.wt_add"
                :min="0"
                :max="9999"
                class="custom-input-number"
                controls-position="right"
              />
            </div>

            <div class="form-item">
              <label class="form-label">
                <span class="label-icon">👤</span>
                测试人员
              </label>
              <el-input
                v-model="config.ceshi_user"
                placeholder="请输入测试人员姓名"
                class="custom-input"
              />
            </div>

            <div class="form-item">
              <label class="form-label">
                <span class="label-icon">📅</span>
                测试时间
              </label>
              <el-date-picker
                v-model="config.ceshi_time"
                type="date"
                placeholder="选择测试时间"
                value-format="YYYY-MM-DD"
                class="custom-input"
                style="width: 100%;"
              />
            </div>

            <div class="form-item">
              <label class="form-label">
                <span class="label-icon">📦</span>
                代码版本
              </label>
              <el-input
                v-model="config.code_version"
                placeholder="例如: V1.0"
                class="custom-input"
              />
            </div>
          </div>
        </div>

        <!-- 生成按钮 -->
        <button
          class="generate-button"
          :class="{ 'generating': isGenerating }"
          :disabled="isGenerating"
          @click="generateReport"
        >
          <span v-if="!isGenerating" class="button-content">
            <svg class="button-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M5 12h14m-7-7v14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            生成报告
          </span>
          <span v-else class="button-content">
            <span class="loading-spinner"></span>
            生成中...
          </span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 主容器 */
.app-container {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #e8edf2 50%, #dfe6ed 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Helvetica Neue", Arial, sans-serif;
  position: relative;
  overflow: hidden;
}

/* 背景装饰 */
.background-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  pointer-events: none;
}

.decoration-circle {
  position: absolute;
  border-radius: 50%;
  background: rgba(0, 122, 255, 0.08);
  backdrop-filter: blur(40px);
}

.decoration-1 {
  width: 400px;
  height: 400px;
  top: -100px;
  right: -100px;
  animation: float 20s ease-in-out infinite;
}

.decoration-2 {
  width: 300px;
  height: 300px;
  bottom: -50px;
  left: -50px;
  animation: float 15s ease-in-out infinite reverse;
}

.decoration-3 {
  width: 200px;
  height: 200px;
  top: 50%;
  right: 10%;
  animation: float 25s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0) scale(1);
  }
  50% {
    transform: translateY(-30px) scale(1.05);
  }
}

/* 内容包裹器 */
.content-wrapper {
  width: 100%;
  max-width: 650px;
  position: relative;
  z-index: 1;
}

/* 头部 */
.header {
  text-align: center;
  margin-bottom: 40px;
}

.header-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 20px;
  background: linear-gradient(135deg, #007AFF 0%, #0051D5 100%);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 32px rgba(0, 122, 255, 0.25);
  animation: pulse 2s ease-in-out infinite;
}

.header-icon svg {
  width: 32px;
  height: 32px;
  color: white;
  stroke-width: 2;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    box-shadow: 0 8px 32px rgba(0, 122, 255, 0.25);
  }
  50% {
    transform: scale(1.05);
    box-shadow: 0 12px 40px rgba(0, 122, 255, 0.35);
  }
}

.header h1 {
  font-size: 36px;
  font-weight: 700;
  color: #1d1d1f;
  margin: 0 0 12px 0;
  letter-spacing: -0.5px;
  text-shadow: none;
}

.header-subtitle {
  font-size: 16px;
  color: #86868b;
  margin: 0;
  font-weight: 400;
}

/* 主卡片 */
.main-card {
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(30px);
  border-radius: 24px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
  padding: 48px;
  border: 1px solid rgba(255, 255, 255, 0.5);
}

/* 区域标题 */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #1d1d1f;
  margin: 0;
}

.section-badge {
  display: inline-block;
  padding: 4px 12px;
  background: linear-gradient(135deg, #007AFF 0%, #0051D5 100%);
  color: white;
  font-size: 12px;
  font-weight: 500;
  border-radius: 12px;
  letter-spacing: 0.5px;
}

/* 上传区域 */
.upload-section {
  margin-bottom: 32px;
}

.upload-area {
  width: 100%;
  padding: 32px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e8eef5 100%);
  border: 2px dashed #c1c9d2;
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
}

.upload-area:hover {
  border-color: #007AFF;
  background: linear-gradient(135deg, #e8f4ff 0%, #f0f8ff 100%);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 122, 255, 0.12);
}

.upload-area.dragging {
  border-color: #007AFF;
  background: linear-gradient(135deg, #e8f4ff 0%, #f0f8ff 100%);
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.08);
  transform: scale(1.02);
}

.upload-area.has-files {
  background: linear-gradient(135deg, #e8f5e9 0%, #f0f4ff 100%);
  border-color: #81c784;
}

.upload-icon {
  width: 56px;
  height: 56px;
  background: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.upload-icon svg {
  width: 28px;
  height: 28px;
  color: #007AFF;
  stroke-width: 2;
}

.upload-icon .el-icon {
  font-size: 28px;
  color: #81c784;
}

.upload-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.upload-title {
  font-size: 16px;
  font-weight: 500;
  color: #1d1d1f;
}

.upload-hint {
  font-size: 13px;
  color: #86868b;
}

/* 文件列表 */
.file-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  transition: all 0.2s ease;
}

.file-item:hover {
  border-color: #007AFF;
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.08);
}

.file-icon {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #007AFF 0%, #0051D5 100%);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.file-icon svg {
  width: 20px;
  height: 20px;
  color: white;
  stroke-width: 2;
}

.file-name {
  flex: 1;
  font-size: 14px;
  color: #1d1d1f;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-remove {
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.file-remove svg {
  width: 14px;
  height: 14px;
  color: #86868b;
  stroke-width: 2;
}

.file-remove:hover {
  background: #fee;
}

.file-remove:hover svg {
  color: #ef4444;
}

/* 分隔线 */
.divider {
  height: 1px;
  background: linear-gradient(to right, transparent, #e5e7eb, transparent);
  margin: 32px 0;
}

/* 配置区域 */
.config-section {
  margin-bottom: 32px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.form-label {
  font-size: 14px;
  font-weight: 500;
  color: #1d1d1f;
  display: flex;
  align-items: center;
  gap: 8px;
}

.label-icon {
  font-size: 16px;
}

.select-button {
  width: 100%;
  padding: 12px 16px;
  background: white;
  border: 1.5px solid #d2d2d7;
  border-radius: 12px;
  font-size: 15px;
  color: #1d1d1f;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  text-align: left;
  font-weight: 400;
}

.select-button:hover {
  border-color: #007AFF;
  background: #f5f7fa;
  transform: translateY(-1px);
}

.select-button:active {
  transform: scale(0.98);
}

/* 生成按钮 */
.generate-button {
  width: 100%;
  padding: 16px 32px;
  background: linear-gradient(135deg, #007AFF 0%, #0051D5 100%);
  color: white;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 8px 24px rgba(0, 122, 255, 0.25);
  position: relative;
  overflow: hidden;
}

.generate-button::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
  transition: left 0.5s;
}

.generate-button:hover:not(:disabled)::before {
  left: 100%;
}

.generate-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 12px 32px rgba(0, 122, 255, 0.35);
}

.generate-button:active:not(:disabled) {
  transform: translateY(0);
}

.generate-button:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.generate-button.generating {
  background: linear-gradient(135deg, #94a3b8 0%, #64748b 100%);
}

.button-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.button-icon {
  width: 20px;
  height: 20px;
  stroke-width: 2.5;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Element Plus 自定义样式 */
.custom-input {
  border-radius: 12px;
}

.custom-input :deep(.el-input__wrapper) {
  border-radius: 12px;
  box-shadow: 0 0 0 1.5px #d2d2d7 inset;
  transition: all 0.2s ease;
}

.custom-input :deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1.5px #007AFF inset;
}

.custom-input :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 2px #007AFF inset;
}

.custom-input-number {
  width: 100%;
}

.custom-input-number :deep(.el-input__wrapper) {
  border-radius: 12px;
  box-shadow: 0 0 0 1.5px #d2d2d7 inset;
  transition: all 0.2s ease;
}

.custom-input-number :deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1.5px #007AFF inset;
}

.custom-input-number :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 2px #007AFF inset;
}

/* 过渡动画 */
.fade-enter-active, .fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

/* 响应式 */
@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr;
  }

  .main-card {
    padding: 32px 24px;
  }

  .header h1 {
    font-size: 28px;
  }
}
</style>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Helvetica Neue", Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
