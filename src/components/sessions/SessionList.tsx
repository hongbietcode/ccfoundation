import { useState, useCallback } from "react";
import { useParams } from "react-router-dom";
import { useSessions, useCreateSession, useDeleteSession, useSessionStream, type StreamEvent } from "@/lib/sessions-query";
import { SessionCard } from "./SessionCard";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Search, Plus, Loader2, ArrowUpDown, Filter, Sparkles, Trash2, CheckSquare } from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface SessionListProps {
	selectedSessionId?: string;
	onSelectSession?: (sessionId: string) => void;
}

export function SessionList({
	selectedSessionId,
	onSelectSession,
}: SessionListProps) {
	const { projectPath } = useParams<{ projectPath: string }>();
	const decodedPath = projectPath ? decodeURIComponent(projectPath) : "";

	const [searchQuery, setSearchQuery] = useState("");
	const [dialogOpen, setDialogOpen] = useState(false);
	const [newMessage, setNewMessage] = useState("");
	const [sortBy, setSortBy] = useState<"date-new" | "date-old" | "messages">("date-new");
	const [selectionMode, setSelectionMode] = useState(false);
	const [selectedSessions, setSelectedSessions] = useState<Set<string>>(new Set());
	const [pendingTempSessionId, setPendingTempSessionId] = useState<string | null>(null);

	const { data: sessions = [], isLoading } = useSessions(decodedPath);
	const createSession = useCreateSession();
	const deleteSession = useDeleteSession();

	// Handle stream event for pending temp session
	const handleStreamEvent = useCallback((event: StreamEvent) => {
		if (event.type === "sessionIdUpdated" && event.tempId === pendingTempSessionId) {
			console.log("🔄 Session ID updated:", event.tempId, "->", event.realId);
			setPendingTempSessionId(null);
			// Update selection to real session ID
			onSelectSession?.(event.realId);
		}
	}, [pendingTempSessionId, onSelectSession]);

	// Listen for session ID updates on pending temp session
	useSessionStream(pendingTempSessionId || "", handleStreamEvent);

	// Filter sessions
	let filteredSessions = sessions.filter((session) => {
		const matchesSearch = session.title.toLowerCase().includes(searchQuery.toLowerCase());
		return matchesSearch;
	});

	// Sort sessions
	filteredSessions = [...filteredSessions].sort((a, b) => {
		switch (sortBy) {
			case "date-new":
				return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
			case "date-old":
				return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
			case "messages":
				return b.messageCount - a.messageCount;
			default:
				return 0;
		}
	});

	const handleCreateSession = async () => {
		if (!newMessage.trim()) return;

		try {
			const tempSessionId = await createSession.mutateAsync({
				message: newMessage,
				projectPath: decodedPath,
			});
			setNewMessage("");
			setDialogOpen(false);
			// Store temp session ID and select it
			if (tempSessionId) {
				console.log("📝 Created session with temp ID:", tempSessionId);
				setPendingTempSessionId(tempSessionId);
				onSelectSession?.(tempSessionId);
			}
		} catch (error) {
			console.error("Failed to create session:", error);
		}
	};

	const toggleSessionSelection = (sessionId: string) => {
		const newSelected = new Set(selectedSessions);
		if (newSelected.has(sessionId)) {
			newSelected.delete(sessionId);
		} else {
			newSelected.add(sessionId);
		}
		setSelectedSessions(newSelected);
	};

	const handleBulkDelete = async () => {
		if (selectedSessions.size === 0) return;

		try {
			// Delete all selected sessions
			await Promise.all(
				Array.from(selectedSessions).map((sessionId) =>
					deleteSession.mutateAsync({ sessionId, projectPath: decodedPath })
				)
			);
			setSelectedSessions(new Set());
			setSelectionMode(false);
		} catch (error) {
			console.error("Failed to delete sessions:", error);
		}
	};

	return (
		<div className="flex flex-col h-full">
			<div className="p-4 border-b space-y-4">
				<div className="flex items-center justify-between">
					<div>
						<h2 className="text-lg font-semibold">Claude Sessions</h2>
						<p className="text-sm text-muted-foreground">
							{selectionMode && selectedSessions.size > 0
								? `${selectedSessions.size} selected`
								: `${sessions.length} session${sessions.length !== 1 ? "s" : ""}`
							}
						</p>
					</div>

					<div className="flex gap-2">
						{selectionMode ? (
							<>
								<Button
									size="sm"
									variant="destructive"
									onClick={handleBulkDelete}
									disabled={selectedSessions.size === 0 || deleteSession.isPending}
								>
									<Trash2 className="h-4 w-4 mr-2" />
									Delete {selectedSessions.size > 0 ? `(${selectedSessions.size})` : ""}
								</Button>
								<Button
									size="sm"
									variant="outline"
									onClick={() => {
										setSelectionMode(false);
										setSelectedSessions(new Set());
									}}
								>
									Cancel
								</Button>
							</>
						) : (
							<>
								<Button
									size="sm"
									variant="ghost"
									onClick={() => setSelectionMode(true)}
								>
									<CheckSquare className="h-4 w-4 mr-2" />
									Select
								</Button>
							</>
						)}
					</div>

					<Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
						<DialogTrigger asChild>
							<Button size="sm">
								<Plus className="h-4 w-4 mr-2" />
								New
							</Button>
						</DialogTrigger>
						<DialogContent className="sm:max-w-[500px]">
							<DialogHeader>
								<DialogTitle>Create New Session</DialogTitle>
								<DialogDescription>
									Start a new Claude Code session with an initial prompt.
								</DialogDescription>
							</DialogHeader>
							<div className="space-y-4 py-4">
								<div className="space-y-2">
									<Label htmlFor="message">Initial Message</Label>
									<Textarea
										id="message"
										value={newMessage}
										onChange={(e) => setNewMessage(e.target.value)}
										placeholder="What do you want Claude to help with?"
										className="min-h-[120px]"
									/>
								</div>
							</div>
							<DialogFooter>
								<Button
									variant="outline"
									onClick={() => setDialogOpen(false)}
								>
									Cancel
								</Button>
								<Button
									onClick={handleCreateSession}
									disabled={!newMessage.trim() || createSession.isPending}
								>
									{createSession.isPending && (
										<Loader2 className="h-4 w-4 mr-2 animate-spin" />
									)}
									Create Session
								</Button>
							</DialogFooter>
						</DialogContent>
					</Dialog>
				</div>

				<div className="flex gap-2">
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button variant="outline" size="sm" className="flex-1">
								<ArrowUpDown className="h-4 w-4 mr-2" />
								Sort
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="start">
							<DropdownMenuLabel>Sort By</DropdownMenuLabel>
							<DropdownMenuSeparator />
							<DropdownMenuItem onClick={() => setSortBy("date-new")}>
								Date (Newest)
							</DropdownMenuItem>
							<DropdownMenuItem onClick={() => setSortBy("date-old")}>
								Date (Oldest)
							</DropdownMenuItem>
							<DropdownMenuItem onClick={() => setSortBy("messages")}>
								Message Count
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				</div>

				<div className="relative">
					<Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
					<Input
						value={searchQuery}
						onChange={(e) => setSearchQuery(e.target.value)}
						placeholder="Search sessions..."
						className="pl-9"
					/>
				</div>
			</div>

			<ScrollArea className="flex-1">
				<div className="p-4 space-y-2">
					{isLoading ? (
						<div className="text-center text-muted-foreground py-8">
							Loading sessions...
						</div>
					) : filteredSessions.length === 0 ? (
						<div className="text-center py-8 px-4">
							{searchQuery ? (
								<div>
									<Search className="h-12 w-12 mx-auto mb-4 text-muted-foreground/50" />
									<p className="text-sm text-muted-foreground">
										No sessions match "{searchQuery}"
									</p>
								</div>
							) : sessions.length === 0 ? (
								<div className="space-y-4">
									<Sparkles className="h-12 w-12 mx-auto text-primary/50" />
									<div>
										<h3 className="font-semibold mb-2">Welcome to Claude Sessions!</h3>
										<p className="text-sm text-muted-foreground mb-4">
											Get started by creating your first Claude Code session
										</p>
									</div>
									<div className="space-y-2 text-left bg-muted/50 rounded-lg p-4">
										<p className="text-xs font-medium">Quick Start Ideas:</p>
										<ul className="text-xs text-muted-foreground space-y-1">
											<li>• "Help me refactor this codebase for better performance"</li>
											<li>• "Add unit tests for my authentication module"</li>
											<li>• "Review my code and suggest improvements"</li>
											<li>• "Implement a new feature: user dashboard"</li>
										</ul>
									</div>
									<Button onClick={() => setDialogOpen(true)} className="mt-4">
										<Plus className="h-4 w-4 mr-2" />
										Create Your First Session
									</Button>
								</div>
							) : (
								<div>
									<Filter className="h-12 w-12 mx-auto mb-4 text-muted-foreground/50" />
									<p className="text-sm text-muted-foreground">
										No sessions match the current filters
									</p>
								</div>
							)}
						</div>
					) : (
						filteredSessions.map((session) => (
							<div key={session.id} className="relative">
								{selectionMode && (
									<div className="absolute left-2 top-1/2 -translate-y-1/2 z-10">
										<input
											type="checkbox"
											checked={selectedSessions.has(session.id)}
											onChange={() => toggleSessionSelection(session.id)}
											className="h-4 w-4 rounded border-gray-300 cursor-pointer"
											onClick={(e) => e.stopPropagation()}
										/>
									</div>
								)}
								<SessionCard
									session={session}
									isSelected={!selectionMode && selectedSessionId === session.id}
									onClick={() => {
										if (selectionMode) {
											toggleSessionSelection(session.id);
										} else {
											onSelectSession?.(session.id);
										}
									}}
								/>
							</div>
						))
					)}
				</div>
			</ScrollArea>
		</div>
	);
}
