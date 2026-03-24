import { Globe, Smartphone } from 'lucide-react';
import { useTerminalStore } from '../store/terminalStore';

interface TeleportActionsProps {
  terminalId: string;
}

export function TeleportActions({ terminalId }: TeleportActionsProps) {
  const { writeToTerminal } = useTerminalStore();

  const handleTeleport = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await writeToTerminal(terminalId, '/teleport\n');
  };

  const handleRemoteControl = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await writeToTerminal(terminalId, '/remote-control\n');
  };

  return (
    <div className="flex items-center gap-0.5">
      <button
        onClick={handleTeleport}
        title="Teleport to Web"
        className="p-1 rounded hover:bg-white/[0.06] text-text-tertiary hover:text-accent-primary transition-colors"
      >
        <Globe size={12} />
      </button>
      <button
        onClick={handleRemoteControl}
        title="Remote Control"
        className="p-1 rounded hover:bg-white/[0.06] text-text-tertiary hover:text-accent-primary transition-colors"
      >
        <Smartphone size={12} />
      </button>
    </div>
  );
}
