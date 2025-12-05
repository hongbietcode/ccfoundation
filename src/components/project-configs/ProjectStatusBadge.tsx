import { CheckCircle2, FileIcon, CircleIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ProjectStatus } from "@/lib/project-utils";

interface ProjectStatusBadgeProps {
	status: ProjectStatus;
}

export function ProjectStatusBadge({ status }: ProjectStatusBadgeProps) {
	const { t } = useTranslation();

	const statusConfig = {
		has_config: {
			icon: CheckCircle2,
			label: t("projectSelector.status.hasConfig"),
			className: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
		},
		has_local: {
			icon: FileIcon,
			label: t("projectSelector.status.hasLocal"),
			className: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
		},
		none: {
			icon: CircleIcon,
			label: t("projectSelector.status.noConfig"),
			className: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400",
		},
	};

	const config = statusConfig[status];
	const Icon = config.icon;

	return (
		<span
			className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium ${config.className}`}
		>
			<Icon className="h-3 w-3" />
			{config.label}
		</span>
	);
}
