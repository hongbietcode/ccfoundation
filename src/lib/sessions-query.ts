import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Session Types
export interface Session {
	id: string;
	projectPath: string;
	title: string;
	createdAt: string;
	updatedAt: string;
	messageCount: number;
	model?: string;
	filePath: string;
}

export interface SessionMessage {
	parentUuid?: string;
	uuid?: string;
	sessionId: string;
	timestamp: string;
	msgType: "user" | "assistant" | "summary" | "other";
	message?: {
		role: string;
		content?: unknown;
	};
	cwd?: string;
	version?: string;
	isSidechain?: boolean;
	id?: string;
	model?: string;
	usage?: {
		inputTokens: number;
		outputTokens: number;
		cacheCreationInputTokens?: number;
		cacheReadInputTokens?: number;
	};
}

export type StreamEvent =
	| { type: "sessionIdUpdated"; tempId: string; realId: string }
	| { type: "messageStart"; messageId: string }
	| { type: "contentDelta"; messageId: string; delta: string }
	| { type: "messageComplete"; messageId: string; content: string }
	| { type: "error"; error: string };

// Query Keys
export const sessionKeys = {
	all: ["sessions"] as const,
	project: (projectPath: string) => [...sessionKeys.all, projectPath] as const,
	detail: (sessionId: string) => [...sessionKeys.all, "detail", sessionId] as const,
	messages: (sessionId: string) =>
		[...sessionKeys.all, "messages", sessionId] as const,
};

// Check Claude Installation
export function useCheckClaudeInstalled() {
	return useQuery({
		queryKey: ["claude-installed"],
		queryFn: async () => {
			return await invoke<boolean>("session_check_claude_installed");
		},
		staleTime: 5 * 60 * 1000, // 5 minutes
	});
}

// List Sessions for Project
export function useSessions(projectPath: string) {
	return useQuery({
		queryKey: sessionKeys.project(projectPath),
		queryFn: async () => {
			return await invoke<Session[]>("session_list", { projectPath });
		},
		enabled: !!projectPath,
	});
}

// Get Session Detail
export function useSession(projectPath: string, sessionId: string) {
	return useQuery({
		queryKey: sessionKeys.detail(sessionId),
		queryFn: async () => {
			return await invoke<Session>("session_get", {
				projectPath,
				sessionId,
			});
		},
		enabled: !!projectPath && !!sessionId,
	});
}

// Get Session Messages
export function useSessionMessages(projectPath: string, sessionId: string) {
	return useQuery({
		queryKey: sessionKeys.messages(sessionId),
		queryFn: async () => {
			return await invoke<SessionMessage[]>("session_get_messages", {
				projectPath,
				sessionId,
			});
		},
		enabled: !!projectPath && !!sessionId,
	});
}

// Resume Session
export function useResumeSession() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: async ({
			sessionId,
			message,
			projectPath,
		}: {
			sessionId: string;
			message: string;
			projectPath: string;
		}) => {
			await invoke("session_resume", {
				sessionId,
				message,
				projectPath,
			});
		},
		onSuccess: (_, variables) => {
			// Invalidate session detail and messages
			queryClient.invalidateQueries({
				queryKey: sessionKeys.detail(variables.sessionId),
			});
			queryClient.invalidateQueries({
				queryKey: sessionKeys.messages(variables.sessionId),
			});
		},
	});
}

// Cancel Session
export function useCancelSession() {
	return useMutation({
		mutationFn: async (sessionId: string) => {
			await invoke("session_cancel", { sessionId });
		},
	});
}

// Delete Session
export function useDeleteSession() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: async ({
			sessionId,
			projectPath,
		}: {
			sessionId: string;
			projectPath: string;
		}) => {
			await invoke("session_delete", { sessionId, projectPath });
		},
		onSuccess: (_, variables) => {
			// Invalidate sessions list
			queryClient.invalidateQueries({
				queryKey: sessionKeys.project(variables.projectPath),
			});
		},
	});
}

// Create New Session
export function useCreateSession() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: async ({
			message,
			projectPath,
		}: {
			message: string;
			projectPath: string;
		}) => {
			const sessionId = await invoke<string>("session_create", {
				message,
				projectPath,
			});
			return sessionId;
		},
		onSuccess: (_, variables) => {
			// Invalidate sessions list to show new session
			queryClient.invalidateQueries({
				queryKey: sessionKeys.project(variables.projectPath),
			});
		},
	});
}

// Stream Events Listener
export function useSessionStream(
	sessionId: string,
	onEvent: (event: StreamEvent) => void
) {
	return useQuery({
		queryKey: ["session-stream", sessionId],
		queryFn: async () => {
			const unlisten = await listen<StreamEvent>(
				`session-stream:${sessionId}`,
				(event) => {
					onEvent(event.payload);
				}
			);
			return unlisten;
		},
		enabled: !!sessionId,
		staleTime: Infinity,
		gcTime: 0,
	});
}

// Helper to extract text content from message
export function getMessageText(message: SessionMessage): string {
	if (!message.message) {
		return "";
	}

	const content = message.message.content;

	if (typeof content === "string") {
		return content;
	}

	if (Array.isArray(content)) {
		return content
			.map((block: any) => {
				if (block.type === "text" && block.text) {
					return block.text;
				}
				return "";
			})
			.filter(Boolean)
			.join("\n");
	}

	return "";
}
