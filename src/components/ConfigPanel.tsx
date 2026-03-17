import React from 'react';

export interface AppConfig {
  default_directory: string | null;
  excluded_dirs: string[];
  top_k: number;
  threshold: number;
  file_types: string[];
}

interface ConfigPanelProps {
  config: AppConfig;
  loading: boolean;
  saving: boolean;
  indexing: boolean;
  onConfigChange: (config: AppConfig) => void;
  onSave: () => Promise<void>;
  onSelectDirectory: () => Promise<void>;
  onStartIndexing: () => Promise<void>;
}

export function ConfigPanel({
  config,
  loading,
  saving,
  indexing,
  onConfigChange,
  onSave,
  onSelectDirectory,
  onStartIndexing,
}: ConfigPanelProps) {
  if (loading) {
    return <div className="config-panel">加载配置...</div>;
  }

  return (
    <div className="config-panel">
      <h2>⚙️ 配置</h2>

      <div className="config-section">
        <h3>搜索设置</h3>

        <div className="config-item">
          <label>默认搜索目录</label>
          <div className="config-input-group">
            <input
              type="text"
              value={config.default_directory || ''}
              onChange={(e) => onConfigChange({ ...config, default_directory: e.target.value || null })}
              placeholder="请选择一个要建立索引的目录"
            />
            <button className="action-button" onClick={onSelectDirectory}>
              选择目录
            </button>
          </div>
        </div>

        <div className="config-item">
          <label>返回结果数量 (Top K)</label>
          <input
            type="number"
            min="1"
            max="100"
            value={config.top_k}
            onChange={(e) => onConfigChange({ ...config, top_k: parseInt(e.target.value, 10) || 10 })}
          />
        </div>

        <div className="config-item">
          <label>匹配阈值 ({config.threshold.toFixed(2)})</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={config.threshold}
            onChange={(e) => onConfigChange({ ...config, threshold: parseFloat(e.target.value) })}
          />
          <small>数值越高，结果越严格</small>
        </div>
      </div>

      <div className="config-section">
        <h3>排除目录</h3>
        <div className="config-item">
          <textarea
            value={config.excluded_dirs.join('\n')}
            onChange={(e) => onConfigChange({
              ...config,
              excluded_dirs: e.target.value.split('\n').map((d) => d.trim()).filter(Boolean),
            })}
            placeholder="每行一个目录名"
            rows={5}
          />
          <small>这些目录不会参与索引</small>
        </div>
      </div>

      <div className="config-section">
        <h3>文件类型过滤</h3>
        <div className="config-item">
          <textarea
            value={config.file_types.join('\n')}
            onChange={(e) => onConfigChange({
              ...config,
              file_types: e.target.value.split('\n').map((t) => t.trim()).filter(Boolean),
            })}
            placeholder=".md&#10;.py&#10;.ts"
            rows={4}
          />
          <small>留空表示索引并搜索所有文件类型</small>
        </div>
      </div>

      <div className="config-actions">
        <button
          className="search-button"
          onClick={onSave}
          disabled={saving}
        >
          {saving ? '保存中...' : '保存配置'}
        </button>
        <button
          className="action-button primary-action"
          onClick={onStartIndexing}
          disabled={indexing || !config.default_directory}
        >
          {indexing ? '索引中...' : '开始索引'}
        </button>
      </div>
    </div>
  );
}
