export interface WorktreeInfo {
  path: string;
  branch: string | null;
  head_sha: string;
  is_main: boolean;
  is_bare: boolean;
  is_detached: boolean;
}

export interface WorktreeDetectResult {
  is_git_repo: boolean;
  is_worktree: boolean;
  main_repo_path: string | null;
  current_branch: string | null;
  worktree_root: string | null;
}
