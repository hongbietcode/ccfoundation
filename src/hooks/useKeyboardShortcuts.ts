import { useEffect } from "react";

export interface KeyboardShortcut {
	key: string;
	ctrlKey?: boolean;
	metaKey?: boolean;
	shiftKey?: boolean;
	action: () => void;
	description: string;
}

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[], enabled = true) {
	useEffect(() => {
		if (!enabled) return;

		const handleKeyDown = (event: KeyboardEvent) => {
			for (const shortcut of shortcuts) {
				const keyMatches = event.key.toLowerCase() === shortcut.key.toLowerCase();
				const ctrlMatches = shortcut.ctrlKey === undefined || shortcut.ctrlKey === event.ctrlKey;
				const metaMatches = shortcut.metaKey === undefined || shortcut.metaKey === event.metaKey;
				const shiftMatches = shortcut.shiftKey === undefined || shortcut.shiftKey === event.shiftKey;

				if (keyMatches && ctrlMatches && metaMatches && shiftMatches) {
					// Check if we're in an input/textarea to avoid conflicts
					const target = event.target as HTMLElement;
					const isInput = target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable;

					// Only prevent default if it's not a search shortcut or we're not in an input
					if (shortcut.key === "f" || !isInput) {
						event.preventDefault();
						shortcut.action();
					}
				}
			}
		};

		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [shortcuts, enabled]);
}
