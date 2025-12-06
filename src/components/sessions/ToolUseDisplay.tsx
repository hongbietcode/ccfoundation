import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Code, Terminal, FileText, Folder, Loader2 } from "lucide-react";

interface ToolUseDisplayProps {
	toolName: string;
	toolInput?: Record<string, unknown>;
	status?: "running" | "completed" | "error";
}

const TOOL_ICONS: Record<string, React.ReactNode> = {
	read: <FileText className="h-4 w-4" />,
	write: <FileText className="h-4 w-4" />,
	edit: <FileText className="h-4 w-4" />,
	bash: <Terminal className="h-4 w-4" />,
	glob: <Folder className="h-4 w-4" />,
	grep: <Code className="h-4 w-4" />,
};

export function ToolUseDisplay({ toolName, toolInput, status = "running" }: ToolUseDisplayProps) {
	const icon = TOOL_ICONS[toolName.toLowerCase()] || <Code className="h-4 w-4" />;

	const statusColor = {
		running: "bg-blue-500/10 text-blue-500 border-blue-500/20",
		completed: "bg-green-500/10 text-green-500 border-green-500/20",
		error: "bg-red-500/10 text-red-500 border-red-500/20",
	}[status];

	return (
		<Card className={`p-3 ${statusColor} border`}>
			<div className="flex items-center gap-2 mb-2">
				{status === "running" && <Loader2 className="h-3 w-3 animate-spin" />}
				{icon}
				<span className="text-sm font-medium">{toolName}</span>
				<Badge variant="outline" className="ml-auto text-xs">
					{status}
				</Badge>
			</div>
			{toolInput && (
				<div className="text-xs font-mono bg-background/50 rounded p-2 mt-2">
					{Object.entries(toolInput).map(([key, value]) => (
						<div key={key} className="truncate">
							<span className="text-muted-foreground">{key}:</span>{" "}
							<span>{String(value).substring(0, 100)}</span>
						</div>
					))}
				</div>
			)}
		</Card>
	);
}
