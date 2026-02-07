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
    { value: Priority.LOW, label: 'Low', color: 'bg-blue-500/20 text-blue-400 hover:bg-blue-500/30' },
    { value: Priority.MEDIUM, label: 'Medium', color: 'bg-yellow-500/20 text-yellow-400 hover:bg-yellow-500/30' },
    { value: Priority.HIGH, label: 'High', color: 'bg-orange-500/20 text-orange-400 hover:bg-orange-500/30' },
    { value: Priority.CRITICAL, label: 'Critical', color: 'bg-red-500/20 text-red-400 hover:bg-red-500/30' },
  ];

  return (
    <div className="space-y-4">
      {/* Priority Filter */}
      <div>
        <label className="block text-sm font-medium text-slate-400 mb-2">Priority</label>
        <div className="flex flex-wrap gap-2">
          {priorityOptions.map((option) => (
            <button
              key={option.value}
              onClick={() =>
                setSelectedPriority(selectedPriority === option.value ? null : option.value)
              }
              className={`px-4 py-2 rounded-lg font-medium transition-colors border ${
                selectedPriority === option.value
                  ? `${option.color} border-current`
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600 border-slate-600'
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      {/* Status Filter */}
      <div>
        <label className="block text-sm font-medium text-slate-400 mb-2">Status</label>
        <label className="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            checked={showAcknowledged}
            onChange={(e) => setShowAcknowledged(e.target.checked)}
            className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-primary-600 focus:ring-primary-500 focus:ring-offset-slate-800"
          />
          <span className="text-slate-300">Show acknowledged alerts</span>
        </label>
      </div>
    </div>
  );
}
