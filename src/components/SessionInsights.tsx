import { Sparkles } from 'lucide-react';

interface SessionInsightsProps {
  summary: string;
}

export function SessionInsights({ summary }: SessionInsightsProps) {
  return (
    <div className="bg-accent-primary/5 ring-1 ring-accent-primary/20 rounded-md p-3 mx-3 mb-3">
      <div className="flex items-start gap-2">
        <Sparkles size={14} className="text-accent-primary flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-text-secondary text-[11px] font-medium mb-1">Session Summary</p>
          <p className="text-text-primary text-[12px] whitespace-pre-wrap leading-relaxed">{summary}</p>
        </div>
      </div>
    </div>
  );
}
