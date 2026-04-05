import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { useTerminalStore } from '../store/terminalStore';
import { useAppStore } from '../store/appStore';
import {
  Terminal,
  Cpu,
  Bell,
  BellOff,
  ArrowDownCircle,
  Columns,
  LayoutGrid,
} from 'lucide-react';

const MODEL_COLORS: Record<string, { bg: string; text: string; label: string }> = {
  opus: { bg: 'bg-purple-500/15', text: 'text-purple-400', label: 'Opus' },
  sonnet: { bg: 'bg-blue-500/15', text: 'text-blue-400', label: 'Sonnet' },
  haiku: { bg: 'bg-green-500/15', text: 'text-green-400', label: 'Haiku' },
};

const STATUS_COLORS: Record<string, string> = {
  Running: 'text-success',
  Idle: 'text-warning',
  Stopped: 'text-text-tertiary',
  Error: 'text-error',
};

const STATUS_DOT_COLORS: Record<string, string> = {
  Running: 'bg-success',
  Idle: 'bg-warning',
  Stopped: 'bg-text-tertiary',
  Error: 'bg-error',
};

export function StatusBar() {
  const { terminals, activeTerminalId } = useTerminalStore();
  const {
    toggleSidebar,
    gridMode,
    toggleGridMode,
    notifyOnFinish,
    setNotifyOnFinish,
    openSettings,
  } = useAppStore();

  const [appVersion, setAppVersion] = useState('');
  const [claudeVersion, setClaudeVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
    invoke<string>('get_claude_version')
      .then((v) => setClaudeVersion(v))
      .catch(() => setClaudeVersion(null));
  }, []);

  const terminalCount = terminals.size;
  const runningCount = Array.from(terminals.values()).filter(
    (t) => t.config.status === 'Running'
  ).length;

  const activeTerminal = activeTerminalId ? terminals.get(activeTerminalId) : null;
  const activeStatus = activeTerminal?.config.status || 'Stopped';
  const activeModel = activeTerminal?.model;

  // Resolve model display
  const modelKey = activeModel
    ? Object.keys(MODEL_COLORS).find((k) => activeModel.toLowerCase().includes(k))
    : null;
  const modelInfo = modelKey ? MODEL_COLORS[modelKey] : null;

  return (
    <div className="h-6 flex items-center justify-between px-2 bg-elevation-1 border-t border-border text-[11px] select-none shrink-0">
      {/* Left side */}
      <div className="flex items-center gap-3">
        {/* Terminal count */}
        <button
          onClick={toggleSidebar}
          className="flex items-center gap-1.5 text-text-secondary hover:text-text-primary transition-colors"
          title="Toggle sidebar"
        >
          <Terminal size={12} />
          <span>
            {runningCount > 0
              ? `${runningCount}/${terminalCount} running`
              : `${terminalCount} terminal${terminalCount !== 1 ? 's' : ''}`}
          </span>
        </button>

        {/* Active terminal status */}
        {activeTerminal && (
          <div className="flex items-center gap-1.5">
            <span className="text-text-tertiary">|</span>
            <div
              className={`w-1.5 h-1.5 rounded-full ${STATUS_DOT_COLORS[activeStatus]}`}
              title={activeStatus}
            />
            <span className={`${STATUS_COLORS[activeStatus]} font-medium`}>
              {activeTerminal.config.nickname || activeTerminal.config.label}
            </span>
          </div>
        )}

        {/* Grid/Split indicator */}
        <button
          onClick={toggleGridMode}
          className={`flex items-center gap-1 transition-colors ${
            gridMode
              ? 'text-accent-primary'
              : 'text-text-tertiary hover:text-text-secondary'
          }`}
          title={gridMode ? 'Exit grid mode' : 'Enter grid mode'}
        >
          {gridMode ? <LayoutGrid size={11} /> : <Columns size={11} />}
          <span>{gridMode ? 'Grid' : 'Tabs'}</span>
        </button>
      </div>

      {/* Right side */}
      <div className="flex items-center gap-3">
        {/* Model indicator */}
        {modelInfo && (
          <div
            className={`flex items-center gap-1 px-1.5 py-0.5 rounded ${modelInfo.bg}`}
          >
            <Cpu size={10} className={modelInfo.text} />
            <span className={`${modelInfo.text} font-medium`}>
              {modelInfo.label}
            </span>
          </div>
        )}

        {/* Notifications toggle */}
        <button
          onClick={() => setNotifyOnFinish(!notifyOnFinish)}
          className={`flex items-center transition-colors ${
            notifyOnFinish
              ? 'text-text-secondary hover:text-text-primary'
              : 'text-text-tertiary hover:text-text-secondary'
          }`}
          title={notifyOnFinish ? 'Notifications on' : 'Notifications off'}
        >
          {notifyOnFinish ? <Bell size={12} /> : <BellOff size={12} />}
        </button>

        {/* Claude version */}
        {claudeVersion && (
          <button
            onClick={openSettings}
            className="flex items-center gap-1 text-text-tertiary hover:text-text-secondary transition-colors"
            title="Open settings"
          >
            <ArrowDownCircle size={10} />
            <span>Claude {claudeVersion}</span>
          </button>
        )}

        {/* App version */}
        <span className="text-text-tertiary" title={`ClaudeTerminal v${appVersion}`}>
          v{appVersion}
        </span>
      </div>
    </div>
  );
}
