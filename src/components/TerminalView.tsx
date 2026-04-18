import { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { WebglAddon } from '@xterm/addon-webgl';
import { invoke } from '@tauri-apps/api/core';
import { useTerminalStore } from '../store/terminalStore';
import { TerminalSearch } from './TerminalSearch';
import { TerminalStatusBar } from './TerminalStatusBar';
import '@xterm/xterm/css/xterm.css';

interface TerminalViewProps {
  terminalId: string;
}

export function TerminalView({ terminalId }: TerminalViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const searchAddonRef = useRef<SearchAddon | null>(null);
  const [searchVisible, setSearchVisible] = useState(false);
  // Narrow selector — only re-render when THIS terminal's instance changes,
  // not on every output-unread-set update for other terminals.
  const instance = useTerminalStore((s) => s.terminals.get(terminalId));
  // Stable action refs: these are static on the store so pulling them via
  // getState avoids putting them in the effect dep array (which was causing
  // the xterm instance to tear down on every unrelated store update).
  const writeToTerminal = useTerminalStore.getState().writeToTerminal;
  const resizeTerminal = useTerminalStore.getState().resizeTerminal;
  const setXterm = useTerminalStore.getState().setXterm;

  const toggleSearch = useCallback(() => {
    setSearchVisible(prev => !prev);
  }, []);

  useEffect(() => {
    if (!containerRef.current || !instance) return;

    const terminal = new Terminal({
      theme: {
        background: '#101010',
        foreground: '#E5E5E5',
        cursor: '#E5E5E5',
        cursorAccent: '#101010',
        selectionBackground: 'rgba(59, 130, 246, 0.25)',
        black: '#171717',
        red: '#EF4444',
        green: '#4ADE80',
        yellow: '#FBBF24',
        blue: '#3B82F6',
        magenta: '#A855F7',
        cyan: '#22D3EE',
        white: '#E5E5E5',
        brightBlack: '#525252',
        brightRed: '#F87171',
        brightGreen: '#86EFAC',
        brightYellow: '#FDE047',
        brightBlue: '#60A5FA',
        brightMagenta: '#C084FC',
        brightCyan: '#67E8F9',
        brightWhite: '#FFFFFF',
      },
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: 'bar',
      cursorWidth: 2,
      allowProposedApi: true,
      // 5000 lines is ample for reading claude output and caps per-terminal
      // memory — at 8 terminals * default 1000 we'd be underprovisioned.
      scrollback: 5000,
    });

    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon((_event, uri) => {
      invoke('open_external_url', { url: uri }).catch((err) => {
        console.error('Failed to open URL:', err);
      });
    });
    const searchAddon = new SearchAddon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(webLinksAddon);
    terminal.loadAddon(searchAddon);
    searchAddonRef.current = searchAddon;

    terminal.open(containerRef.current);

    // Attach WebGL renderer for GPU-accelerated rendering. Gracefully fall
    // back to the default DOM renderer if the context is lost or unavailable
    // (older GPUs, headless CI, etc.).
    let webglAddon: WebglAddon | null = null;
    try {
      webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => {
        webglAddon?.dispose();
        webglAddon = null;
      });
      terminal.loadAddon(webglAddon);
    } catch (err) {
      console.warn('WebGL renderer unavailable, using DOM fallback:', err);
      webglAddon = null;
    }

    fitAddon.fit();

    // Auto-focus so keyboard input works immediately without requiring a click
    terminal.focus();

    // Re-focus xterm if its textarea loses focus to nothing (body/canvas),
    // which happens in WebView2 after PTY output triggers React re-renders
    // or when clicking the terminal canvas directly.
    const handleBlur = () => {
      requestAnimationFrame(() => {
        const focused = document.activeElement;
        if (!focused || focused === document.body || focused.tagName === 'CANVAS') {
          terminal.focus();
        }
      });
    };
    terminal.textarea?.addEventListener('blur', handleBlur);

    // Handle Ctrl+C (copy) and Ctrl+V (paste) keyboard shortcuts
    terminal.attachCustomKeyEventHandler((e: KeyboardEvent) => {
      const isCtrl = e.ctrlKey || e.metaKey;

      // Ctrl+Shift+F: Toggle search
      if (isCtrl && e.shiftKey && e.key === 'F' && e.type === 'keydown') {
        e.preventDefault();
        toggleSearch();
        return false;
      }

      if (isCtrl && e.key === 'c' && e.type === 'keydown') {
        if (terminal.hasSelection()) {
          navigator.clipboard.writeText(terminal.getSelection());
          terminal.clearSelection();
          return false; // Prevent xterm from sending \x03
        }
        // No selection — let xterm send interrupt signal (Ctrl+C)
        return true;
      }

      // Ctrl+V: Let browser handle paste natively — fires paste event
      // on xterm's internal textarea, which xterm processes via onData.
      // This is more reliable than the async Clipboard API which can fail
      // silently due to focus/permission issues.
      if (isCtrl && e.key === 'v') {
        return false;
      }

      // Ctrl+Z: Send suspend/EOF signal to terminal (prevent browser undo)
      if (isCtrl && !e.shiftKey && e.key === 'z') {
        if (e.type === 'keydown') {
          e.preventDefault();
          writeToTerminal(terminalId, '\x1a');
        }
        return false;
      }

      return true;
    });

    terminal.onData((data) => {
      writeToTerminal(terminalId, data).catch((err) => {
        console.error(`Failed to write to terminal ${terminalId}:`, err);
      });
    });

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      resizeTerminal(terminalId, terminal.cols, terminal.rows);
    });
    resizeObserver.observe(containerRef.current);

    terminalRef.current = terminal;
    setXterm(terminalId, terminal);

    return () => {
      resizeObserver.disconnect();
      terminal.textarea?.removeEventListener('blur', handleBlur);
      searchAddonRef.current = null;
      terminalRef.current = null;
      webglAddon?.dispose();
      terminal.dispose();
    };
    // Intentionally omit store action refs from deps — they are stable via
    // getState() and including them caused the xterm instance to be recreated
    // on every unrelated store update.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [terminalId, !!instance, toggleSearch]);

  return (
    <div className="h-full w-full bg-bg-primary relative flex flex-col">
      <TerminalSearch
        searchAddon={searchAddonRef.current}
        visible={searchVisible}
        onClose={() => setSearchVisible(false)}
      />
      <div
        ref={containerRef}
        className="flex-1 min-h-0 w-full"
        onMouseDown={() => terminalRef.current?.focus()}
      />
      <TerminalStatusBar terminalId={terminalId} />
    </div>
  );
}
