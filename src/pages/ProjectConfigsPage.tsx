import { ask, open } from "@tauri-apps/plugin-dialog";
import { Search, FolderPlus } from "lucide-react";
import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import {
	useActivateProjectConfig,
	useActiveContext,
	useInitProjectClaudeDir,
	useClaudeProjects,
	useImportProjectLocalSettings,
	useProjectRegistry,
	useUpdateProjectRegistry,
	useDeleteProject,
} from "../lib/query";
import {
	mergeProjectsWithConfigs,
	searchProjects,
	filterProjectsByStatus,
	sortProjects,
	type ProjectStatus,
} from "../lib/project-utils";
import { ProjectCard } from "../components/project-configs/ProjectCard";

export function ProjectConfigsPage() {
	const { t } = useTranslation();
	const navigate = useNavigate();

	// Queries
	const { data: claudeProjects, isLoading: loadingProjects } = useClaudeProjects();
	const { data: projectRegistry, isLoading: loadingRegistry } = useProjectRegistry();
	const { data: activeContext } = useActiveContext();

	// Mutations
	const activateProjectConfig = useActivateProjectConfig();
	const initProjectClaudeDir = useInitProjectClaudeDir();
	const importProjectLocalSettings = useImportProjectLocalSettings();
	const updateProjectRegistry = useUpdateProjectRegistry();
	const deleteProject = useDeleteProject();

	// UI State
	const [searchQuery, setSearchQuery] = useState("");
	const [statusFilter, setStatusFilter] = useState<ProjectStatus | "all">("all");
	const [sortBy, setSortBy] = useState<"name" | "path" | "lastUsed">("name");

	// Merge and filter projects
	const enrichedProjects = useMemo(() => {
		if (!claudeProjects) return [];

		const registry = projectRegistry || [];
		let projects = mergeProjectsWithConfigs(claudeProjects, registry);
		projects = searchProjects(projects, searchQuery);
		projects = filterProjectsByStatus(projects, statusFilter);
		projects = sortProjects(projects, sortBy);

		return projects;
	}, [claudeProjects, projectRegistry, searchQuery, statusFilter, sortBy]);

	// Handlers
	const handleDelete = async (projectPath: string, title: string) => {
		const confirmed = await ask(
			t("configEditor.deleteConfirm", { name: title }),
			{ title: t("configEditor.deleteTitle"), kind: "warning" },
		);

		if (confirmed) {
			deleteProject.mutate(projectPath);
		}
	};

	const handleActivate = (projectPath: string) => {
		activateProjectConfig.mutate(projectPath);
	};

	const handleCreate = (projectPath: string) => {
		// Initialize .claude/ directory structure
		initProjectClaudeDir.mutate(projectPath, {
			onSuccess: () => {
				// Add to registry
				const projectName = projectPath.split("/").pop() || "Project";
				updateProjectRegistry.mutate({
					projectPath,
					title: projectName,
					inheritFromGlobal: false,
					parentGlobalConfigId: null,
				});
			},
		});
	};

	const handleImport = (projectPath: string) => {
		importProjectLocalSettings.mutate(projectPath);
	};

	const handleEdit = (projectPath: string) => {
		navigate(`/project-configs/${encodeURIComponent(projectPath)}`);
	};

	const handleAddProject = async () => {
		const selected = await open({
			directory: true,
			multiple: false,
			title: t("projectSelector.addProject"),
		});

		if (selected) {
			// Project will automatically appear in list after Claude Code tracks it
			// For now, we could show a message or trigger a refresh
			console.log("Selected project:", selected);
		}
	};

	const isLoading = loadingProjects || loadingRegistry;

	if (isLoading) {
		return (
			<div className="">
				<div
					className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
					data-tauri-drag-region
				>
					<div data-tauri-drag-region>
						<h3 className="font-bold" data-tauri-drag-region>
							{t("projectSelector.title")}
						</h3>
						<p className="text-sm text-muted-foreground" data-tauri-drag-region>
							{t("loading")}
						</p>
					</div>
				</div>
			</div>
		);
	}

	const hasProjects = enrichedProjects.length > 0;
	const hasNoClaudeProjects = !claudeProjects || claudeProjects.length === 0;

	return (
		<div className="">
			{/* Header */}
			<div
				className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
				data-tauri-drag-region
			>
				<div data-tauri-drag-region>
					<h3 className="font-bold" data-tauri-drag-region>
						{t("projectSelector.title")}
					</h3>
					<p className="text-sm text-muted-foreground" data-tauri-drag-region>
						{t("projectSelector.description")}
					</p>
				</div>
			</div>

			<div className="p-4 space-y-4">
				{/* No Projects Empty State */}
				{hasNoClaudeProjects ? (
					<Alert>
						<AlertDescription>
							{t("projectSelector.noProjects")}
						</AlertDescription>
					</Alert>
				) : (
					<>
						{/* Search and Filters */}
						<div className="flex flex-col sm:flex-row gap-2">
							{/* Search */}
							<div className="relative flex-1">
								<Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
								<Input
									placeholder={t("projectSelector.searchPlaceholder")}
									value={searchQuery}
									onChange={(e) => setSearchQuery(e.target.value)}
									className="pl-9"
								/>
							</div>

							{/* Status Filter */}
							<Select
								value={statusFilter}
								onValueChange={(value) =>
									setStatusFilter(value as ProjectStatus | "all")
								}
							>
								<SelectTrigger className="w-[180px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="all">
										{t("projectSelector.filterAll")}
									</SelectItem>
									<SelectItem value="has_config">
										{t("projectSelector.filterHasConfig")}
									</SelectItem>
									<SelectItem value="none">
										{t("projectSelector.filterNoConfig")}
									</SelectItem>
								</SelectContent>
							</Select>

							{/* Sort */}
							<Select
								value={sortBy}
								onValueChange={(value) =>
									setSortBy(value as "name" | "path" | "lastUsed")
								}
							>
								<SelectTrigger className="w-[160px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="name">
										{t("projectSelector.sortByName")}
									</SelectItem>
									<SelectItem value="path">
										{t("projectSelector.sortByPath")}
									</SelectItem>
									<SelectItem value="lastUsed">
										{t("projectSelector.sortByLastUsed")}
									</SelectItem>
								</SelectContent>
							</Select>

							{/* Add Project Button */}
							<Button
								variant="outline"
								size="default"
								onClick={handleAddProject}
								className="shrink-0"
							>
								<FolderPlus className="h-4 w-4 mr-2" />
								{t("projectSelector.addProject")}
							</Button>
						</div>

						{/* No Search Results */}
						{!hasProjects && searchQuery && (
							<Alert>
								<AlertDescription>
									{t("projectSelector.noSearchResults")}
								</AlertDescription>
							</Alert>
						)}

						{/* Project Grid */}
						{hasProjects && (
							<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
								{enrichedProjects.map((project) => {
									const isActive =
										activeContext?.type === "project" &&
										activeContext?.projectPath === project.path;

									return (
										<ProjectCard
											key={project.path}
											project={project}
											isActive={isActive}
											onEdit={handleEdit}
											onDelete={handleDelete}
											onActivate={handleActivate}
											onCreate={handleCreate}
											onImport={handleImport}
										/>
									);
								})}
							</div>
						)}
					</>
				)}
			</div>
		</div>
	);
}
