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
  identifier_tag: "WT",
  wt_add: 0,
  ceshi_time: new Date().toISOString().split("T")[0],
  code_version: "V1.0",
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
          offset: 60,
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
          offset: 60,
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
        offset: 60,
      });
    }
  } catch (error) {
    console.error("选择文件失败:", error);
    ElMessage({
      message: `选择文件失败: ${error}`,
      type: "error",
      offset: 60,
    });
  }
};

const removeFile = (index: number) => {
  config.excel_files.splice(index, 1);
  ElMessage({
    message: "已移除文件",
    type: "success",
    offset: 60,
  });
};

// 选择输出目录
const selectOutputDir = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: "选择输出目录",
    });

    if (selected) {
      config.output_dir = selected as string;
      ElMessage({
        message: "已选择输出目录",
        type: "success",
        offset: 60,
      });
    }
  } catch (error) {
    console.error("选择目录失败:", error);
    ElMessage({
      message: `选择目录失败: ${error}`,
      type: "error",
      placement: "top-right",
      offset: 60,
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
      offset: 60,
    });
    return;
  }

  if (!config.output_dir) {
    ElMessage({
      message: "请选择输出目录",
      type: "warning",
      placement: "top-right",
      offset: 60,
    });
    return;
  }

  if (!config.identifier_tag || !config.ceshi_user) {
    ElMessage({
      message: "请填写所有必填字段",
      type: "warning",
      placement: "top-right",
      offset: 60,
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
      offset: 60,
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
      offset: 60,
    });
  } finally {
    isGenerating.value = false;
  }
};
</script>

<template>
  <div class="app-container">
    <div class="content-wrapper">
      <div class="header">
        <h1>代码审计报告问题清单生成器</h1>
      </div>

      <div class="main-card">
        <!-- 配置表单 -->
        <div class="form-section">
          <!-- Excel文件 -->
          <div class="form-item">
            <label class="form-label">Excel文件</label>
            <button
              class="select-button"
              :class="{ 'dragging': isDragging }"
              @click="selectExcelFiles"
            >
              <el-icon><Folder /></el-icon>
              <span>{{ config.excel_files.length > 0 ? `已选择 ${config.excel_files.length} 个文件` : '选择Excel文件或拖拽到此处' }}</span>
            </button>
            <div v-if="config.excel_files.length > 0" class="file-tags">
              <span
                v-for="(file, index) in config.excel_files"
                :key="index"
                class="file-tag"
              >
                {{ file.split('\\').pop() || file.split('/').pop() }}
                <button class="file-tag-close" @click="removeFile(index)">×</button>
              </span>
            </div>
          </div>

          <!-- 输出目录 -->
          <div class="form-item">
            <label class="form-label">输出目录</label>
            <button class="select-button" @click="selectOutputDir">
              <el-icon><Folder /></el-icon>
              <span>{{ config.output_dir || '选择输出目录' }}</span>
            </button>
          </div>

          <!-- 标识号前缀 -->
          <div class="form-item">
            <label class="form-label">标识号前缀</label>
            <el-input
              v-model="config.identifier_tag"
              placeholder="如: SZ25QT9B00WT"
              class="custom-input"
            />
          </div>
          <div class="form-item">
            <label class="form-label">问题编号起始</label>
            <el-input-number
              v-model="config.wt_add"
              :min="0"
              :max="9999"
              class="custom-input-number"
            />
          </div>
          <!-- 测试人员 -->
          <div class="form-item">
            <label class="form-label">测试人员</label>
            <el-input
              v-model="config.ceshi_user"
              placeholder="请输入测试人员姓名"
              class="custom-input"
            />
          </div>
          <div class="form-item">
            <label class="form-label">测试时间</label>
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
            <label class="form-label">代码版本</label>
            <el-input
              v-model="config.code_version"
              placeholder="如: V1.0"
              class="custom-input"
            />
          </div>

          <!-- 生成按钮 -->
          <button
            class="generate-button"
            :class="{ 'generating': isGenerating }"
            :disabled="isGenerating"
            @click="generateReport"
          >
            {{ isGenerating ? "生成中..." : "生成报告" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-container {
  min-height: 100vh;
  background: linear-gradient(to bottom, #f5f7fa 0%, #e8eef5 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Helvetica Neue", Arial, sans-serif;
}

.content-wrapper {
  width: 100%;
  max-width: 560px;
}

.header {
  text-align: center;
  margin-bottom: 32px;
}

.header h1 {
  font-size: 32px;
  font-weight: 600;
  color: #1d1d1f;
  margin: 0;
  letter-spacing: -0.5px;
}

.main-card {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.06);
  padding: 40px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  font-size: 14px;
  font-weight: 500;
  color: #1d1d1f;
  display: block;
}

.select-button {
  width: 100%;
  padding: 12px 16px;
  background: white;
  border: 1.5px solid #d2d2d7;
  border-radius: 10px;
  font-size: 15px;
  color: #1d1d1f;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  text-align: left;
}

.select-button:hover {
  border-color: #007aff;
  background: #f5f7fa;
}

.select-button:active {
  transform: scale(0.98);
}

.select-button.dragging {
  border-color: #007aff;
  background: #e8f4ff;
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.1);
}

.file-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
}

.file-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #f5f7fa;
  border-radius: 8px;
  font-size: 13px;
  color: #1d1d1f;
}

.file-tag-close {
  background: none;
  border: none;
  font-size: 18px;
  color: #86868b;
  cursor: pointer;
  padding: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
}

.file-tag-close:hover {
  background: #d2d2d7;
  color: #1d1d1f;
}

.generate-button {
  width: 100%;
  padding: 14px;
  background: #007aff;
  color: white;
  border: none;
  border-radius: 10px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-top: 12px;
}

.generate-button:hover:not(:disabled) {
  background: #0051d5;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
}

.generate-button:active:not(:disabled) {
  transform: translateY(0);
}

.generate-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.generate-button.generating {
  background: #86868b;
}

/* Element Plus 自定义样式 */
.custom-input {
  border-radius: 10px;
}

.custom-input :deep(.el-input__wrapper) {
  border-radius: 10px;
  box-shadow: 0 0 0 1.5px #d2d2d7 inset;
  transition: all 0.2s ease;
}

.custom-input :deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1.5px #007aff inset;
}

.custom-input :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 2px #007aff inset;
}

.optional-section {
  border: none;
  background: #f5f7fa;
  border-radius: 10px;
  padding: 4px;
}

.optional-section :deep(.el-collapse-item__header) {
  background: transparent;
  border: none;
  font-size: 14px;
  font-weight: 500;
  color: #1d1d1f;
  padding: 12px 16px;
}

.optional-section :deep(.el-collapse-item__wrap) {
  border: none;
  background: transparent;
}

.optional-section :deep(.el-collapse-item__content) {
  padding: 8px 16px 16px;
}

.custom-input-number {
  width: 100%;
}

.custom-input-number :deep(.el-input__wrapper) {
  border-radius: 10px;
  box-shadow: 0 0 0 1.5px #d2d2d7 inset;
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
