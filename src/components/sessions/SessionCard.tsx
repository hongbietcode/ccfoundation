import { type Session } from "@/lib/sessions-query";
import { formatDistanceToNow } from "date-fns";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface SessionCardProps {
	session: Session;
	isSelected?: boolean;
	onClick?: () => void;
}

export function SessionCard({ session, isSelected, onClick }: SessionCardProps) {
	const createdDate = new Date(session.createdAt);
	const formattedDate = formatDistanceToNow(createdDate, { addSuffix: true });

	return (
		<Card
			className={`p-4 cursor-pointer transition-colors hover:bg-accent ${
				isSelected ? "bg-accent border-primary" : ""
			}`}
			onClick={onClick}
		>
			<div className="flex items-start justify-between gap-2">
				<div className="flex-1 min-w-0">
					<h3 className="font-medium truncate">{session.title}</h3>
					<div className="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
						<span>{formattedDate}</span>
						<span>•</span>
						<span>{session.messageCount} messages</span>
						{session.model && (
							<>
								<span>•</span>
								<Badge variant="outline" className="text-xs">
									{session.model}
								</Badge>
							</>
						)}
					</div>
				</div>
			</div>
		</Card>
	);
}
