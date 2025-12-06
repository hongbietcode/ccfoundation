import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";

/**
 * Model information from backend config
 */
export interface ModelInfo {
	id: string;
	displayName: string;
	family: string;
	releaseDate: string;
	aliases: string[];
}

/**
 * Query keys for model-related queries
 */
export const modelKeys = {
	all: ["models"] as const,
	list: () => [...modelKeys.all, "list"] as const,
	default: () => [...modelKeys.all, "default"] as const,
};

/**
 * Fetch all available models from backend
 */
export async function getModels(): Promise<ModelInfo[]> {
	return await invoke<ModelInfo[]>("get_models");
}

/**
 * Fetch the default model ID
 */
export async function getDefaultModelId(): Promise<string> {
	return await invoke<string>("get_default_model_id");
}

/**
 * Normalize a model name using backend config
 */
export async function normalizeModel(modelName: string): Promise<string> {
	return await invoke<string>("normalize_model", { modelName });
}

/**
 * React Query hook to fetch all models
 */
export function useModels() {
	return useQuery({
		queryKey: modelKeys.list(),
		queryFn: getModels,
		staleTime: Number.POSITIVE_INFINITY, // Config rarely changes
	});
}

/**
 * React Query hook to fetch default model
 */
export function useDefaultModel() {
	return useQuery({
		queryKey: modelKeys.default(),
		queryFn: getDefaultModelId,
		staleTime: Number.POSITIVE_INFINITY,
	});
}

/**
 * Get display name for a model ID or alias
 */
export function getModelDisplayName(
	modelIdOrAlias: string | undefined,
	models: ModelInfo[]
): string {
	if (!modelIdOrAlias) return "Default Model";

	// Try exact ID match first
	const exactMatch = models.find((m) => m.id === modelIdOrAlias);
	if (exactMatch) return exactMatch.displayName;

	// Try alias match (case-insensitive)
	const lowerQuery = modelIdOrAlias.toLowerCase();
	const aliasMatch = models.find((m) =>
		m.aliases.some((alias) => alias.toLowerCase() === lowerQuery)
	);
	if (aliasMatch) return aliasMatch.displayName;

	// Fallback to showing the ID itself
	return modelIdOrAlias;
}

/**
 * Get model family for a model ID or alias
 */
export function getModelFamily(
	modelIdOrAlias: string | undefined,
	models: ModelInfo[]
): string | undefined {
	if (!modelIdOrAlias) return undefined;

	// Try exact ID match first
	const exactMatch = models.find((m) => m.id === modelIdOrAlias);
	if (exactMatch) return exactMatch.family;

	// Try alias match (case-insensitive)
	const lowerQuery = modelIdOrAlias.toLowerCase();
	const aliasMatch = models.find((m) =>
		m.aliases.some((alias) => alias.toLowerCase() === lowerQuery)
	);
	return aliasMatch?.family;
}
