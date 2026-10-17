import {
  LayoutDashboard,
  Smartphone,
  Bell,
  QrCode,
  ClipboardList,
  FolderOpen,
  Terminal,
  Settings,
} from "lucide-react";

interface MobileNavProps {
  currentTab: string;
  onSelectTab: (tab: string) => void;
}

export const MobileNav: React.FC<MobileNavProps> = ({ currentTab, onSelectTab }) => {
  const items = [
    { id: "overview", label: "Overview", icon: LayoutDashboard },
    { id: "devices", label: "Perangkat", icon: Smartphone },
    { id: "notifications", label: "Notif", icon: Bell },
    { id: "pair", label: "Pair", icon: QrCode },
    { id: "clipboard", label: "Clip", icon: ClipboardList },
    { id: "files", label: "Berkas", icon: FolderOpen },
    { id: "commands", label: "Cmds", icon: Terminal },
    { id: "settings", label: "Setting", icon: Settings },
  ];

  return (
    <nav className="lg:hidden fixed bottom-0 left-0 right-0 z-40 bg-slate-950/95 backdrop-blur-md border-t border-slate-800 px-2 py-1.5 flex items-center justify-around">
      {items.map((item) => {
        const Icon = item.icon;
        const isActive = currentTab === item.id;
        return (
          <button
            key={item.id}
            onClick={() => onSelectTab(item.id)}
            className={`flex flex-col items-center gap-1 py-1 px-2 rounded-xl text-[10px] font-medium transition cursor-pointer ${
              isActive
                ? "text-indigo-400"
                : "text-slate-400 hover:text-slate-200"
            }`}
          >
            <Icon className="w-5 h-5" />
            <span>{item.label}</span>
          </button>
        );
      })}
    </nav>
  );
};
