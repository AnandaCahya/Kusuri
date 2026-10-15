import React from "react";
import { Smartphone, Monitor, Terminal, Globe, Laptop, HelpCircle } from "lucide-react";

interface StatusBadgeProps {
  isOnline: boolean;
  className?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ isOnline, className = "" }) => {
  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium border ${
        isOnline
          ? "bg-emerald-950/60 text-emerald-400 border-emerald-800/60"
          : "bg-slate-800 text-slate-400 border-slate-700"
      } ${className}`}
    >
      <span
        className={`w-1.5 h-1.5 rounded-full ${
          isOnline ? "bg-emerald-400 animate-pulse" : "bg-slate-500"
        }`}
      />
      {isOnline ? "Online" : "Offline"}
    </span>
  );
};

interface DeviceTypeBadgeProps {
  type: string;
}

export const DeviceTypeBadge: React.FC<DeviceTypeBadgeProps> = ({ type }) => {
  const normalized = type.toLowerCase();

  const getIcon = () => {
    switch (normalized) {
      case "android":
        return <Smartphone className="w-3.5 h-3.5 text-emerald-400" />;
      case "windows":
        return <Monitor className="w-3.5 h-3.5 text-sky-400" />;
      case "linux":
        return <Terminal className="w-3.5 h-3.5 text-amber-400" />;
      case "web":
        return <Globe className="w-3.5 h-3.5 text-indigo-400" />;
      default:
        return <Laptop className="w-3.5 h-3.5 text-slate-400" />;
    }
  };

  return (
    <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium bg-slate-800/80 border border-slate-700/80 text-slate-300 capitalize">
      {getIcon()}
      {type}
    </span>
  );
};

interface CommandStatusBadgeProps {
  status: string;
}

export const CommandStatusBadge: React.FC<CommandStatusBadgeProps> = ({ status }) => {
  const normalized = status.toLowerCase();

  let colorClasses = "bg-slate-800 text-slate-300 border-slate-700";
  if (normalized === "completed") {
    colorClasses = "bg-emerald-950/60 text-emerald-400 border-emerald-800/60";
  } else if (normalized === "failed") {
    colorClasses = "bg-rose-950/60 text-rose-400 border-rose-800/60";
  } else if (normalized === "sent") {
    colorClasses = "bg-sky-950/60 text-sky-400 border-sky-800/60";
  } else if (normalized === "pending") {
    colorClasses = "bg-amber-950/60 text-amber-400 border-amber-800/60";
  }

  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border capitalize ${colorClasses}`}
    >
      {status}
    </span>
  );
};
