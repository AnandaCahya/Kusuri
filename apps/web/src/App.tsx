import "./index.css";
import React, { useState } from "react";
import { ToastProvider } from "./context/ToastContext";
import { AuthProvider, useAuth } from "./context/AuthContext";
import { WebSocketProvider } from "./context/WebSocketContext";
import { Header } from "./components/layout/Header";
import { Sidebar } from "./components/layout/Sidebar";
import { MobileNav } from "./components/layout/MobileNav";

import { AuthView } from "./views/AuthView";
import { OverviewView } from "./views/OverviewView";
import { DevicesView } from "./views/DevicesView";
import { NotificationsView } from "./views/NotificationsView";
import { PairDeviceView } from "./views/PairDeviceView";
import { ClipboardView } from "./views/ClipboardView";
import { FilesView } from "./views/FilesView";
import { CommandsView } from "./views/CommandsView";
import { SettingsView } from "./views/SettingsView";
import { AdminView } from "./views/AdminView";
import { Loader2 } from "lucide-react";

function MainDashboard() {
  const { isAuthenticated, isLoading } = useAuth();
  const [currentTab, setCurrentTab] = useState<string>("overview");

  if (isLoading) {
    return (
      <div className="min-h-screen w-full flex flex-col items-center justify-center bg-slate-950 text-slate-400 gap-3">
        <Loader2 className="w-8 h-8 animate-spin text-indigo-500" />
        <span className="text-xs font-medium">Memuat Kusuri Web Dashboard...</span>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <AuthView />;
  }

  const renderActiveView = () => {
    switch (currentTab) {
      case "overview":
        return <OverviewView onNavigate={setCurrentTab} />;
      case "devices":
        return <DevicesView onNavigate={setCurrentTab} />;
      case "notifications":
        return <NotificationsView />;
      case "pair":
        return <PairDeviceView onNavigate={setCurrentTab} />;
      case "clipboard":
        return <ClipboardView />;
      case "files":
        return <FilesView />;
      case "commands":
        return <CommandsView />;
      case "settings":
        return <SettingsView />;
      case "admin":
        return <AdminView />;
      default:
        return <OverviewView onNavigate={setCurrentTab} />;
    }
  };

  return (
    <div className="min-h-screen bg-slate-950 flex flex-row text-slate-100">
      {/* Sidebar for Desktop */}
      <Sidebar currentTab={currentTab} onSelectTab={setCurrentTab} />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 pb-16 lg:pb-0">
        <Header currentTab={currentTab} />

        <main className="flex-1 p-4 sm:p-6 lg:p-8 overflow-y-auto">
          {renderActiveView()}
        </main>
      </div>

      {/* Mobile Navigation */}
      <MobileNav currentTab={currentTab} onSelectTab={setCurrentTab} />
    </div>
  );
}

export function App() {
  return (
    <ToastProvider>
      <AuthProvider>
        <WebSocketProvider>
          <MainDashboard />
        </WebSocketProvider>
      </AuthProvider>
    </ToastProvider>
  );
}

export default App;
