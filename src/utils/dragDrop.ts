export const TERMINAL_DRAG_TYPE = 'application/x-claude-terminal';

export interface DragPayload {
  terminalId: string;
  source: 'sidebar' | 'grid' | 'tab';
  sourceIndex?: number;
}

export function setDragData(e: React.DragEvent, payload: DragPayload) {
  e.dataTransfer.setData(TERMINAL_DRAG_TYPE, JSON.stringify(payload));
  e.dataTransfer.effectAllowed = 'move';
}

export function getDragData(e: React.DragEvent): DragPayload | null {
  try {
    const data = e.dataTransfer.getData(TERMINAL_DRAG_TYPE);
    if (!data) return null;
    return JSON.parse(data) as DragPayload;
  } catch {
    return null;
  }
}

export function isTerminalDrag(e: React.DragEvent): boolean {
  return e.dataTransfer.types.includes(TERMINAL_DRAG_TYPE);
}
