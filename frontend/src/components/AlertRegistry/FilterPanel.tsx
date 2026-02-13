import { Priority } from '../../types/contracts';

interface FilterPanelProps {
  selectedPriority: number | null;
  setSelectedPriority: (priority: number | null) => void;
  showAcknowledged: boolean;
  setShowAcknowledged: (show: boolean) => void;
}

export default function FilterPanel({
  selectedPriority,
  setSelectedPriority,
  showAcknowledged,
  setShowAcknowledged,
}: FilterPanelProps) {
  const priorityOptions = [
    { value: Priority.LOW, label: 'Low', color: 'cyan' },
    { value: Priority.MEDIUM, label: 'Medium', color: 'blue' },
    { value: Priority.HIGH, label: 'Orange', color: 'orange' },
    { value: Priority.CRITICAL, label: 'Critical', color: 'red' },
  ];

  return (
    <div className="space-y-8">
      {/* Priority Filter */}
      <div>
        <label className="block text-[10px] font-black text-slate-500 uppercase tracking-widest mb-4">Threat Priority</label>
        <div className="grid grid-cols-2 gap-3">
          {priorityOptions.map((option) => (
            <button
              key={option.value}
              onClick={() =>
                setSelectedPriority(selectedPriority === option.value ? null : option.value)
              }
              className={`px-4 py-3 rounded-xl font-bold text-xs transition-all border ${selectedPriority === option.value
                  ? `bg-${option.color}-500/10 border-${option.color}-500/40 text-${option.color}-400 shadow-[0_0_15px_rgba(0,0,0,0.2)]`
                  : 'bg-slate-900 shadow-inner text-slate-500 hover:text-slate-300 border-white/5'
                }`}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      {/* Status Filter */}
      <div>
        <label className="block text-[10px] font-black text-slate-500 uppercase tracking-widest mb-4">Status Filter</label>
        <label className="flex items-center group cursor-pointer">
          <div className="relative flex items-center">
            <input
              type="checkbox"
              checked={showAcknowledged}
              onChange={(e) => setShowAcknowledged(e.target.checked)}
              className="peer sr-only"
            />
            <div className="w-10 h-5 bg-slate-800 rounded-full peer-checked:bg-cyan-500 transition-colors"></div>
            <div className="absolute left-1 top-1 w-3 h-3 bg-white rounded-full peer-checked:translate-x-5 transition-transform"></div>
          </div>
          <span className="ml-3 text-sm font-bold text-slate-400 group-hover:text-slate-200 transition-colors">
            Include Resolved
          </span>
        </label>
      </div>
    </div>
  );
}
