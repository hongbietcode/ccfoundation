import { useState } from "react";
import { useConfigFile, useWriteConfigFile, type ConfigType } from "./lib/query";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Alert, AlertDescription } from "@/components/ui/alert";
import "./App.css";

function App() {
  const [selectedConfig, setSelectedConfig] = useState<ConfigType>("user");
  const [jsonContent, setJsonContent] = useState<string>("");

  const { data: configFile, isLoading: configLoading } = useConfigFile(selectedConfig);
  const writeConfigMutation = useWriteConfigFile();

  const handleLoadConfig = () => {
    if (configFile && configFile.content) {
      setJsonContent(JSON.stringify(configFile.content, null, 2));
    }
  };

  const handleSaveConfig = () => {
    try {
      const parsedContent = JSON.parse(jsonContent);
      writeConfigMutation.mutate({
        configType: selectedConfig,
        content: parsedContent,
      });
    } catch (error) {
      alert("Invalid JSON format. Please fix the errors and try again.");
    }
  };

  const getConfigDisplayName = (configType: ConfigType) => {
    const displayNames: Record<ConfigType, string> = {
      user: "~/.claude/settings.json",
      project: ".claude/settings.json",
      project_local: ".claude/settings.local.json",
      enterprise_macos: "/Library/Application Support/ClaudeCode/managed-settings.json",
      enterprise_linux: "/etc/claude-code/managed-settings.json",
      enterprise_windows: "C:\\ProgramData\\ClaudeCode\\managed-settings.json",
      mcp_macos: "/Library/Application Support/ClaudeCode/managed-mcp.json",
      mcp_linux: "/etc/claude-code/managed-mcp.json",
      mcp_windows: "C:\\ProgramData\\ClaudeCode\\managed-mcp.json",
    };
    return displayNames[configType];
  };

  return (
    <main className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">
          Claude Code Configuration Manager
        </h1>

        <Tabs defaultValue="raw" className="space-y-6">
          <TabsList>
            <TabsTrigger value="raw">Raw</TabsTrigger>
          </TabsList>

          <TabsContent value="raw" className="space-y-6">
            <div className="bg-white rounded-lg shadow p-6 space-y-6">
              <div>
                <label htmlFor="config-type" className="block text-sm font-medium text-gray-700 mb-2">
                  Configuration File
                </label>
                <Select value={selectedConfig} onValueChange={(value) => setSelectedConfig(value as ConfigType)}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select configuration file" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="user">User Settings ({getConfigDisplayName("user")})</SelectItem>
                    <SelectItem value="project">Project Settings ({getConfigDisplayName("project")})</SelectItem>
                    <SelectItem value="project_local">Project Local Settings ({getConfigDisplayName("project_local")})</SelectItem>
                  </SelectContent>
                </Select>

                {configFile && (
                  <div className="mt-2 text-sm text-gray-600">
                    Path: {configFile.path}
                    {!configFile.exists && (
                      <span className="ml-2 text-orange-600">(File does not exist)</span>
                    )}
                  </div>
                )}
              </div>

              <div>
                <div className="flex justify-between items-center mb-2">
                  <h3 className="text-lg font-medium text-gray-900">JSON Content</h3>
                  <div className="flex gap-2">
                    <Button
                      onClick={handleLoadConfig}
                      disabled={configLoading}
                      variant="outline"
                    >
                      {configLoading ? "Loading..." : "Load"}
                    </Button>
                    <Button
                      onClick={handleSaveConfig}
                      disabled={writeConfigMutation.isPending}
                    >
                      {writeConfigMutation.isPending ? "Saving..." : "Save"}
                    </Button>
                  </div>
                </div>

                <Textarea
                  value={jsonContent}
                  onChange={(e) => setJsonContent(e.target.value)}
                  className="min-h-96 font-mono text-sm"
                  placeholder="Configuration content in JSON format..."
                  spellCheck={false}
                />
              </div>

              {writeConfigMutation.error && (
                <Alert variant="destructive">
                  <AlertDescription>
                    Error: {writeConfigMutation.error.message}
                  </AlertDescription>
                </Alert>
              )}

              {writeConfigMutation.isSuccess && (
                <Alert>
                  <AlertDescription>
                    Configuration saved successfully!
                  </AlertDescription>
                </Alert>
              )}
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </main>
  );
}

export default App;
