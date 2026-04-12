import { useEffect, useRef } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { CheckCircle, XCircle, AlertTriangle, Info, X } from 'lucide-react';
import { useToastStore } from '../store/toastStore';
import type { ToastType } from '../store/toastStore';

const ICON_MAP: Record<ToastType, typeof CheckCircle> = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
};

const COLOR_MAP: Record<ToastType, { icon: string; bar: string; bg: string }> = {
  success: {
    icon: 'text-success',
    bar: 'bg-success',
    bg: 'bg-success/[0.06]',
  },
  error: {
    icon: 'text-error',
    bar: 'bg-error',
    bg: 'bg-error/[0.06]',
  },
  warning: {
    icon: 'text-warning',
    bar: 'bg-warning',
    bg: 'bg-warning/[0.06]',
  },
  info: {
    icon: 'text-accent-primary',
    bar: 'bg-accent-primary',
    bg: 'bg-accent-primary/[0.06]',
  },
};

function ToastItem({ id, type, title, message, duration }: {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration: number;
}) {
  const removeToast = useToastStore((s) => s.removeToast);
  const progressRef = useRef<HTMLDivElement>(null);
  const Icon = ICON_MAP[type];
  const colors = COLOR_MAP[type];

  useEffect(() => {
    const el = progressRef.current;
    if (!el || duration <= 0) return;
    // Start the shrink animation on next frame so the transition applies
    requestAnimationFrame(() => {
      el.style.transition = `width ${duration}ms linear`;
      el.style.width = '0%';
    });
  }, [duration]);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, x: 80, scale: 0.95 }}
      animate={{ opacity: 1, x: 0, scale: 1 }}
      exit={{ opacity: 0, x: 80, scale: 0.95 }}
      transition={{ duration: 0.2, ease: 'easeOut' }}
      className={`
        relative overflow-hidden rounded-lg
        bg-elevation-3 ${colors.bg}
        ring-1 ring-white/[0.08]
        shadow-elevation-3
        w-[320px] pointer-events-auto
      `}
    >
      {/* Content */}
      <div className="flex items-start gap-2.5 px-3 py-2.5">
        <Icon size={16} className={`${colors.icon} mt-0.5 shrink-0`} />
        <div className="flex-1 min-w-0">
          <p className="text-text-primary text-[12px] font-medium leading-tight">
            {title}
          </p>
          {message && (
            <p className="text-text-secondary text-[11px] mt-0.5 leading-snug">
              {message}
            </p>
          )}
        </div>
        <button
          onClick={() => removeToast(id)}
          className="text-text-tertiary hover:text-text-secondary transition-colors shrink-0 mt-0.5"
        >
          <X size={13} />
        </button>
      </div>

      {/* Progress bar */}
      {duration > 0 && (
        <div className="h-[2px] w-full bg-white/[0.04]">
          <div
            ref={progressRef}
            className={`h-full ${colors.bar} opacity-40`}
            style={{ width: '100%' }}
          />
        </div>
      )}
    </motion.div>
  );
}

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);

  return (
    <div className="fixed bottom-8 right-3 z-[9999] flex flex-col-reverse gap-2 pointer-events-none">
      <AnimatePresence mode="popLayout">
        {toasts.map((t) => (
          <ToastItem key={t.id} {...t} />
        ))}
      </AnimatePresence>
    </div>
  );
}
