import type { ProjectConfig, ProjectRegistryEntry } from "./query";

export type ProjectStatus = "has_config" | "has_local" | "none";

export interface EnrichedProject {
	path: string;
	name: string;
	registryEntry?: ProjectRegistryEntry;
	hasLocalSettings?: boolean;
	status: ProjectStatus;
	claudeConfig?: Record<string, any>;
}

/**
 * Merge Claude projects from ~/.claude.json with project registry
 */
export function mergeProjectsWithConfigs(
	claudeProjects: ProjectConfig[],
	projectRegistry: ProjectRegistryEntry[],
): EnrichedProject[] {
	const enrichedProjects: EnrichedProject[] = [];

	// Create a map of registry entries by project path for quick lookup
	const registryMap = new Map<string, ProjectRegistryEntry>();
	for (const entry of projectRegistry) {
		registryMap.set(entry.projectPath, entry);
	}

	// Process Claude projects
	for (const claudeProject of claudeProjects) {
		const projectPath = claudeProject.path;
		const projectName = extractProjectName(projectPath);
		const registryEntry = registryMap.get(projectPath);

		// Check if project has .claude/ directory (has_config status)
		// This will be determined by checking if PROJECT/.claude/settings.json exists
		// For now, we determine by registry entry existence
		const status: ProjectStatus = registryEntry ? "has_config" : "none";

		enrichedProjects.push({
			path: projectPath,
			name: projectName,
			registryEntry: registryEntry,
			status: status,
			claudeConfig: claudeProject.config,
		});

		// Remove from map to track processed entries
		if (registryEntry) {
			registryMap.delete(projectPath);
		}
	}

	// Add remaining registry entries that don't have Claude project entries
	// (projects removed from Claude but still have configs)
	for (const entry of registryMap.values()) {
		const projectName = extractProjectName(entry.projectPath);
		enrichedProjects.push({
			path: entry.projectPath,
			name: projectName,
			registryEntry: entry,
			status: "has_config",
		});
	}

	return enrichedProjects;
}

/**
 * Get project status based on config and local settings
 */
export function getProjectStatus(
	hasConfig: boolean,
	hasLocalSettings: boolean,
): ProjectStatus {
	if (hasConfig) return "has_config";
	if (hasLocalSettings) return "has_local";
	return "none";
}

/**
 * Filter projects by status
 */
export function filterProjectsByStatus(
	projects: EnrichedProject[],
	status: ProjectStatus | "all",
): EnrichedProject[] {
	if (status === "all") return projects;
	return projects.filter((p) => p.status === status);
}

/**
 * Search projects by name or path
 */
export function searchProjects(
	projects: EnrichedProject[],
	query: string,
): EnrichedProject[] {
	if (!query.trim()) return projects;

	const lowerQuery = query.toLowerCase();
	return projects.filter(
		(p) =>
			p.name.toLowerCase().includes(lowerQuery) ||
			p.path.toLowerCase().includes(lowerQuery),
	);
}

/**
 * Extract project name from path
 */
export function extractProjectName(path: string): string {
	const parts = path.split(/[/\\]/);
	return parts[parts.length - 1] || path;
}

/**
 * Sort projects by various criteria
 */
export function sortProjects(
	projects: EnrichedProject[],
	sortBy: "name" | "path" | "lastUsed",
): EnrichedProject[] {
	const sorted = [...projects];

	switch (sortBy) {
		case "name":
			sorted.sort((a, b) => a.name.localeCompare(b.name));
			break;
		case "path":
			sorted.sort((a, b) => a.path.localeCompare(b.path));
			break;
		case "lastUsed":
			sorted.sort((a, b) => {
				const aTime = a.registryEntry?.lastUsedAt || 0;
				const bTime = b.registryEntry?.lastUsedAt || 0;
				return bTime - aTime; // Most recent first
			});
			break;
	}

	return sorted;
}

/**
 * Check if project path is valid
 */
export function isValidProjectPath(path: string): boolean {
	return path.length > 0 && !path.includes("null") && !path.includes("undefined");
}
