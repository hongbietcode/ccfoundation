import { FolderIcon, Star, Download, PlusCircle, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { EnrichedProject } from "@/lib/project-utils";
import { ProjectStatusBadge } from "./ProjectStatusBadge";
import { ProjectActionsMenu } from "./ProjectActionsMenu";

interface ProjectCardProps {
	project: EnrichedProject;
	isActive?: boolean;
	onEdit?: (path: string) => void;
	onDelete?: (path: string, title: string) => void;
	onActivate?: (path: string) => void;
	onCreate?: (path: string) => void;
	onImport?: (path: string) => void;
}

export function ProjectCard({
	project,
	isActive = false,
	onEdit,
	onDelete,
	onActivate,
	onCreate,
	onImport,
}: ProjectCardProps) {
	const { t } = useTranslation();

	const handlePrimaryAction = () => {
		switch (project.status) {
			case "has_config":
				onEdit?.(project.path);
				break;
			case "has_local":
				onImport?.(project.path);
				break;
			case "none":
				onCreate?.(project.path);
				break;
		}
	};

	const getPrimaryButtonConfig = () => {
		switch (project.status) {
			case "has_config":
				return {
					label: t("projectSelector.actions.edit"),
					icon: FolderIcon,
					variant: "outline" as const,
				};
			case "has_local":
				return {
					label: t("projectSelector.actions.import"),
					icon: Download,
					variant: "default" as const,
				};
			case "none":
				return {
					label: t("projectSelector.actions.create"),
					icon: PlusCircle,
					variant: "default" as const,
				};
		}
	};

	const primaryButton = getPrimaryButtonConfig();
	const PrimaryIcon = primaryButton.icon;

	return (
		<Card
			className={cn(
				"p-4 flex flex-col space-y-3 transition-colors hover:border-primary/50",
				isActive && "border-primary border-2 bg-primary/5",
			)}
		>
			{/* Header */}
			<div className="flex items-start justify-between gap-2">
				<div className="flex-1 min-w-0">
					<h4 className="font-medium flex items-center gap-2">
						<FolderIcon className="h-4 w-4 shrink-0 text-muted-foreground" />
						<span className="truncate">{project.name}</span>
						{isActive && (
							<Star className="h-4 w-4 fill-primary text-primary shrink-0" />
						)}
					</h4>
					<p
						className="text-xs text-muted-foreground truncate mt-1"
						title={project.path}
					>
						{project.path}
					</p>
				</div>

				{/* Actions - Delete button and Menu */}
				<div className="flex items-center gap-1">
					{/* Delete Button */}
					<Button
						variant="ghost"
						size="sm"
						className="h-8 w-8 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
						onClick={() =>
							onDelete?.(project.path, project.registryEntry?.title || project.name)
						}
					>
						<Trash2 className="h-4 w-4" />
						<span className="sr-only">{t("projectSelector.actions.delete")}</span>
					</Button>

					{/* Actions Menu - show for projects with config */}
					{project.status === "has_config" && (
						<ProjectActionsMenu
							onEdit={() => onEdit?.(project.path)}
							onDelete={() =>
								onDelete?.(project.path, project.registryEntry?.title || project.name)
							}
							onActivate={() => onActivate?.(project.path)}
							showEdit={true}
							showDelete={false}
							showActivate={!isActive}
						/>
					)}
				</div>
			</div>

			{/* Status Badge */}
			<div className="flex items-center gap-2">
				<ProjectStatusBadge status={project.status} />
				{project.registryEntry?.inheritFromGlobal && (
					<span className="text-xs text-muted-foreground">
						• {t("projectSelector.status.inheriting")}
					</span>
				)}
			</div>

			{/* Primary Action Button */}
			<Button
				size="sm"
				variant={primaryButton.variant}
				className="w-full"
				onClick={handlePrimaryAction}
			>
				<PrimaryIcon className="h-4 w-4 mr-2" />
				{primaryButton.label}
			</Button>
		</Card>
	);
}
