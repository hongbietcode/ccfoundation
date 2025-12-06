import { useState } from "react";
import { SessionList, SessionDetail } from "@/components/sessions";
import {
	ResizableHandle,
	ResizablePanel,
	ResizablePanelGroup,
} from "@/components/ui/resizable";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";

export function SessionsPage() {
	const [selectedSessionId, setSelectedSessionId] = useState<
		string | undefined
	>();

	// Keyboard shortcuts
	useKeyboardShortcuts([
		{
			key: "f",
			metaKey: true,
			description: "Focus search",
			action: () => {
				// Find and focus the search input
				const searchInput = document.querySelector('input[placeholder*="Search"]') as HTMLInputElement;
				if (searchInput) {
					searchInput.focus();
					searchInput.select();
				}
			},
		},
	]);

	return (
		<div className="h-full w-full">
			<ResizablePanelGroup direction="horizontal">
				<ResizablePanel defaultSize={35} minSize={25} maxSize={50}>
					<SessionList
						selectedSessionId={selectedSessionId}
						onSelectSession={setSelectedSessionId}
					/>
				</ResizablePanel>

				<ResizableHandle withHandle />

				<ResizablePanel defaultSize={65} minSize={50}>
					{selectedSessionId ? (
						<SessionDetail sessionId={selectedSessionId} />
					) : (
						<div className="flex items-center justify-center h-full text-muted-foreground">
							<div className="text-center space-y-2">
								<p>Select a session to view details</p>
								<p className="text-xs text-muted-foreground/60">
									Tip: Press Cmd+F to search
								</p>
							</div>
						</div>
					)}
				</ResizablePanel>
			</ResizablePanelGroup>
		</div>
	);
}
