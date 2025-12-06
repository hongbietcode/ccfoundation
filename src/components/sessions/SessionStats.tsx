import { useMemo } from "react";
import type { Session, SessionMessage } from "@/lib/sessions-query";
import { Card } from "@/components/ui/card";
import { Clock, Coins, MessageSquare, Zap } from "lucide-react";
import { formatDistanceToNow, differenceInMinutes } from "date-fns";

interface SessionStatsProps {
	session: Session;
	messages: SessionMessage[];
}

// Pricing per million tokens (approximate, based on Anthropic pricing)
const MODEL_PRICING = {
	"claude-opus-4": { input: 15, output: 75 },
	"claude-sonnet-4-5-20250929": { input: 3, output: 15 },
	"claude-haiku-4": { input: 0.8, output: 4 },
	"claude-sonnet-3.5": { input: 3, output: 15 },
	opus: { input: 15, output: 75 },
	sonnet: { input: 3, output: 15 },
	haiku: { input: 0.8, output: 4 },
	default: { input: 3, output: 15 },
};

function getModelPricing(model?: string) {
	if (!model) return MODEL_PRICING.default;

	const lowerModel = model.toLowerCase();
	for (const [key, pricing] of Object.entries(MODEL_PRICING)) {
		if (lowerModel.includes(key)) {
			return pricing;
		}
	}
	return MODEL_PRICING.default;
}

export function SessionStats({ session, messages }: SessionStatsProps) {
	const stats = useMemo(() => {
		// Filter out "other" type messages
		const validMessages = messages.filter((m) => m.msgType !== "other");

		let totalInputTokens = 0;
		let totalOutputTokens = 0;
		let totalCacheCreationTokens = 0;
		let totalCacheReadTokens = 0;

		validMessages.forEach((msg) => {
			if (msg.usage) {
				totalInputTokens += msg.usage.inputTokens || 0;
				totalOutputTokens += msg.usage.outputTokens || 0;
				totalCacheCreationTokens += msg.usage.cacheCreationInputTokens || 0;
				totalCacheReadTokens += msg.usage.cacheReadInputTokens || 0;
			}
		});

		const pricing = getModelPricing(session.model);

		// Calculate cost (pricing is per million tokens)
		const inputCost = (totalInputTokens / 1_000_000) * pricing.input;
		const outputCost = (totalOutputTokens / 1_000_000) * pricing.output;
		const totalCost = inputCost + outputCost;

		// Calculate duration
		const createdAt = new Date(session.createdAt);
		const updatedAt = new Date(session.updatedAt);
		const durationMinutes = differenceInMinutes(updatedAt, createdAt);

		return {
			totalInputTokens,
			totalOutputTokens,
			totalCacheCreationTokens,
			totalCacheReadTokens,
			totalTokens: totalInputTokens + totalOutputTokens,
			estimatedCost: totalCost,
			durationMinutes,
			messageCount: validMessages.length,
		};
	}, [session, messages]);

	return (
		<div className="grid grid-cols-2 md:grid-cols-4 gap-3">
			<Card className="p-3">
				<div className="flex items-center gap-2 mb-2">
					<Zap className="h-4 w-4 text-muted-foreground" />
					<span className="text-xs font-medium text-muted-foreground">Tokens</span>
				</div>
				<div className="space-y-1">
					<div className="text-lg font-semibold">{stats.totalTokens.toLocaleString()}</div>
					<div className="text-xs text-muted-foreground space-x-2">
						<span>↑ {stats.totalInputTokens.toLocaleString()}</span>
						<span>↓ {stats.totalOutputTokens.toLocaleString()}</span>
					</div>
					{(stats.totalCacheReadTokens > 0 || stats.totalCacheCreationTokens > 0) && (
						<div className="text-xs text-blue-500">
							<span>Cache: {stats.totalCacheReadTokens.toLocaleString()} read</span>
						</div>
					)}
				</div>
			</Card>

			<Card className="p-3">
				<div className="flex items-center gap-2 mb-2">
					<Coins className="h-4 w-4 text-muted-foreground" />
					<span className="text-xs font-medium text-muted-foreground">Est. Cost</span>
				</div>
				<div className="space-y-1">
					<div className="text-lg font-semibold">${stats.estimatedCost.toFixed(4)}</div>
					<div className="text-xs text-muted-foreground">{session.model || "Default model"}</div>
				</div>
			</Card>

			<Card className="p-3">
				<div className="flex items-center gap-2 mb-2">
					<Clock className="h-4 w-4 text-muted-foreground" />
					<span className="text-xs font-medium text-muted-foreground">Duration</span>
				</div>
				<div className="space-y-1">
					<div className="text-lg font-semibold">
						{stats.durationMinutes < 60
							? `${stats.durationMinutes}m`
							: `${Math.floor(stats.durationMinutes / 60)}h ${stats.durationMinutes % 60}m`}
					</div>
					<div className="text-xs text-muted-foreground">{formatDistanceToNow(new Date(session.createdAt), { addSuffix: true })}</div>
				</div>
			</Card>

			<Card className="p-3">
				<div className="flex items-center gap-2 mb-2">
					<MessageSquare className="h-4 w-4 text-muted-foreground" />
					<span className="text-xs font-medium text-muted-foreground">Messages</span>
				</div>
				<div className="space-y-1">
					<div className="text-lg font-semibold">{stats.messageCount}</div>
					<div className="text-xs text-muted-foreground">
						{messages.filter((m) => m.msgType === "user").length} user / {messages.filter((m) => m.msgType === "assistant").length}{" "}
						assistant
					</div>
				</div>
			</Card>
		</div>
	);
}
