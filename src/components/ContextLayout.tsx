import {
	ActivityIcon,
	BotIcon,
	BrainIcon,
	ChevronLeftIcon,
	CpuIcon,
	FileJsonIcon,
	TerminalIcon,
} from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import { NavLink, Outlet, useParams, useNavigate } from "react-router-dom";
import { cn, isMacOS } from "../lib/utils";
import { UpdateButton } from "./UpdateButton";
import { ScrollArea } from "./ui/scroll-area";
import { Button } from "./ui/button";
import { ConfigProvider } from "../contexts/ConfigContext";

export function ContextLayout() {
	const { t } = useTranslation();
	const { contextType, projectPath } = useParams<{
		contextType?: "global" | "project";
		projectPath?: string;
	}>();
	const navigate = useNavigate();

	// projectPath from useParams is already decoded by React Router
	const decodedProjectPath = projectPath || "";
	const projectName = decodedProjectPath.split("/").pop() || "";

	// Build basePath with proper encoding for URLs
	const basePath = projectPath
		? `/context/project/${encodeURIComponent(projectPath)}`
		: `/context/${contextType || "global"}`;

	// Build navLinks based on context
	const navLinks = [
		{
			to: `${basePath}`,
			icon: FileJsonIcon,
			label: t("navigation.configurations"),
			exact: true,
		},
		{
			to: `${basePath}/mcp`,
			icon: CpuIcon,
			label: t("navigation.mcp"),
		},
		{
			to: `${basePath}/agents`,
			icon: BotIcon,
			label: t("navigation.agents"),
		},
		{
			to: `${basePath}/commands`,
			icon: TerminalIcon,
			label: t("navigation.commands"),
		},
		{
			to: `${basePath}/memory`,
			icon: BrainIcon,
			label: t("navigation.memory"),
		},
		// Usage only for global context
		...(!projectPath ? [{
			to: `${basePath}/usage`,
			icon: ActivityIcon,
			label: t("navigation.usage"),
		}] : []),
	];

	const handleBackToSelector = () => {
		navigate("/");
	};

	return (
		<div className="min-h-screen bg-background flex flex-col">
			{/* macOS title bar */}
			{isMacOS && (
				<div
					data-tauri-drag-region
					className=""
					style={
						{
							WebkitUserSelect: "none",
							WebkitAppRegion: "drag",
						} as React.CSSProperties
					}
				></div>
			)}

			<div className="flex flex-1 overflow-hidden">
				<nav
					className="w-[200px] bg-background border-r flex flex-col"
					data-tauri-drag-region
				>
					{isMacOS && (
						<div
							data-tauri-drag-region
							className="h-10"
							style={
								{
									WebkitUserSelect: "none",
									WebkitAppRegion: "drag",
								} as React.CSSProperties
							}
						></div>
					)}
					<div
						className="flex flex-col flex-1 justify-between"
						data-tauri-drag-region
					>
						{/* Back button + Context info */}
						<div>
							<div className="px-3 pt-3 pb-2 border-b">
								<Button
									variant="ghost"
									size="sm"
									onClick={handleBackToSelector}
									className="w-full justify-start"
								>
									<ChevronLeftIcon size={14} />
									<span className="text-xs">Back to Selector</span>
								</Button>
								<div className="mt-2 px-2">
									<div className="text-xs font-medium text-muted-foreground">
										{projectPath ? "Project" : "Global Config"}
									</div>
									{projectPath && (
										<div className="text-xs truncate font-semibold mt-0.5">
											{projectName}
										</div>
									)}
								</div>
							</div>

							<ul className="px-3 pt-3 space-y-2">
								{navLinks.map((link) => (
									<li key={link.to}>
										<NavLink
											to={link.to}
											end={link.exact}
											className={({ isActive }) =>
												cn(
													"flex items-center gap-2 px-3 py-2 rounded-xl cursor-default select-none",
													{
														"bg-primary text-primary-foreground": isActive,
														"hover:bg-accent hover:text-accent-foreground":
															!isActive,
													},
												)
											}
										>
											<link.icon size={14} />
											{link.label}
										</NavLink>
									</li>
								))}
							</ul>
						</div>

						<div className="space-y-2">
							<UpdateButton />
						</div>
					</div>
				</nav>

				<ScrollArea className="flex-1 h-screen [&>div>div]:!block">
					<main className="" data-tauri-drag-region>
						<ConfigProvider>
							<Outlet />
						</ConfigProvider>
					</main>
				</ScrollArea>
			</div>
		</div>
	);
}
