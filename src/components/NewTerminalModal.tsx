import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { X, FolderOpen, Terminal, Zap } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store/appStore';
import { useTerminalStore } from '../store/terminalStore';
import { homeDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';

const isMac = navigator.platform.toUpperCase().includes('MAC');

interface ConfigProfile {
  id: string;
  name: string;
  description: string | null;
  working_directory: string;
  claude_args: string[];
  env_vars: Record<string, string>;
  is_default: boolean;
}

const TAG_COLORS = [
  'bg-red-500',
  'bg-orange-500',
  'bg-yellow-500',
  'bg-green-500',
  'bg-blue-500',
  'bg-purple-500',
  'bg-pink-500',
];

export function NewTerminalModal() {
  const { closeNewTerminalModal, defaultClaudeArgs } = useAppStore();
  const { terminals, createTerminal } = useTerminalStore();

  const [profiles, setProfiles] = useState<ConfigProfile[]>([]);
  const [selectedProfileId, setSelectedProfileId] = useState<string | null>(null);
  const [nickname, setNickname] = useState('');
  const [workingDirectory, setWorkingDirectory] = useState('');
  const [claudeArgs, setClaudeArgs] = useState<string[]>(defaultClaudeArgs);
  const [envVars, setEnvVars] = useState<Record<string, string>>({});
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [defaultDirectory, setDefaultDirectory] = useState('');
  const [useWorktree, setUseWorktree] = useState(false);
  const [selectedModel, setSelectedModel] = useState<'default' | 'opus' | 'sonnet' | 'haiku'>('default');
  const [selectedEffort, setSelectedEffort] = useState<'default' | 'low' | 'medium' | 'high'>('default');

  useEffect(() => {
    loadProfiles();
    loadDefaultDirectory();
  }, []);

  useEffect(() => {
    // When profile is selected, update form with profile settings
    if (selectedProfileId) {
      const profile = profiles.find(p => p.id === selectedProfileId);
      if (profile) {
        setWorkingDirectory(profile.working_directory || defaultDirectory);
        setClaudeArgs(profile.claude_args.length > 0 ? profile.claude_args : defaultClaudeArgs);
        setEnvVars(profile.env_vars || {});
      }
    } else {
      // Reset to defaults when "No Profile" is selected
      setWorkingDirectory(defaultDirectory);
      setClaudeArgs(defaultClaudeArgs);
      setEnvVars({});
    }
  }, [selectedProfileId, profiles, defaultDirectory, defaultClaudeArgs]);

  const loadProfiles = async () => {
    try {
      const loadedProfiles = await invoke<ConfigProfile[]>('get_profiles');
      setProfiles(loadedProfiles);

      // Select default profile if exists
      const defaultProfile = loadedProfiles.find(p => p.is_default);
      if (defaultProfile) {
        setSelectedProfileId(defaultProfile.id);
      }
    } catch (error) {
      console.error('Failed to load profiles:', error);
    }
  };

  const loadDefaultDirectory = async () => {
    try {
      const home = await homeDir();
      setWorkingDirectory(home);
      setDefaultDirectory(home);
    } catch (error) {
      console.error('Failed to get home directory:', error);
    }
  };

  const handleBrowseDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: workingDirectory,
      });
      if (selected && typeof selected === 'string') {
        setWorkingDirectory(selected);
      }
    } catch (error) {
      console.error('Failed to open directory picker:', error);
    }
  };

  const handleCreateTerminal = async () => {
    setError(null);

    // Validate working directory is not empty
    if (!workingDirectory.trim()) {
      setError('Working directory is required.');
      return;
    }

    // Validate claude args don't contain shell metacharacters
    const dangerousPattern = /[;&|`$(){}<>^\n\r'"\\~*?[\]!#\t]/;
    for (const arg of claudeArgs) {
      if (dangerousPattern.test(arg)) {
        setError(`Invalid character in argument: "${arg}". Remove shell metacharacters.`);
        return;
      }
    }

    setIsCreating(true);
    try {
      const selectedProfile = profiles.find(p => p.id === selectedProfileId);
      const baseName = selectedProfile?.name || 'Terminal';
      const label = `${baseName} ${terminals.size + 1}`;
      const colorTag = TAG_COLORS[terminals.size % TAG_COLORS.length];

      // Build final args with model, effort, and worktree prepended
      const finalArgs = [...claudeArgs];
      if (selectedModel !== 'default') {
        finalArgs.unshift('--model', selectedModel);
      }
      if (selectedEffort !== 'default') {
        finalArgs.unshift('--effort', selectedEffort);
      }
      if (useWorktree) {
        finalArgs.unshift('--worktree');
      }

      await createTerminal(
        label,
        workingDirectory,
        finalArgs,
        envVars,
        colorTag,
        nickname || undefined
      );

      closeNewTerminalModal();
    } catch (err) {
      console.error('Failed to create terminal:', err);
      setError(String(err));
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15 }}
      className="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
      onClick={closeNewTerminalModal}
    >
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.15 }}
        onClick={(e) => e.stopPropagation()}
        className="bg-bg-elevated ring-1 ring-white/[0.08] rounded-lg shadow-2xl w-full max-w-lg max-h-[90vh] flex flex-col overflow-hidden"
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <div className="flex items-center gap-2">
            <Terminal size={16} className="text-text-secondary" />
            <h2 className="text-text-primary text-[14px] font-semibold">New Terminal</h2>
          </div>
          <button
            onClick={closeNewTerminalModal}
            className="p-1 rounded hover:bg-white/[0.06] text-text-tertiary transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4 overflow-y-auto flex-1 min-h-0">
          {/* Nickname */}
          <div>
            <label className="block text-text-secondary text-[12px] mb-1.5">
              Nickname
            </label>
            <input
              type="text"
              value={nickname}
              onChange={(e) => setNickname(e.target.value)}
              placeholder="e.g., My Project, Backend API"
              className="w-full bg-bg-primary ring-1 ring-border-light rounded-md h-9 px-3 text-text-primary text-[13px] focus:outline-none focus:ring-accent-primary transition-colors"
            />
          </div>

          {/* Profile Selection */}
          {profiles.length > 0 && (
            <div>
              <label className="block text-text-secondary text-[12px] mb-1.5">
                Profile
              </label>
              <div className="grid grid-cols-2 gap-2">
                <button
                  onClick={() => setSelectedProfileId(null)}
                  className={`p-2.5 rounded-md text-left transition-colors ${
                    selectedProfileId === null
                      ? 'bg-accent-primary/10 ring-1 ring-accent-primary/30'
                      : 'bg-bg-primary ring-1 ring-border hover:ring-border-light'
                  }`}
                >
                  <p className="text-text-primary text-[12px] font-medium">No Profile</p>
                  <p className="text-text-tertiary text-[11px]">Custom settings</p>
                </button>
                {profiles.map((profile) => (
                  <button
                    key={profile.id}
                    onClick={() => setSelectedProfileId(profile.id)}
                    className={`p-2.5 rounded-md text-left transition-colors ${
                      selectedProfileId === profile.id
                        ? 'bg-accent-primary/10 ring-1 ring-accent-primary/30'
                        : 'bg-bg-primary ring-1 ring-border hover:ring-border-light'
                    }`}
                  >
                    <p className="text-text-primary text-[12px] font-medium truncate">{profile.name}</p>
                    <p className="text-text-tertiary text-[11px] truncate">
                      {profile.description || profile.working_directory || 'No description'}
                    </p>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Working Directory */}
          <div>
            <label className="block text-text-secondary text-[12px] mb-1.5">
              Working Directory
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={workingDirectory}
                onChange={(e) => setWorkingDirectory(e.target.value)}
                className="flex-1 bg-bg-primary ring-1 ring-border-light rounded-md h-9 px-3 text-text-primary text-[13px] focus:outline-none focus:ring-accent-primary transition-colors"
                placeholder={isMac ? "/path/to/project" : "C:\\path\\to\\project"}
              />
              <button
                onClick={handleBrowseDirectory}
                className="px-3 h-9 bg-bg-primary ring-1 ring-border-light rounded-md hover:bg-white/[0.04] transition-colors"
              >
                <FolderOpen size={16} className="text-text-secondary" />
              </button>
            </div>
          </div>

          {/* Claude Arguments */}
          <div>
            <label className="block text-text-secondary text-[12px] mb-1.5">
              Claude Arguments (one per line)
            </label>
            <textarea
              value={claudeArgs.join('\n')}
              onChange={(e) => setClaudeArgs(e.target.value.split('\n').filter(Boolean))}
              className="w-full bg-bg-primary ring-1 ring-border-light rounded-md py-2 px-3 text-text-primary text-[13px] focus:outline-none focus:ring-accent-primary font-mono h-20 resize-none transition-colors"
              placeholder="--dangerously-skip-permissions&#10;--model opus"
            />
            <p className="text-text-tertiary text-[11px] mt-1">
              Command: <code className="text-text-secondary">claude {claudeArgs.join(' ')}</code>
            </p>
          </div>
          {/* Worktree Toggle */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-text-secondary text-[12px]">Isolated Worktree</label>
              <p className="text-text-tertiary text-[11px]">Run in a separate git worktree</p>
            </div>
            <button
              onClick={() => setUseWorktree(!useWorktree)}
              className={`relative w-9 h-5 rounded-full transition-colors ${
                useWorktree ? 'bg-accent-primary' : 'bg-border-light'
              }`}
            >
              <div
                className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                  useWorktree ? 'translate-x-4' : 'translate-x-0.5'
                }`}
              />
            </button>
          </div>

          {/* Model Selector */}
          <div>
            <label className="block text-text-secondary text-[12px] mb-1.5">Model</label>
            <div className="flex gap-1.5">
              {(['default', 'opus', 'sonnet', 'haiku'] as const).map((model) => (
                <button
                  key={model}
                  onClick={() => setSelectedModel(model)}
                  className={`px-3 py-1.5 rounded-md text-[12px] font-medium transition-colors ${
                    selectedModel === model
                      ? model === 'opus' ? 'bg-purple-500/20 text-purple-400 ring-1 ring-purple-500/30'
                      : model === 'sonnet' ? 'bg-blue-500/20 text-blue-400 ring-1 ring-blue-500/30'
                      : model === 'haiku' ? 'bg-green-500/20 text-green-400 ring-1 ring-green-500/30'
                      : 'bg-accent-primary/10 text-accent-primary ring-1 ring-accent-primary/30'
                      : 'bg-bg-primary ring-1 ring-border-light text-text-secondary hover:ring-border'
                  }`}
                >
                  {model === 'default' ? 'Default' : model.charAt(0).toUpperCase() + model.slice(1)}
                </button>
              ))}
            </div>
          </div>

          {/* Effort Selector */}
          <div>
            <label className="block text-text-secondary text-[12px] mb-1.5">Effort</label>
            <div className="flex gap-1.5">
              {(['default', 'low', 'medium', 'high'] as const).map((effort) => (
                <button
                  key={effort}
                  onClick={() => setSelectedEffort(effort)}
                  className={`px-3 py-1.5 rounded-md text-[12px] font-medium transition-colors ${
                    selectedEffort === effort
                      ? 'bg-accent-primary/10 text-accent-primary ring-1 ring-accent-primary/30'
                      : 'bg-bg-primary ring-1 ring-border-light text-text-secondary hover:ring-border'
                  }`}
                >
                  {effort === 'default' ? 'Default' : effort.charAt(0).toUpperCase() + effort.slice(1)}
                </button>
              ))}
            </div>
          </div>

          {error && (
            <div className="p-3 rounded-md bg-error/5 ring-1 ring-error/20">
              <p className="text-error text-[12px]">{error}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 p-3 border-t border-border">
          <button
            onClick={closeNewTerminalModal}
            className="px-4 h-9 text-text-secondary hover:text-text-primary hover:bg-white/[0.04] rounded-md text-[13px] transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleCreateTerminal}
            disabled={isCreating || !workingDirectory}
            className="flex items-center gap-2 bg-accent-primary hover:bg-accent-secondary disabled:opacity-50 disabled:cursor-not-allowed text-white h-9 px-4 rounded-md text-[13px] font-medium transition-colors"
          >
            <Zap size={14} />
            {isCreating ? 'Creating...' : 'Start Terminal'}
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}
