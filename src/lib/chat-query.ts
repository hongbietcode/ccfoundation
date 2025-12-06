import { useMutation, useQuery, useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import i18n from "../i18n";
import { useEffect, useRef } from "react";

// Types
export interface ChatSession {
	id: string;
	projectPath: string;
	title: string;
	createdAt: number;
	updatedAt: number;
	messageCount: number;
}

export interface ChatMessage {
	id: string;
	sessionId: string;
	role: "user" | "assistant" | "system" | "tool";
	content: string;
	timestamp: number;
	toolUse?: {
		toolName: string;
		input: unknown;
		output?: string;
	};
	metadata?: unknown;
}

export interface ChatConfig {
	model: string;
	permissionMode: "default" | "acceptEdits" | "bypassPermissions" | "plan";
	maxTokens?: number;
	temperature?: number;
}

export type StreamEvent =
	| {
			type: "messageStart";
			messageId: string;
	  }
	| {
			type: "contentDelta";
			messageId: string;
			delta: string;
	  }
	| {
			type: "messageComplete";
			messageId: string;
			content: string;
	  }
	| {
			type: "toolUse";
			messageId: string;
			toolName: string;
			input: unknown;
	  }
	| {
			type: "toolResult";
			messageId: string;
			toolName: string;
			output: string;
	  }
	| {
			type: "error";
			error: string;
	  };

// Check if Claude CLI is installed
export const useCheckClaudeInstalled = () => {
	return useQuery({
		queryKey: ["claude-cli-installed"],
		queryFn: () => invoke<boolean>("chat_check_claude_installed"),
		staleTime: 60000, // Cache for 1 minute
	});
};

// Create a new chat session
export const useCreateChatSession = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			projectPath,
			title,
		}: {
			projectPath: string;
			title?: string;
		}) =>
			invoke<ChatSession>("chat_create_session", {
				projectPath,
				title,
			}),
		onSuccess: (session) => {
			queryClient.invalidateQueries({
				queryKey: ["chat-sessions", session.projectPath],
			});
			toast.success(i18n.t("toast.chatSessionCreated", "Chat session created"));
		},
		onError: (error) => {
			const errorMessage = error instanceof Error ? error.message : String(error);
			toast.error(
				i18n.t("toast.chatSessionCreateFailed", {
					defaultValue: "Failed to create chat session: {{error}}",
					error: errorMessage,
				}),
			);
		},
	});
};

// Get all sessions for a project
export const useChatSessions = (projectPath: string) => {
	return useSuspenseQuery({
		queryKey: ["chat-sessions", projectPath],
		queryFn: () =>
			invoke<ChatSession[]>("chat_get_sessions", {
				projectPath,
			}),
	});
};

// Get messages for a session
export const useChatMessages = (sessionId: string) => {
	return useSuspenseQuery({
		queryKey: ["chat-messages", sessionId],
		queryFn: () =>
			invoke<ChatMessage[]>("chat_get_messages", {
				sessionId,
			}),
	});
};

// Delete a session
export const useDeleteChatSession = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: (sessionId: string) =>
			invoke<void>("chat_delete_session", {
				sessionId,
			}),
		onSuccess: (_, sessionId) => {
			// Invalidate sessions list
			queryClient.invalidateQueries({
				queryKey: ["chat-sessions"],
			});
			// Remove messages cache
			queryClient.removeQueries({
				queryKey: ["chat-messages", sessionId],
			});
			toast.success(i18n.t("toast.chatSessionDeleted", "Chat session deleted"));
		},
		onError: (error) => {
			const errorMessage = error instanceof Error ? error.message : String(error);
			toast.error(
				i18n.t("toast.chatSessionDeleteFailed", {
					defaultValue: "Failed to delete session: {{error}}",
					error: errorMessage,
				}),
			);
		},
	});
};

// Send a message
export const useSendChatMessage = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			sessionId,
			message,
			config,
		}: {
			sessionId: string;
			message: string;
			config?: ChatConfig;
		}) =>
			invoke<void>("chat_send_message", {
				sessionId,
				message,
				config,
			}),
		onSuccess: (_, { sessionId }) => {
			// Invalidate messages to trigger reload
			queryClient.invalidateQueries({
				queryKey: ["chat-messages", sessionId],
			});
			queryClient.invalidateQueries({
				queryKey: ["chat-sessions"],
			});
		},
		onError: (error) => {
			const errorMessage = error instanceof Error ? error.message : String(error);
			toast.error(
				i18n.t("toast.chatSendMessageFailed", {
					defaultValue: "Failed to send message: {{error}}",
					error: errorMessage,
				}),
			);
		},
	});
};

// Cancel streaming
export const useCancelChatStream = () => {
	return useMutation({
		mutationFn: (sessionId: string) =>
			invoke<void>("chat_cancel_stream", {
				sessionId,
			}),
		onError: (error) => {
			const errorMessage = error instanceof Error ? error.message : String(error);
			toast.error(
				i18n.t("toast.chatCancelFailed", {
					defaultValue: "Failed to cancel stream: {{error}}",
					error: errorMessage,
				}),
			);
		},
	});
};

// Save assistant message
export const useSaveAssistantMessage = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ sessionId, content }: { sessionId: string; content: string }) =>
			invoke<void>("chat_save_assistant_message", {
				sessionId,
				content,
			}),
		onSuccess: (_, { sessionId }) => {
			queryClient.invalidateQueries({
				queryKey: ["chat-messages", sessionId],
			});
			queryClient.invalidateQueries({
				queryKey: ["chat-sessions"],
			});
		},
	});
};

// Update session title
export const useUpdateSessionTitle = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ sessionId, title }: { sessionId: string; title: string }) =>
			invoke<void>("chat_update_session_title", {
				sessionId,
				title,
			}),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: ["chat-sessions"],
			});
		},
		onError: (error) => {
			const errorMessage = error instanceof Error ? error.message : String(error);
			toast.error(
				i18n.t("toast.chatUpdateTitleFailed", {
					defaultValue: "Failed to update title: {{error}}",
					error: errorMessage,
				}),
			);
		},
	});
};

// Hook to listen for stream events
export const useChatStreamListener = (
	sessionId: string,
	onEvent: (event: StreamEvent) => void,
) => {
	const unlistenRef = useRef<(() => void) | null>(null);

	useEffect(() => {
		const eventName = `chat-stream:${sessionId}`;

		const setupListener = async () => {
			if (unlistenRef.current) {
				unlistenRef.current();
			}

			const unlisten = await listen<StreamEvent>(eventName, (event) => {
				onEvent(event.payload);
			});

			unlistenRef.current = unlisten;
		};

		setupListener();

		return () => {
			if (unlistenRef.current) {
				unlistenRef.current();
				unlistenRef.current = null;
			}
		};
	}, [sessionId, onEvent]);
};
