import { useMemo, useState, useCallback } from 'react';
import { AnimatePresence, Reorder } from 'framer-motion';
import { X, Plus, Grid3X3, SplitSquareHorizontal, RotateCw, GitBranch } from 'lucide-react';
import { useTerminalStore } from '../store/terminalStore';
import { useAppStore } from '../store/appStore';
import { TerminalView } from './TerminalView';
import { TerminalGrid } from './TerminalGrid';
import { SplitView } from './SplitView';
import { TeleportActions } from './TeleportActions';
import { SessionInsights } from './SessionInsights';
import { getDragData, isTerminalDrag } from '../utils/dragDrop';

const isMac = navigator.platform.toUpperCase().includes('MAC');

export function TerminalTabs() {
  const { terminals, activeTerminalId, setActiveTerminal, closeTerminal, unreadTerminalIds, gitInfoCache } = useTerminalStore();
  const { openNewTerminalModal, gridMode, toggleGridMode, addToGrid, gridTerminalIds, splitMode, splitTerminalIds, splitOrientation, splitRatio, setSplitOrientation, setSplitRatio, clearSplit, setSplitTerminals, setSplitMode } = useAppStore();
  const terminalList = useMemo(() => Array.from(terminals.values()).map(t => t.config), [terminals]);

  const handleNewTab = () => {
    openNewTerminalModal();
  };

  const handleAddToGrid = (terminalId: string) => {
    addToGrid(terminalId);
    if (!gridMode) {
      toggleGridMode();
    }
  };

  const [splitDropTargetId, setSplitDropTargetId] = useState<string | null>(null);

  const handleSplitWith = (terminalId: string) => {
    if (activeTerminalId && terminalId !== activeTerminalId) {
      setSplitTerminals([activeTerminalId, terminalId]);
      setSplitMode(true);
    }
  };

  const handleTabDragOver = useCallback((e: React.DragEvent, tabTerminalId: string) => {
    if (isTerminalDrag(e)) {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
      setSplitDropTargetId(tabTerminalId);
    }
  }, []);

  const handleTabDragLeave = useCallback((e: React.DragEvent) => {
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      setSplitDropTargetId(null);
    }
  }, []);

  const handleTabDrop = useCallback((e: React.DragEvent, tabTerminalId: string) => {
    e.preventDefault();
    setSplitDropTargetId(null);
    const payload = getDragData(e);
    if (!payload || payload.terminalId === tabTerminalId) return;
    setSplitTerminals([tabTerminalId, payload.terminalId]);
    setSplitMode(true);
  }, [setSplitTerminals, setSplitMode]);

  // If split mode is active with valid terminals, show split view
  if (splitMode && splitTerminalIds && terminals.has(splitTerminalIds[0]) && terminals.has(splitTerminalIds[1])) {
    return (
      <div className="h-full flex flex-col">
        {/* Split Toolbar */}
        <div className="h-10 bg-bg-secondary border-b border-border flex items-center justify-between px-3">
          <div className="flex items-center gap-2">
            <SplitSquareHorizontal size={14} className="text-accent-primary" />
            <span className="text-text-primary text-[12px] font-medium">Split View</span>
            <span className="text-text-tertiary text-[11px]">
              {terminals.get(splitTerminalIds[0])?.config.nickname || terminals.get(splitTerminalIds[0])?.config.label}
              {' | '}
              {terminals.get(splitTerminalIds[1])?.config.nickname || terminals.get(splitTerminalIds[1])?.config.label}
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <button
              onClick={() => setSplitOrientation(splitOrientation === 'horizontal' ? 'vertical' : 'horizontal')}
              className="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] text-text-secondary hover:bg-white/[0.04] transition-colors"
              title="Toggle orientation"
            >
              <RotateCw size={12} />
              {splitOrientation === 'horizontal' ? 'Vertical' : 'Horizontal'}
            </button>
            <button
              onClick={clearSplit}
              className="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] text-text-secondary hover:bg-white/[0.04] transition-colors"
            >
              <X size={12} />
              Exit Split
            </button>
          </div>
        </div>
        <div className="flex-1">
          <SplitView
            terminalIds={splitTerminalIds}
            orientation={splitOrientation}
            ratio={splitRatio}
            onRatioChange={setSplitRatio}
          />
        </div>
      </div>
    );
  }

  // If grid mode is active, show the grid
  if (gridMode) {
    return <TerminalGrid />;
  }

  return (
    <div className="h-full flex flex-col">
      {/* Tab Bar */}
      <div className="h-10 bg-bg-secondary border-b border-border flex items-center justify-between px-1">
        <div className="flex items-center flex-1 min-w-0">
          <Reorder.Group
            axis="x"
            values={terminalList}
            onReorder={() => {}}
            className="flex items-center overflow-x-auto"
          >
            {terminalList.map((terminal) => {
              const instance = terminals.get(terminal.id);
              const model = instance?.model;
              const isWorktree = instance?.isWorktree;
              const loopInfo = instance?.loopInfo;
              const isRunning = terminal.status === 'Running';

              return (
              <Reorder.Item
                key={terminal.id}
                value={terminal}
                className="flex-shrink-0"
              >
                <button
                  onClick={() => setActiveTerminal(terminal.id)}
                  onDragOver={(e) => handleTabDragOver(e, terminal.id)}
                  onDragLeave={handleTabDragLeave}
                  onDrop={(e) => handleTabDrop(e, terminal.id)}
                  className={`group relative flex items-center gap-2 px-3 h-10 text-[12px] transition-colors border-t-2 ${
                    splitDropTargetId === terminal.id
                      ? 'bg-accent-primary/10 text-accent-primary border-t-accent-primary'
                      : activeTerminalId === terminal.id
                        ? 'bg-bg-primary text-text-primary border-t-accent-primary'
                        : 'hover:bg-white/[0.04] text-text-secondary border-t-transparent'
                  }`}
                >
                  {splitDropTargetId === terminal.id && (
                    <SplitSquareHorizontal size={12} className="text-accent-primary flex-shrink-0 animate-pulse" />
                  )}
                  {unreadTerminalIds.has(terminal.id) && activeTerminalId !== terminal.id && (
                    <div className="w-1.5 h-1.5 rounded-full bg-accent-primary flex-shrink-0" />
                  )}
                  {terminal.color_tag && (
                    <div className={`w-2 h-2 rounded-full ${terminal.color_tag} flex-shrink-0`} />
                  )}
                  {/* Badges */}
                  {model && (
                    <span className={`text-[9px] px-1 rounded font-medium flex-shrink-0 ${
                      model === 'opus' ? 'bg-purple-500/20 text-purple-400' :
                      model === 'sonnet' ? 'bg-blue-500/20 text-blue-400' :
                      model === 'haiku' ? 'bg-green-500/20 text-green-400' :
                      'bg-white/[0.06] text-text-tertiary'
                    }`}>
                      {model}
                    </span>
                  )}
                  {isWorktree && (
                    <GitBranch size={10} className="text-cyan-400 flex-shrink-0" />
                  )}
                  {loopInfo && (
                    <span className="w-2 h-2 rounded-full bg-purple-400 animate-pulse flex-shrink-0" title={`Loop: ${loopInfo.interval}`} />
                  )}
                  <span className="max-w-[120px] truncate">{terminal.nickname || terminal.label}</span>
                  {gitInfoCache.get(terminal.id)?.current_branch && (
                    <span className={`text-[11px] font-mono max-w-[60px] truncate ${
                      gitInfoCache.get(terminal.id)?.is_worktree ? 'text-purple-400' : 'text-text-tertiary'
                    }`}>
                      {gitInfoCache.get(terminal.id)?.current_branch}
                    </span>
                  )}
                  {isRunning && (
                    <TeleportActions terminalId={terminal.id} />
                  )}
                  <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                    {activeTerminalId && terminal.id !== activeTerminalId && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleSplitWith(terminal.id);
                        }}
                        className="p-0.5 rounded hover:bg-white/[0.08] text-text-tertiary hover:text-text-secondary transition-colors"
                        title="Split with active terminal"
                      >
                        <SplitSquareHorizontal size={12} />
                      </button>
                    )}
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleAddToGrid(terminal.id);
                      }}
                      className={`p-0.5 rounded hover:bg-white/[0.08] transition-colors ${
                        gridTerminalIds.includes(terminal.id) ? 'text-accent-primary' : 'text-text-tertiary hover:text-text-secondary'
                      }`}
                      title="Add to grid"
                    >
                      <Grid3X3 size={12} />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        closeTerminal(terminal.id);
                      }}
                      className="p-0.5 rounded hover:bg-white/[0.08] text-text-tertiary hover:text-text-secondary"
                    >
                      <X size={12} />
                    </button>
                  </div>
                </button>
              </Reorder.Item>
              );
            })}
          </Reorder.Group>

          <button
            onClick={handleNewTab}
            className="p-1.5 rounded hover:bg-white/[0.04] text-text-tertiary hover:text-text-secondary transition-colors flex-shrink-0 ml-1"
            title="New Terminal"
          >
            <Plus size={14} />
          </button>
        </div>

        {/* Grid Mode Toggle */}
        <div className="flex items-center gap-1 ml-2 flex-shrink-0">
          {gridTerminalIds.length > 0 && (
            <span className="text-[11px] text-text-tertiary mr-1">
              {gridTerminalIds.length} in grid
            </span>
          )}
          <button
            onClick={toggleGridMode}
            className={`flex items-center gap-1.5 px-2 py-1 rounded-md text-[12px] transition-colors ${
              gridTerminalIds.length > 0
                ? 'bg-accent-primary/15 text-accent-primary hover:bg-accent-primary/20'
                : 'hover:bg-white/[0.04] text-text-secondary'
            }`}
            title="Toggle Grid View"
          >
            <Grid3X3 size={14} />
            <span className="hidden sm:inline">Grid</span>
          </button>
        </div>
      </div>

      {/* Terminal Content */}
      <div className="flex-1 relative">
        <AnimatePresence mode="wait">
          {activeTerminalId ? (
            <div
              key={activeTerminalId}
              className="absolute inset-0 flex flex-col"
            >
              <div className="flex-1 min-h-0">
                <TerminalView terminalId={activeTerminalId} />
              </div>
              {(() => {
                const inst = terminals.get(activeTerminalId);
                if (inst?.config.status === 'Stopped' && inst?.sessionSummary) {
                  return <SessionInsights summary={inst.sessionSummary} />;
                }
                return null;
              })()}
            </div>
          ) : (
            <div className="absolute inset-0 flex flex-col items-center justify-center text-text-secondary">
              <p className="text-[13px] text-text-tertiary mb-4">Press {isMac ? 'Cmd' : 'Ctrl'}+Shift+N to start a new terminal</p>
              <div className="flex gap-3">
                <button
                  onClick={handleNewTab}
                  className="flex items-center gap-2 bg-accent-primary hover:bg-accent-secondary text-white py-2 px-5 rounded-md text-[13px] font-medium transition-colors"
                >
                  <Plus size={16} />
                  New Terminal
                </button>
                {terminalList.length > 0 && (
                  <button
                    onClick={toggleGridMode}
                    className="flex items-center gap-2 ring-1 ring-border-light hover:bg-white/[0.04] text-text-primary py-2 px-5 rounded-md text-[13px] font-medium transition-colors"
                  >
                    <Grid3X3 size={16} />
                    Grid View
                  </button>
                )}
              </div>
            </div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
