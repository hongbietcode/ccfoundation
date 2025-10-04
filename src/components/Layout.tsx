import { Outlet } from "react-router-dom";
import { useCurrentStore } from "../lib/query";
import { Button } from "@/components/ui/button";
import { useNavigate } from "react-router-dom";

export function Layout() {
  const navigate = useNavigate();
  const { data: currentStore } = useCurrentStore();

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              <h1 className="text-2xl font-bold">Claude Code Config</h1>
              {currentStore && (
                <div className="text-sm text-muted-foreground bg-muted/50 px-3 py-1 rounded-full">
                  Active store: <span className="font-medium text-foreground">{currentStore.name}</span>
                </div>
              )}
            </div>
            <nav className="flex items-center gap-2">
              <Button
                variant="ghost"
                onClick={() => navigate("/")}
                className="text-sm"
              >
                Stores
              </Button>
              <Button
                variant="ghost"
                onClick={() => navigate("/config/editor")}
                className="text-sm"
              >
                Edit Config
              </Button>
              <Button
                variant="outline"
                onClick={() => navigate("/stores/new")}
                className="text-sm"
              >
                New Store
              </Button>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="border-t mt-auto">
        <div className="container mx-auto px-4 py-4">
          <div className="text-center text-sm text-muted-foreground">
            Claude Code Configuration Manager
          </div>
        </div>
      </footer>
    </div>
  );
}