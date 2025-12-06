import { useState, useEffect, useRef, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
	useSession,
	useSessionMessages,
	useResumeSession,
	useCancelSession,
	useDeleteSession,
	useSessionStream,
	getMessageText,
	type StreamEvent,
} from "@/lib/sessions-query";
import { SessionMessages } from "./SessionMessages";
import { SessionStats } from "./SessionStats";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
	DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { formatDistanceToNow } from "date-fns";
import {
	MoreVertical,
	Send,
	Square,
	Trash2,
	Loader2,
	Copy,
	FileJson,
	FileText,
	Code
} from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

interface SessionDetailProps {
	sessionId: string;
}

export function SessionDetail({ sessionId }: SessionDetailProps) {
	const navigate = useNavigate();
	const scrollRef = useRef<HTMLDivElement>(null);
	const { projectPath } = useParams<{ projectPath: string }>();
	const decodedPath = projectPath ? decodeURIComponent(projectPath) : "";

	const { data: session } = useSession(decodedPath, sessionId);
	const { data: messages = [], refetch: refetchMessages } = useSessionMessages(
		decodedPath,
		sessionId
	);
	const resumeSession = useResumeSession();
	const cancelSession = useCancelSession();
	const deleteSession = useDeleteSession();

	const [message, setMessage] = useState("");
	const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
	const [streamingContent, setStreamingContent] = useState("");
	const [realSessionId, setRealSessionId] = useState<string | null>(null);

	// Active session ID (real if available, otherwise temp)
	const activeSessionId = realSessionId || sessionId;

	// Handle stream events
	const handleStreamEvent = useCallback((event: StreamEvent) => {
		console.log("📥 Received stream event:", event);

		if (event.type === "sessionIdUpdated") {
			console.log("🔄 Session ID updated:", event.tempId, "->", event.realId);
			// Update to real session ID
			setRealSessionId(event.realId);
			// Navigate to new URL with real session ID
			navigate(`/sessions/${encodeURIComponent(decodedPath)}/${event.realId}`, { replace: true });
			// Refetch messages with real session ID
			refetchMessages();
		} else if (event.type === "messageStart") {
			console.log("🎬 Message started:", event.messageId);
			setStreamingContent("");
		} else if (event.type === "contentDelta") {
			console.log("📝 Content delta:", event.delta.substring(0, 50));
			setStreamingContent((prev) => prev + event.delta);
		} else if (event.type === "messageComplete") {
			console.log("✅ Message completed");
			setStreamingContent("");
			refetchMessages();
		}
	}, [navigate, decodedPath, refetchMessages]);

	useSessionStream(activeSessionId, handleStreamEvent);

	// Auto-scroll to bottom when messages change
	useEffect(() => {
		if (scrollRef.current) {
			scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
		}
	}, [messages, streamingContent]);

	const handleSend = async () => {
		if (!message.trim() || resumeSession.isPending) return;

		console.log("🚀 Sending message to session:", {
			sessionId: activeSessionId,
			message: message.substring(0, 50),
			projectPath: decodedPath,
		});

		try {
			await resumeSession.mutateAsync({
				sessionId: activeSessionId,
				message,
				projectPath: decodedPath,
			});
			console.log("✅ Message sent successfully");
			setMessage("");
		} catch (error) {
			console.error("❌ Failed to resume session:", error);
		}
	};

	const handleCancel = async () => {
		try {
			await cancelSession.mutateAsync(activeSessionId);
		} catch (error) {
			console.error("Failed to cancel session:", error);
		}
	};

	const handleDelete = async () => {
		try {
			await deleteSession.mutateAsync({
				sessionId: activeSessionId,
				projectPath: decodedPath,
			});
			setDeleteDialogOpen(false);
			navigate(-1);
		} catch (error) {
			console.error("Failed to delete session:", error);
		}
	};

	const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
		if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			handleSend();
		}
	};

	const exportToMarkdown = () => {
		if (!session || !messages.length) return;

		let markdown = `# ${session.title}\n\n`;
		markdown += `Created: ${new Date(session.createdAt).toLocaleString()}\n`;
		markdown += `Model: ${session.model || "Default"}\n`;
		markdown += `Messages: ${session.messageCount}\n\n`;
		markdown += `---\n\n`;

		messages.forEach((msg) => {
			const role = msg.msgType === "user" ? "You" : "Claude";
			const text = getMessageText(msg);
			markdown += `## ${role}\n\n${text}\n\n`;

			if (msg.usage) {
				markdown += `_Tokens: ${msg.usage.inputTokens} in / ${msg.usage.outputTokens} out_\n\n`;
			}
		});

		return markdown;
	};

	const exportToJSON = () => {
		if (!session || !messages.length) return "";

		return JSON.stringify({
			session,
			messages,
		}, null, 2);
	};

	const handleCopyToClipboard = async () => {
		const markdown = exportToMarkdown();
		if (!markdown) return;

		try {
			await writeText(markdown);
			// Could show a toast notification here
		} catch (error) {
			console.error("Failed to copy to clipboard:", error);
		}
	};

	const handleExportMarkdown = async () => {
		const markdown = exportToMarkdown();
		if (!markdown) return;

		try {
			const filePath = await save({
				defaultPath: `${session?.title || "session"}.md`,
				filters: [{ name: "Markdown", extensions: ["md"] }],
			});

			if (filePath) {
				await writeTextFile(filePath, markdown);
			}
		} catch (error) {
			console.error("Failed to export markdown:", error);
		}
	};

	const handleExportJSON = async () => {
		const json = exportToJSON();
		if (!json) return;

		try {
			const filePath = await save({
				defaultPath: `${session?.title || "session"}.json`,
				filters: [{ name: "JSON", extensions: ["json"] }],
			});

			if (filePath) {
				await writeTextFile(filePath, json);
			}
		} catch (error) {
			console.error("Failed to export JSON:", error);
		}
	};

	if (!session) {
		return (
			<div className="flex items-center justify-center h-full">
				<Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		);
	}

	return (
		<div className="flex flex-col h-full">
			{/* Header */}
			<div className="p-4 border-b">
				<div className="flex items-start justify-between gap-4">
					<div className="flex-1 min-w-0">
						<h2 className="text-lg font-semibold truncate">{session.title}</h2>
						<div className="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
							<span>
								Created{" "}
								{formatDistanceToNow(new Date(session.createdAt), {
									addSuffix: true,
								})}
							</span>
							{session.model && (
								<>
									<span>•</span>
									<span>Model: {session.model}</span>
								</>
							)}
						</div>
					</div>

					<div className="flex items-center gap-2">
						{resumeSession.isPending && (
							<Button
								size="sm"
								variant="outline"
								onClick={handleCancel}
								disabled={cancelSession.isPending}
							>
								<Square className="h-4 w-4 mr-2" />
								Stop
							</Button>
						)}

						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button size="sm" variant="ghost">
									<MoreVertical className="h-4 w-4" />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end">
								<DropdownMenuItem onClick={handleCopyToClipboard}>
									<Copy className="h-4 w-4 mr-2" />
									Copy to Clipboard
								</DropdownMenuItem>
								<DropdownMenuItem onClick={handleExportMarkdown}>
									<FileText className="h-4 w-4 mr-2" />
									Export as Markdown
								</DropdownMenuItem>
								<DropdownMenuItem onClick={handleExportJSON}>
									<FileJson className="h-4 w-4 mr-2" />
									Export as JSON
								</DropdownMenuItem>
								<DropdownMenuSeparator />
								<DropdownMenuItem
									onClick={() => setDeleteDialogOpen(true)}
									className="text-destructive"
								>
									<Trash2 className="h-4 w-4 mr-2" />
									Delete Session
								</DropdownMenuItem>
							</DropdownMenuContent>
						</DropdownMenu>
					</div>
				</div>
			</div>

			{/* Statistics */}
			<div className="p-4 border-b">
				<SessionStats session={session} messages={messages} />
			</div>

			{/* Messages */}
			<ScrollArea className="flex-1 p-4" ref={scrollRef}>
				<SessionMessages messages={messages} />
				{streamingContent && (
					<div className="mt-4 p-4 bg-gradient-to-br from-primary/5 to-primary/10 border border-primary/30 rounded-lg animate-in fade-in duration-200">
						<div className="flex items-center gap-2 mb-3">
							<div className="flex items-center gap-2">
								<div className="relative">
									<Loader2 className="h-4 w-4 animate-spin text-primary" />
									<div className="absolute inset-0 h-4 w-4 animate-ping bg-primary/20 rounded-full" />
								</div>
								<span className="font-semibold text-sm">Claude is thinking...</span>
							</div>
							<div className="ml-auto flex gap-1">
								<div className="h-1.5 w-1.5 rounded-full bg-primary animate-pulse" style={{ animationDelay: "0ms" }} />
								<div className="h-1.5 w-1.5 rounded-full bg-primary animate-pulse" style={{ animationDelay: "150ms" }} />
								<div className="h-1.5 w-1.5 rounded-full bg-primary animate-pulse" style={{ animationDelay: "300ms" }} />
							</div>
						</div>
						<div className="prose prose-sm dark:prose-invert max-w-none">
							{streamingContent}
						</div>
						{streamingContent.includes("```") && (
							<div className="mt-2 text-xs text-muted-foreground flex items-center gap-1">
								<Code className="h-3 w-3" />
								<span>Generating code...</span>
							</div>
						)}
					</div>
				)}
			</ScrollArea>

			{/* Input */}
			<div className="p-4 border-t">
				<div className="flex gap-2">
					<Textarea
						value={message}
						onChange={(e) => setMessage(e.target.value)}
						onKeyDown={handleKeyDown}
						placeholder="Send a message to continue this session..."
						disabled={resumeSession.isPending}
						className="min-h-[100px] resize-none"
					/>
					<Button
						onClick={handleSend}
						disabled={resumeSession.isPending || !message.trim()}
						size="icon"
					>
						{resumeSession.isPending ? (
							<Loader2 className="h-4 w-4 animate-spin" />
						) : (
							<Send className="h-4 w-4" />
						)}
					</Button>
				</div>
			</div>

			{/* Delete Dialog */}
			<AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Delete Session</AlertDialogTitle>
						<AlertDialogDescription>
							Are you sure you want to delete this session? This will
							permanently delete the session file from{" "}
							<code>~/.claude/projects/</code>. This action cannot be undone.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Cancel</AlertDialogCancel>
						<AlertDialogAction
							onClick={handleDelete}
							className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
						>
							Delete
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</div>
	);
}
