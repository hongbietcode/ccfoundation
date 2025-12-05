import { Folder, Globe, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { ask } from "@tauri-apps/plugin-dialog";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useClaudeProjects, useActiveContext, useDeleteProject } from "../lib/query";
import { isMacOS } from "../lib/utils";

export function ContextSelectorPage() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const { data: claudeProjects } = useClaudeProjects();
	const { data: activeContext } = useActiveContext();
	const deleteProject = useDeleteProject();

	const handleSelectGlobal = () => {
		navigate("/context/global");
	};

	const handleSelectProject = (projectPath: string) => {
		navigate(`/context/project/${encodeURIComponent(projectPath)}`);
	};

	const handleDeleteProject = async (
		e: React.MouseEvent,
		projectPath: string,
		projectName: string,
	) => {
		e.stopPropagation(); // Prevent card click

		const confirmed = await ask(
			t("configEditor.deleteConfirm", { name: projectName }),
			{ title: t("configEditor.deleteTitle"), kind: "warning" },
		);

		if (confirmed) {
			deleteProject.mutate(projectPath);
		}
	};

	return (
		<div className="min-h-screen bg-background">
			{/* macOS title bar space */}
			{isMacOS && (
				<div
					data-tauri-drag-region
					className="h-10"
					style={{
						WebkitUserSelect: "none",
						WebkitAppRegion: "drag",
					} as React.CSSProperties}
				/>
			)}

			<div className="container max-w-4xl mx-auto px-4 py-8">
				<div className="mb-8">
					<h1 className="text-3xl font-bold mb-2">
						{t("contextSelector.title", "Choose Configuration Context")}
					</h1>
					<p className="text-muted-foreground">
						{t(
							"contextSelector.description",
							"Select global configuration or a specific project",
						)}
					</p>
				</div>

				<div className="space-y-4">
					{/* Global Config Card */}
					<Card
						className={`p-6 cursor-pointer hover:border-primary transition-colors ${
							activeContext?.type === "global" ? "border-primary border-2" : ""
						}`}
						onClick={handleSelectGlobal}
					>
						<div className="flex items-center gap-4">
							<div className="p-3 rounded-full bg-primary/10">
								<Globe className="h-6 w-6 text-primary" />
							</div>
							<div className="flex-1">
								<h3 className="font-semibold text-lg">
									{t("contextSelector.global", "Global Configuration")}
								</h3>
								<p className="text-sm text-muted-foreground">
									{t(
										"contextSelector.globalDesc",
										"Default settings applied to all projects",
									)}
								</p>
							</div>
							{activeContext?.type === "global" && (
								<div className="text-xs bg-primary text-primary-foreground px-2 py-1 rounded">
									Active
								</div>
							)}
						</div>
					</Card>

					{/* Projects Section */}
					<div className="mt-8">
						<h2 className="text-xl font-semibold mb-4">
							{t("contextSelector.projects", "Project Configurations")}
						</h2>

						{!claudeProjects || claudeProjects.length === 0 ? (
							<Card className="p-6 text-center text-muted-foreground">
								{t(
									"contextSelector.noProjects",
									"No projects found in Claude Code",
								)}
							</Card>
						) : (
							<div className="space-y-2">
								{claudeProjects.map((project) => {
									const isActive =
										activeContext?.type === "project" &&
										activeContext?.projectPath === project.path;

									return (
										<Card
											key={project.path}
											className={`p-4 cursor-pointer hover:border-primary transition-colors ${
												isActive ? "border-primary border-2" : ""
											}`}
											onClick={() => handleSelectProject(project.path)}
										>
											<div className="flex items-center gap-3">
												<div className="p-2 rounded bg-muted">
													<Folder className="h-4 w-4" />
												</div>
												<div className="flex-1 min-w-0">
													<h4 className="font-medium truncate">
														{project.path.split("/").pop() || project.path}
													</h4>
													<p className="text-xs text-muted-foreground truncate">
														{project.path}
													</p>
												</div>
												{isActive && (
													<div className="text-xs bg-primary text-primary-foreground px-2 py-1 rounded">
														Active
													</div>
												)}
												<Button
													variant="ghost"
													size="sm"
													className="h-8 w-8 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
													onClick={(e) =>
														handleDeleteProject(
															e,
															project.path,
															project.path.split("/").pop() || project.path,
														)
													}
												>
													<Trash2 className="h-4 w-4" />
													<span className="sr-only">
														{t("projectSelector.actions.delete")}
													</span>
												</Button>
											</div>
										</Card>
									);
								})}
							</div>
						)}
					</div>
				</div>
			</div>
		</div>
	);
}
