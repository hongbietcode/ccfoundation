import { useState, useEffect } from "react";
import { useConfigFile, useWriteConfigFile } from "../lib/query";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function ConfigEditorPage() {
  const [jsonContent, setJsonContent] = useState<string>("");

  const { data: configFile } = useConfigFile("user");
  const writeConfigMutation = useWriteConfigFile();

  useEffect(() => {
    if (configFile && configFile.content) {
      setJsonContent(JSON.stringify(configFile.content, null, 2));
    }
  }, [configFile]);

  const handleSaveConfig = () => {
    try {
      const parsedContent = JSON.parse(jsonContent);
      writeConfigMutation.mutate({
        configType: "user",
        content: parsedContent,
      });
    } catch (error) {
      alert("Invalid JSON format. Please fix the errors and try again.");
    }
  };

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <div className="space-y-2">
        <h1 className="text-2xl font-bold">Configuration Editor</h1>
        <p className="text-muted-foreground">
          Edit your Claude Code configuration in JSON format
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Configuration File</CardTitle>
          <CardDescription>
            Make changes to your current configuration
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {configFile && (
            <div className="text-xs text-muted-foreground">
              {configFile.path}
              {!configFile.exists && (
                <span className="ml-2 text-orange-600">(File does not exist)</span>
              )}
            </div>
          )}

          <div className="space-y-2">
            <div className="flex gap-2">
              <Button
                onClick={handleSaveConfig}
                disabled={writeConfigMutation.isPending}
                size="sm"
              >
                {writeConfigMutation.isPending ? "Saving..." : "Save"}
              </Button>
            </div>

            <Textarea
              value={jsonContent}
              onChange={(e) => setJsonContent(e.target.value)}
              className="min-h-96 font-mono text-sm resize-none border-0 bg-muted/50"
              placeholder="Configuration content in JSON format..."
              spellCheck={false}
            />
          </div>

          {writeConfigMutation.error && (
            <Alert variant="destructive" className="text-sm">
              <AlertDescription>
                {writeConfigMutation.error.message}
              </AlertDescription>
            </Alert>
          )}

          {writeConfigMutation.isSuccess && (
            <Alert className="text-sm">
              <AlertDescription>
                Configuration saved successfully!
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>
    </div>
  );
}