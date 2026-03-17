import React from 'react';

interface WelcomePanelProps {
  hasDirectory: boolean;
  indexedFiles: number;
  isIndexing: boolean;
  onOpenSettings: () => void;
  onStartIndexing: () => Promise<void>;
}

export function WelcomePanel({
  hasDirectory,
  indexedFiles,
  isIndexing,
  onOpenSettings,
  onStartIndexing,
}: WelcomePanelProps) {
  const needsSetup = !hasDirectory;
  const needsIndex = hasDirectory && indexedFiles === 0 && !isIndexing;

  return (
    <section className="welcome-panel">
      <div className="welcome-copy">
        <p className="eyebrow">首次使用指南</p>
        <h2>先选目录，再建立索引，最后用自然语言搜索</h2>
        <p>
          这个应用不会直接扫全盘。先指定一个目录建立本地语义索引，之后搜索才会有结果。
        </p>
      </div>

      <div className="welcome-steps">
        <div className={`welcome-step ${hasDirectory ? 'done' : ''}`}>
          <strong>1. 选择目录</strong>
          <span>{hasDirectory ? '已设置默认目录' : '还没有默认目录'}</span>
        </div>
        <div className={`welcome-step ${indexedFiles > 0 ? 'done' : ''}`}>
          <strong>2. 建立索引</strong>
          <span>
            {isIndexing ? '正在建立索引' : indexedFiles > 0 ? `已索引 ${indexedFiles} 个文件` : '尚未建立索引'}
          </span>
        </div>
        <div className={`welcome-step ${indexedFiles > 0 ? 'done' : ''}`}>
          <strong>3. 输入搜索描述</strong>
          <span>例如：用户登录逻辑、报表导出代码、合同模板</span>
        </div>
      </div>

      <div className="welcome-actions">
        <button className="action-button" onClick={onOpenSettings}>
          打开设置
        </button>
        {(needsSetup || needsIndex) && (
          <button
            className="search-button"
            onClick={() => void onStartIndexing()}
            disabled={needsSetup || isIndexing}
          >
            {isIndexing ? '索引中...' : '立即建立索引'}
          </button>
        )}
      </div>
    </section>
  );
}
