import { useState, useEffect, useMemo } from 'react';
import appIcon from '../assets/app-icon.png';
import {
  Lightbulb,
  FileDiff,
  Users,
  Settings,
  Minus,
  Square,
  X,
  GitBranch,
  ChevronDown,
} from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getVersion } from '@tauri-apps/api/app';
import { useAppStore } from '../store/appStore';
import { useTerminalStore } from '../store/terminalStore';

const isMac = navigator.platform.toUpperCase().includes('MAC');

function pickBreadcrumb(path: string | undefined): { project: string; sub: string | null } {
  if (!path) return { project: 'No terminal', sub: null };
  // Normalise slashes and trim trailing separators
  const clean = path.replace(/\\/g, '/').replace(/\/+$/, '');
  const parts = clean.split('/').filter(Boolean);
  if (parts.length === 0) return { project: clean || '/', sub: null };
  if (parts.length === 1) return { project: parts[0], sub: null };
  const project = parts[parts.length - 1];
  const parent = parts[parts.length - 2];
  return { project, sub: parent };
}

export function TitleBar() {
  const {
    toggleSidebar,
    toggleHints,
    toggleChanges,
    toggleOrchestration,
    openSettings,
    openCommandPalette,
    sidebarOpen,
    hintsOpen,
    changesOpen,
    orchestrationOpen,
  } = useAppStore();
  const { terminals, activeTerminalId, gitInfoCache } = useTerminalStore();
  const appWindow = getCurrentWindow();
  const [appVersion, setAppVersion] = useState('');

  useEffect(() => {
    getVersion().then(setAppVersion);
  }, []);

  const active = activeTerminalId ? terminals.get(activeTerminalId) : null;
  const gitInfo = activeTerminalId ? gitInfoCache.get(activeTerminalId) : null;
  const breadcrumb = useMemo(
    () => pickBreadcrumb(active?.config.working_directory),
    [active?.config.working_directory]
  );
  const statusDot = !active
    ? 'bg-text-tertiary'
    : active.config.status === 'Running'
    ? 'bg-success'
    : active.config.status === 'Idle'
    ? 'bg-warning'
    : active.config.status === 'Error'
    ? 'bg-error'
    : 'bg-text-tertiary';

  const iconBtn = (active: boolean) =>
    `no-drag w-7 h-7 flex items-center justify-center rounded-[6px] transition-colors ${
      active
        ? 'bg-accent-primary/18 text-accent-primary ring-1 ring-inset ring-accent-primary/35'
        : 'text-text-secondary hover:bg-white/[0.06] hover:text-text-primary'
    }`;

  return (
    <div
      onMouseDown={(e) => { if (e.buttons === 1 && (e.target as HTMLElement).closest('.no-drag') === null) appWindow.startDragging(); }}
      className="h-9 bg-elevation-1 flex items-center justify-between pl-2 pr-0 border-b border-[var(--ij-divider)] drag-region select-none"
    >
      {/* Left cluster — traffic lights (mac), sidebar toggle */}
      <div className="flex items-center gap-1 min-w-0">
        {isMac && (
          <div className="flex items-center gap-1.5 no-drag mr-1">
            <button
              onClick={() => appWindow.close()}
              className="w-3 h-3 rounded-full bg-[#ff5f57] hover:brightness-90 transition-all"
              title="Close"
            />
            <button
              onClick={() => appWindow.minimize()}
              className="w-3 h-3 rounded-full bg-[#febc2e] hover:brightness-90 transition-all"
              title="Minimize"
            />
            <button
              onClick={() => appWindow.toggleMaximize()}
              className="w-3 h-3 rounded-full bg-[#28c840] hover:brightness-90 transition-all"
              title="Maximize"
            />
          </div>
        )}

        <button
          onClick={toggleSidebar}
          className="no-drag w-7 h-7 flex items-center justify-center rounded-[6px] transition-colors text-text-secondary hover:bg-white/[0.06] hover:text-text-primary"
          title="Toggle sidebar (Ctrl+B)"
        >
          <img src={appIcon} alt="ClaudeTerminal" className="w-[20px] h-[20px]" />
        </button>

        {/* Project breadcrumb — IntelliJ main-toolbar project widget */}
        <button
          onClick={openCommandPalette}
          className="no-drag group flex items-center gap-1.5 h-7 ml-1 pl-2 pr-2 rounded-[6px] hover:bg-white/[0.06] transition-colors max-w-[360px]"
          title={active?.config.working_directory || 'No active terminal'}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${statusDot} flex-shrink-0`} />
          {breadcrumb.sub && (
            <>
              <span className="text-text-tertiary text-[12px] truncate max-w-[100px]">
                {breadcrumb.sub}
              </span>
              <span className="text-text-tertiary/60 text-[11px]">/</span>
            </>
          )}
          <span className="text-text-primary text-[12px] font-medium truncate">
            {breadcrumb.project}
          </span>
          <ChevronDown
            size={11}
            strokeWidth={2}
            className="text-text-tertiary group-hover:text-text-secondary flex-shrink-0"
          />
        </button>

        {/* Branch widget */}
        {gitInfo?.is_git_repo && gitInfo.current_branch && (
          <>
            <span className="w-px h-4 bg-[var(--ij-divider-soft)] mx-0.5" />
            <button
              className="no-drag flex items-center gap-1.5 h-7 px-2 rounded-[6px] hover:bg-white/[0.06] transition-colors"
              title={`Branch: ${gitInfo.current_branch}`}
            >
              <GitBranch size={12} strokeWidth={1.75} className="text-text-secondary" />
              <span className="text-text-primary text-[12px] font-mono truncate max-w-[140px]">
                {gitInfo.current_branch}
              </span>
              <ChevronDown size={11} strokeWidth={2} className="text-text-tertiary" />
            </button>
          </>
        )}
      </div>

      {/* Center spacer + brand (small, right-aligned on the drag zone) */}
      <div className="flex-1 flex items-center justify-center min-w-0 px-3">
        <span className="text-text-tertiary text-[11px] tracking-[0.02em] truncate">
          ClaudeTerminal
          {appVersion && <span className="text-text-tertiary/60 ml-1.5 font-mono">{appVersion}</span>}
        </span>
      </div>

      {/* Right cluster — search, run, tool windows, settings, window controls */}
      <div className="flex items-stretch">
        <div className="flex items-center gap-0.5 pr-2 no-drag">
          <button onClick={toggleChanges} className={iconBtn(changesOpen)} title="File Changes (F2)">
            <FileDiff size={15} strokeWidth={1.75} />
          </button>
          <button onClick={toggleOrchestration} className={iconBtn(orchestrationOpen)} title="Agent Teams (F4)">
            <Users size={15} strokeWidth={1.75} />
          </button>
          <button onClick={toggleHints} className={iconBtn(hintsOpen)} title="Command Hints">
            <Lightbulb size={15} strokeWidth={1.75} />
          </button>

          <div className="w-px h-4 bg-[var(--ij-divider-soft)] mx-1" />

          <button onClick={openSettings} className={iconBtn(false)} title="Settings (Ctrl+,)">
            <Settings size={15} strokeWidth={1.75} />
          </button>
        </div>

        {!isMac && (
          <div className="flex items-stretch no-drag">
            <button
              onClick={() => appWindow.minimize()}
              className="w-[46px] h-9 flex items-center justify-center hover:bg-white/[0.06] text-text-secondary transition-colors"
              title="Minimize"
            >
              <Minus size={12} strokeWidth={1.75} />
            </button>
            <button
              onClick={() => appWindow.toggleMaximize()}
              className="w-[46px] h-9 flex items-center justify-center hover:bg-white/[0.06] text-text-secondary transition-colors"
              title="Maximize"
            >
              <Square size={11} strokeWidth={1.75} />
            </button>
            <button
              onClick={() => appWindow.close()}
              className="w-[46px] h-9 flex items-center justify-center hover:bg-[#E04545] text-text-secondary hover:text-white transition-colors"
              title="Close"
            >
              <X size={13} strokeWidth={2} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
