import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { ContextLayout } from "./components/ContextLayout";
import { RouteWrapper } from "./components/RouteWrapper";
import { AgentsPage } from "./pages/AgentsPage";
import { CommandsPage } from "./pages/CommandsPage";
import { ConfigEditorPage } from "./pages/ConfigEditorPage";
import { ConfigSwitcherPage } from "./pages/ConfigSwitcherPage";
import { ContextSelectorPage } from "./pages/ContextSelectorPage";
import { MCPPage } from "./pages/MCPPage";
import { MemoryPage } from "./pages/MemoryPage";
import { UsagePage } from "./pages/UsagePage";

const router = createBrowserRouter([
	// Root: Context Selector (chọn Global hoặc Project)
	{
		path: "/",
		element: (
			<RouteWrapper>
				<ContextSelectorPage />
			</RouteWrapper>
		),
	},
	// Context-based routes (Global hoặc Project)
	{
		path: "/context/:contextType",
		element: (
			<RouteWrapper>
				<ContextLayout />
			</RouteWrapper>
		),
		children: [
			{
				index: true,
				element: (
					<RouteWrapper>
						<ConfigSwitcherPage />
					</RouteWrapper>
				),
			},
			{
				path: "edit/:storeId",
				element: (
					<RouteWrapper>
						<ConfigEditorPage />
					</RouteWrapper>
				),
			},
			{
				path: "mcp",
				element: (
					<RouteWrapper>
						<MCPPage />
					</RouteWrapper>
				),
			},
			{
				path: "agents",
				element: (
					<RouteWrapper>
						<AgentsPage />
					</RouteWrapper>
				),
			},
			{
				path: "usage",
				element: (
					<RouteWrapper>
						<UsagePage />
					</RouteWrapper>
				),
			},
			{
				path: "memory",
				element: (
					<RouteWrapper>
						<MemoryPage />
					</RouteWrapper>
				),
			},
			{
				path: "commands",
				element: (
					<RouteWrapper>
						<CommandsPage />
					</RouteWrapper>
				),
			},
		],
	},
	// Project context routes (thêm projectPath vào URL)
	{
		path: "/context/project/:projectPath",
		element: (
			<RouteWrapper>
				<ContextLayout />
			</RouteWrapper>
		),
		children: [
			{
				index: true,
				element: (
					<RouteWrapper>
						<ConfigSwitcherPage />
					</RouteWrapper>
				),
			},
			{
				path: "edit/:storeId",
				element: (
					<RouteWrapper>
						<ConfigEditorPage />
					</RouteWrapper>
				),
			},
			{
				path: "mcp",
				element: (
					<RouteWrapper>
						<MCPPage />
					</RouteWrapper>
				),
			},
			{
				path: "agents",
				element: (
					<RouteWrapper>
						<AgentsPage />
					</RouteWrapper>
				),
			},
			{
				path: "memory",
				element: (
					<RouteWrapper>
						<MemoryPage />
					</RouteWrapper>
				),
			},
			{
				path: "commands",
				element: (
					<RouteWrapper>
						<CommandsPage />
					</RouteWrapper>
				),
			},
		],
	},
]);

export function Router() {
	return <RouterProvider router={router} />;
}
