import { type SessionMessage, getMessageText } from "@/lib/sessions-query";
import { formatDistanceToNow } from "date-fns";
import ReactMarkdown from "react-markdown";
import { Card } from "@/components/ui/card";

interface SessionMessagesProps {
	messages: SessionMessage[];
}

export function SessionMessages({ messages }: SessionMessagesProps) {
	// Filter out "other" type messages (queue operations, etc.)
	const validMessages = messages.filter((m) => m.msgType !== "other");

	if (validMessages.length === 0) {
		return (
			<div className="flex items-center justify-center h-full text-muted-foreground">
				No messages in this session
			</div>
		);
	}

	return (
		<div className="space-y-4">
			{validMessages.map((message) => {
				const messageText = getMessageText(message);
				const messageDate = new Date(message.timestamp);
				const formattedDate = formatDistanceToNow(messageDate, {
					addSuffix: true,
				});

				return (
					<Card
						key={message.uuid}
						className={`p-4 ${
							message.msgType === "user"
								? "bg-accent"
								: "bg-background border-primary/20"
						}`}
					>
						<div className="flex items-start gap-3">
							<div className="flex-1">
								<div className="flex items-center gap-2 mb-2">
									<span className="font-medium text-sm">
										{message.msgType === "user" ? "You" : "Claude"}
									</span>
									<span className="text-xs text-muted-foreground">
										{formattedDate}
									</span>
									{message.model && (
										<span className="text-xs text-muted-foreground">
											• {message.model}
										</span>
									)}
								</div>
								<div className="prose prose-sm dark:prose-invert max-w-none">
									<ReactMarkdown>{messageText}</ReactMarkdown>
								</div>
								{message.usage && (
									<div className="mt-2 text-xs text-muted-foreground">
										{message.usage.inputTokens} in • {message.usage.outputTokens}{" "}
										out
										{message.usage.cacheReadInputTokens && (
											<> • {message.usage.cacheReadInputTokens} cached</>
										)}
									</div>
								)}
							</div>
						</div>
					</Card>
				);
			})}
		</div>
	);
}
