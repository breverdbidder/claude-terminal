import { useState, useEffect } from 'react';
import { RefreshCw, GitBranch, GitFork, FilePlus, FileEdit, FileX, FileQuestion, ArrowRightLeft, FolderOpen, ChevronRight, ChevronDown } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useTerminalStore } from '../store/terminalStore';
import { useAppStore } from '../store/appStore';
import { InlineDiffView } from './InlineDiffView';

interface FileChange {
  path: string;
  status: string;
}

interface FileChangesResult {
  terminal_id: string;
  working_directory: string;
  changes: FileChange[];
  is_git_repo: boolean;
  branch: string | null;
  error: string | null;
}

const statusConfig: Record<string, { label: string; color: string; icon: React.ReactNode }> = {
  new: { label: 'New', color: 'text-green-400', icon: <FilePlus size={14} /> },
  modified: { label: 'Modified', color: 'text-yellow-400', icon: <FileEdit size={14} /> },
  deleted: { label: 'Deleted', color: 'text-red-400', icon: <FileX size={14} /> },
  renamed: { label: 'Renamed', color: 'text-blue-400', icon: <ArrowRightLeft size={14} /> },
  untracked: { label: 'Untracked', color: 'text-text-tertiary', icon: <FileQuestion size={14} /> },
};

export function FileChangesPanel() {
  const activeTerminalId = useTerminalStore((s) => s.activeTerminalId);
  const gitInfoCache = useTerminalStore((s) => s.gitInfoCache);
  const changesRefreshTrigger = useAppStore((s) => s.changesRefreshTrigger);
  const openWorktreeModal = useAppStore((s) => s.openWorktreeModal);
  const activeGitInfo = activeTerminalId ? gitInfoCache.get(activeTerminalId) : null;
  const [result, setResult] = useState<FileChangesResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedFile, setExpandedFile] = useState<string | null>(null);

  const fetchChanges = async () => {
    if (!activeTerminalId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<FileChangesResult>('get_terminal_changes', { id: activeTerminalId });
      setResult(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchChanges();
    setExpandedFile(null);
  }, [activeTerminalId, changesRefreshTrigger]);

  // Group changes by status
  const grouped = result?.changes.reduce<Record<string, FileChange[]>>((acc, change) => {
    const key = change.status;
    if (!acc[key]) acc[key] = [];
    acc[key].push(change);
    return acc;
  }, {}) ?? {};

  const statusOrder = ['new', 'modified', 'deleted', 'renamed', 'untracked'];

  return (
    <div className="h-full bg-bg-secondary border-l border-border flex flex-col">
      {/* Header */}
      <div className="p-3 border-b border-border">
        <div className="flex items-center justify-between mb-1">
          <h3 className="text-text-primary text-[13px] font-semibold">File Changes</h3>
          <button
            onClick={fetchChanges}
            disabled={loading || !activeTerminalId}
            className="p-1 rounded hover:bg-white/[0.04] text-text-secondary transition-colors disabled:opacity-40"
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>
        {result?.branch && (
          <div className="flex items-center gap-1.5 text-text-secondary">
            {activeGitInfo?.is_worktree ? (
              <GitFork size={12} className="text-purple-400" />
            ) : (
              <GitBranch size={12} />
            )}
            <span className="text-[11px] font-mono">{result.branch}</span>
          </div>
        )}
        {activeGitInfo?.is_worktree && activeGitInfo.main_repo_path && (
          <div className="flex items-center justify-between mt-1">
            <span className="text-text-tertiary text-[11px]">
              Worktree of {activeGitInfo.main_repo_path.replace(/^.*[\\/]/, '')}
            </span>
            <button
              onClick={() => openWorktreeModal(activeGitInfo.main_repo_path!)}
              className="text-accent-primary text-[11px] hover:text-accent-secondary transition-colors"
            >
              Manage
            </button>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-1.5">
        {!activeTerminalId && (
          <div className="flex items-center justify-center h-full">
            <p className="text-text-tertiary text-[12px]">No terminal selected</p>
          </div>
        )}

        {activeTerminalId && error && (
          <div className="p-3">
            <p className="text-red-400 text-[12px]">{error}</p>
          </div>
        )}

        {activeTerminalId && result && !result.is_git_repo && (
          <div className="flex items-center justify-center h-full">
            <p className="text-text-tertiary text-[12px]">Not a git repository</p>
          </div>
        )}

        {activeTerminalId && result && result.is_git_repo && result.changes.length === 0 && !result.error && (
          <div className="flex items-center justify-center h-full">
            <p className="text-text-tertiary text-[12px]">No uncommitted changes</p>
          </div>
        )}

        {activeTerminalId && result?.error && (
          <div className="p-3">
            <p className="text-red-400 text-[12px]">{result.error}</p>
          </div>
        )}

        {result && result.changes.length > 0 && statusOrder.map((status) => {
          const files = grouped[status];
          if (!files || files.length === 0) return null;
          const config = statusConfig[status] || statusConfig.untracked;
          return (
            <div key={status} className="mb-2">
              <div className="flex items-center gap-1.5 px-2 py-1">
                <span className={config.color}>{config.icon}</span>
                <span className="text-text-secondary text-[11px] font-medium uppercase tracking-wider">
                  {config.label}
                </span>
                <span className="text-text-tertiary text-[11px]">({files.length})</span>
              </div>
              {files.map((file) => {
                const isExpanded = expandedFile === file.path;
                return (
                  <div key={file.path}>
                    <div
                      onClick={() => setExpandedFile(isExpanded ? null : file.path)}
                      className="ml-3 px-2 py-1 rounded hover:bg-white/[0.04] transition-colors cursor-pointer flex items-center gap-1"
                    >
                      {isExpanded ? (
                        <ChevronDown size={12} className="text-text-tertiary shrink-0" />
                      ) : (
                        <ChevronRight size={12} className="text-text-tertiary shrink-0" />
                      )}
                      <p className={`text-[12px] font-mono truncate ${config.color}`} title={file.path}>
                        {file.path}
                      </p>
                    </div>
                    {isExpanded && activeTerminalId && (
                      <div className="ml-3 mr-1 mb-1 rounded overflow-hidden border border-border/30">
                        <InlineDiffView
                          filePath={file.path}
                          terminalId={activeTerminalId}
                        />
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>

      {/* Footer */}
      <div className="p-2 border-t border-border">
        <div className="bg-bg-primary ring-1 ring-border rounded-md p-2.5">
          {result?.working_directory && (
            <div className="flex items-center gap-1.5 mb-1">
              <FolderOpen size={11} className="text-text-tertiary shrink-0" />
              <p className="text-text-tertiary text-[11px] font-mono truncate" title={result.working_directory}>
                {result.working_directory}
              </p>
            </div>
          )}
          <p className="text-text-secondary text-[11px]">
            {result ? `${result.changes.length} changed file${result.changes.length !== 1 ? 's' : ''}` : 'Press F2 to toggle'}
          </p>
        </div>
      </div>
    </div>
  );
}
