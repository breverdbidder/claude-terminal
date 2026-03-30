import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { Terminal } from '@xterm/xterm';
import type { WorktreeDetectResult } from '../types/git';

export interface TerminalConfig {
  id: string;
  label: string;
  nickname: string | null;
  profile_id: string | null;
  working_directory: string;
  claude_args: string[];
  env_vars: Record<string, string>;
  created_at: string;
  status: 'Running' | 'Idle' | 'Error' | 'Stopped';
  color_tag: string | null;
}

export interface LoopInfo {
  interval: string;
  prompt: string;
}

interface TerminalInstance {
  config: TerminalConfig;
  xterm: Terminal | null;
  restoredOutput?: string;
  model?: string;
  effort?: string;
  isWorktree: boolean;
  loopInfo?: LoopInfo | null;
  sessionSummary?: string | null;
}

interface TerminalState {
  terminals: Map<string, TerminalInstance>;
  activeTerminalId: string | null;
  unreadTerminalIds: Set<string>;
  gitInfoCache: Map<string, WorktreeDetectResult>;

  createTerminal: (
    label: string,
    workingDirectory: string,
    claudeArgs: string[],
    envVars: Record<string, string>,
    colorTag?: string,
    nickname?: string,
    restoredOutput?: string
  ) => Promise<string>;
  closeTerminal: (id: string) => Promise<void>;
  setActiveTerminal: (id: string) => void;
  updateLabel: (id: string, label: string) => Promise<void>;
  updateNickname: (id: string, nickname: string) => Promise<void>;
  writeToTerminal: (id: string, data: string) => Promise<void>;
  resizeTerminal: (id: string, cols: number, rows: number) => Promise<void>;
  setXterm: (id: string, xterm: Terminal) => void;
  handleTerminalOutput: (id: string, data: Uint8Array) => void;
  updateTerminalStatus: (id: string, status: TerminalConfig['status']) => void;
  setLoopMode: (id: string, info: LoopInfo | null) => void;
  setSessionSummary: (id: string, summary: string | null) => void;
  getTerminalList: () => TerminalConfig[];
  clearUnread: (id: string) => void;
  hasUnread: (id: string) => boolean;
  fetchGitInfo: (terminalId: string) => Promise<void>;
}

export const useTerminalStore = create<TerminalState>((set, get) => ({
  terminals: new Map(),
  activeTerminalId: null,
  unreadTerminalIds: new Set(),
  gitInfoCache: new Map(),

  createTerminal: async (label, workingDirectory, claudeArgs, envVars, colorTag, nickname, restoredOutput) => {
    try {
      const config = await invoke<TerminalConfig>('create_terminal', {
        request: {
          label,
          working_directory: workingDirectory,
          claude_args: claudeArgs,
          env_vars: envVars,
          color_tag: colorTag || null,
          nickname: nickname || null,
        },
      });
      // Parse model, effort, worktree from claude_args
      let model: string | undefined;
      let effort: string | undefined;
      const isWorktree = claudeArgs.includes('--worktree');
      for (let i = 0; i < claudeArgs.length; i++) {
        if (claudeArgs[i] === '--model' && i + 1 < claudeArgs.length) {
          model = claudeArgs[i + 1];
        }
        if (claudeArgs[i] === '--effort' && i + 1 < claudeArgs.length) {
          effort = claudeArgs[i + 1];
        }
      }

      set((state) => {
        const newTerminals = new Map(state.terminals);
        newTerminals.set(config.id, { config, xterm: null, restoredOutput, model, effort, isWorktree });
        return {
          terminals: newTerminals,
          activeTerminalId: config.id,
        };
      });

      // Fetch git info in the background
      get().fetchGitInfo(config.id);

      return config.id;
    } catch (error) {
      console.error('Failed to create terminal:', error);
      throw error;
    }
  },

  closeTerminal: async (id) => {
    await invoke('close_terminal', { id });

    set((state) => {
      const newTerminals = new Map(state.terminals);
      const instance = newTerminals.get(id);
      if (instance?.xterm) {
        instance.xterm.dispose();
      }
      newTerminals.delete(id);

      const newUnread = new Set(state.unreadTerminalIds);
      newUnread.delete(id);

      const newGitCache = new Map(state.gitInfoCache);
      newGitCache.delete(id);

      const remainingIds = Array.from(newTerminals.keys());
      return {
        terminals: newTerminals,
        unreadTerminalIds: newUnread,
        gitInfoCache: newGitCache,
        activeTerminalId: state.activeTerminalId === id
          ? (remainingIds[0] || null)
          : state.activeTerminalId,
      };
    });
  },

  setActiveTerminal: (id) => set((state) => {
    const newUnread = new Set(state.unreadTerminalIds);
    newUnread.delete(id);
    return { activeTerminalId: id, unreadTerminalIds: newUnread };
  }),

  updateLabel: async (id, label) => {
    await invoke('update_terminal_label', { id, label });

    set((state) => {
      const newTerminals = new Map(state.terminals);
      const instance = newTerminals.get(id);
      if (instance) {
        instance.config.label = label;
      }
      return { terminals: newTerminals };
    });
  },

  updateNickname: async (id, nickname) => {
    await invoke('update_terminal_nickname', { id, nickname });

    set((state) => {
      const newTerminals = new Map(state.terminals);
      const instance = newTerminals.get(id);
      if (instance) {
        instance.config.nickname = nickname;
      }
      return { terminals: newTerminals };
    });
  },

  writeToTerminal: async (id, data) => {
    const encoder = new TextEncoder();
    await invoke('write_to_terminal', { id, data: Array.from(encoder.encode(data)) });
  },

  resizeTerminal: async (id, cols, rows) => {
    await invoke('resize_terminal', { id, cols, rows });
  },

  setXterm: (id, xterm) => {
    const { terminals } = get();
    const instance = terminals.get(id);

    // Write restored session output before any live output
    if (instance?.restoredOutput) {
      const lines = instance.restoredOutput.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
      xterm.write('\x1b[90m─── Previous session output ───\x1b[0m\r\n\r\n');
      xterm.write(lines.replace(/\n/g, '\r\n'));
      xterm.write('\r\n\r\n\x1b[90m─── Session restored ───\x1b[0m\r\n\r\n');
    }

    set((state) => {
      const newTerminals = new Map(state.terminals);
      const inst = newTerminals.get(id);
      if (inst) {
        inst.xterm = xterm;
        delete inst.restoredOutput; // Free memory
      }
      return { terminals: newTerminals };
    });
  },

  handleTerminalOutput: (id, data) => {
    const { terminals, activeTerminalId } = get();
    const instance = terminals.get(id);
    if (instance?.xterm) {
      instance.xterm.write(data);
    }
    if (id !== activeTerminalId) {
      set((state) => {
        const newUnread = new Set(state.unreadTerminalIds);
        newUnread.add(id);
        return { unreadTerminalIds: newUnread };
      });
    }
  },

  updateTerminalStatus: (id, status) => {
    set((state) => {
      const newTerminals = new Map(state.terminals);
      const instance = newTerminals.get(id);
      if (instance) {
        instance.config.status = status;
      }
      return { terminals: newTerminals };
    });
  },

  setLoopMode: (id, info) => {
    set((state) => {
      const newTerminals = new Map(state.terminals);
      const instance = newTerminals.get(id);
      if (instance) {
        instance.loopInfo = info;
      }
      return { terminals: newTerminals };
    });
  },

  setSessionSummary: (id, summary) => {
    set((state) => {
      const newTerminals = new Map(state.terminals);
      const instance = newTerminals.get(id);
      if (instance) {
        instance.sessionSummary = summary;
      }
      return { terminals: newTerminals };
    });
  },

  getTerminalList: () => {
    const { terminals } = get();
    return Array.from(terminals.values()).map((t) => t.config);
  },

  clearUnread: (id) => set((state) => {
    const newUnread = new Set(state.unreadTerminalIds);
    newUnread.delete(id);
    return { unreadTerminalIds: newUnread };
  }),

  hasUnread: (id) => {
    return get().unreadTerminalIds.has(id);
  },

  fetchGitInfo: async (terminalId) => {
    const instance = get().terminals.get(terminalId);
    if (!instance) return;

    try {
      const info = await invoke<WorktreeDetectResult>('get_worktree_info', {
        path: instance.config.working_directory,
      });
      set((state) => {
        const newCache = new Map(state.gitInfoCache);
        newCache.set(terminalId, info);
        return { gitInfoCache: newCache };
      });
    } catch {
      // Silently ignore — non-git dirs or git not installed
    }
  },
}));
