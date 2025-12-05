import { createContext, useContext, type ReactNode } from "react";
import { useParams } from "react-router-dom";

type ConfigContextType = {
	contextType: "global" | "project";
	projectPath?: string;
	isGlobal: boolean;
	isProject: boolean;
};

const ConfigContext = createContext<ConfigContextType | null>(null);

export function ConfigProvider({ children }: { children: ReactNode }) {
	const { contextType, projectPath } = useParams<{
		contextType?: "global" | "project";
		projectPath?: string;
	}>();

	// projectPath from useParams is already decoded by React Router
	const decodedProjectPath = projectPath || undefined;

	// Determine actual context based on which params are present
	const actualContext: "global" | "project" = projectPath ? "project" : (contextType || "global");

	const value: ConfigContextType = {
		contextType: actualContext,
		projectPath: decodedProjectPath,
		isGlobal: actualContext === "global",
		isProject: actualContext === "project",
	};

	return <ConfigContext.Provider value={value}>{children}</ConfigContext.Provider>;
}

export function useConfigContext() {
	const context = useContext(ConfigContext);
	if (!context) {
		throw new Error("useConfigContext must be used within ConfigProvider");
	}
	return context;
}
