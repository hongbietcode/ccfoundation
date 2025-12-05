import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { yamlFrontmatter } from "@codemirror/lang-yaml";
import CodeMirror, { EditorView } from "@uiw/react-codemirror";
import { SaveIcon } from "lucide-react";
import { Suspense, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useConfigContext } from "@/contexts/ConfigContext";
import {
	useClaudeMemory,
	useProjectMemory,
	useWriteClaudeMemory,
	useWriteProjectMemory,
} from "@/lib/query";
import { useCodeMirrorTheme } from "@/lib/use-codemirror-theme";

function MemoryPageHeader({
	onSave,
	saving,
	memoryPath,
}: {
	onSave: () => void;
	saving: boolean;
	memoryPath?: string;
}) {
	const { t } = useTranslation();

	return (
		<div
			className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
			data-tauri-drag-region
		>
			<div data-tauri-drag-region>
				<h3 className="font-bold" data-tauri-drag-region>
					{t("memory.title")}
				</h3>
				<p className="text-sm text-muted-foreground">
					{memoryPath || t("memory.description")}
				</p>
			</div>
			<Button
				onClick={onSave}
				disabled={saving}
				variant="default"
				size="sm"
				className="flex items-center gap-2"
			>
				<SaveIcon className="w-4 h-4" />
				{saving ? t("memory.saving") : t("memory.save")}
			</Button>
		</div>
	);
}

function MemoryPageSkeleton() {
	return (
		<div className="flex flex-col h-screen">
			<div
				className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
				data-tauri-drag-region
			>
				<div data-tauri-drag-region>
					<Skeleton className="h-6 w-16 mb-2" />
					<Skeleton className="h-4 w-64" />
				</div>
				<Skeleton className="h-8 w-16" />
			</div>
			<div className="flex-1 p-4 overflow-hidden">
				<div className="rounded-lg overflow-hidden border h-full">
					<div className="h-full flex items-center justify-center">
						<div className="space-y-2 w-full max-w-2xl">
							<Skeleton className="h-4 w-full" />
							<Skeleton className="h-4 w-3/4" />
							<Skeleton className="h-4 w-1/2" />
							<Skeleton className="h-4 w-full" />
							<Skeleton className="h-4 w-2/3" />
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}

function MemoryPageContent() {
	const { isProject, projectPath } = useConfigContext();

	// Always call both hooks (React rules), choose which data to use
	const { data: globalMemory } = useClaudeMemory();
	const { data: projectMemory } = useProjectMemory(projectPath || "");
	const memoryData = isProject && projectPath ? projectMemory : globalMemory;

	const { mutate: saveGlobalMemory, isPending: savingGlobal } = useWriteClaudeMemory();
	const { mutate: saveProjectMemory, isPending: savingProject } = useWriteProjectMemory();
	const saving = isProject && projectPath ? savingProject : savingGlobal;

	const [content, setContent] = useState<string>("");
	const codeMirrorTheme = useCodeMirrorTheme();

	// Update local content when memory data loads
	useEffect(() => {
		if (memoryData?.content) {
			setContent(memoryData.content);
		}
	}, [memoryData]);

	const handleSave = () => {
		if (isProject && projectPath) {
			saveProjectMemory({ projectPath, content });
		} else {
			saveGlobalMemory(content);
		}
	};

	const handleKeyDown = (e: KeyboardEvent) => {
		// Cmd+S or Ctrl+S to save
		if ((e.metaKey || e.ctrlKey) && e.key === "s") {
			e.preventDefault();
			handleSave();
		}
	};

	// Add keyboard event listener
	useEffect(() => {
		window.addEventListener("keydown", handleKeyDown);
		return () => {
			window.removeEventListener("keydown", handleKeyDown);
		};
	}, [content]);

	return (
		<div className="flex flex-col h-screen">
			<MemoryPageHeader onSave={handleSave} saving={saving} memoryPath={memoryData?.path} />

			<div className="flex-1 p-4 overflow-hidden">
				<div className="rounded-lg overflow-hidden border h-full">
					<CodeMirror
						value={content}
						height="100%"
						extensions={[
							yamlFrontmatter({
								content: markdown({
									base: markdownLanguage,
								}),
							}),
							EditorView.lineWrapping,
						]}
						placeholder={memoryData?.path || "~/.claude/CLAUDE.md"}
						onChange={(value) => setContent(value)}
						theme={codeMirrorTheme}
						basicSetup={{
							lineNumbers: false,
							highlightActiveLineGutter: true,
							foldGutter: false,
							dropCursor: false,
							allowMultipleSelections: false,
							indentOnInput: true,
							bracketMatching: true,
							closeBrackets: true,
							autocompletion: true,
							highlightActiveLine: true,
							highlightSelectionMatches: true,
							searchKeymap: false,
						}}
						className="h-full"
						style={{ width: "100%" }}
					/>
				</div>
			</div>
		</div>
	);
}

export function MemoryPage() {
	return (
		<Suspense fallback={<MemoryPageSkeleton />}>
			<MemoryPageContent />
		</Suspense>
	);
}
