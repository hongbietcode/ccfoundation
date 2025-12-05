import { MoreVertical, Edit, Trash2, Power } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface ProjectActionsMenuProps {
	onEdit?: () => void;
	onDelete?: () => void;
	onActivate?: () => void;
	showEdit?: boolean;
	showDelete?: boolean;
	showActivate?: boolean;
}

export function ProjectActionsMenu({
	onEdit,
	onDelete,
	onActivate,
	showEdit = true,
	showDelete = true,
	showActivate = true,
}: ProjectActionsMenuProps) {
	const { t } = useTranslation();

	const hasActions = showEdit || showDelete || showActivate;

	if (!hasActions) return null;

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button variant="ghost" size="sm" className="h-8 w-8 p-0">
					<MoreVertical className="h-4 w-4" />
					<span className="sr-only">Open menu</span>
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end">
				{showEdit && onEdit && (
					<DropdownMenuItem onClick={onEdit}>
						<Edit className="mr-2 h-4 w-4" />
						{t("projectSelector.actions.edit")}
					</DropdownMenuItem>
				)}
				{showActivate && onActivate && (
					<DropdownMenuItem onClick={onActivate}>
						<Power className="mr-2 h-4 w-4" />
						{t("projectSelector.actions.activate")}
					</DropdownMenuItem>
				)}
				{showDelete && onDelete && (
					<DropdownMenuItem onClick={onDelete} className="text-destructive">
						<Trash2 className="mr-2 h-4 w-4" />
						{t("projectSelector.actions.delete")}
					</DropdownMenuItem>
				)}
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
