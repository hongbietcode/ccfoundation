import { get, isEmpty, isPlainObject, set, transform } from "lodash-es";
import { ChevronLeftIcon } from "lucide-react";
import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router-dom";
import { match } from "ts-pattern";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import {
	useProjectSettings,
	useWriteProjectSettings,
	useProjectRegistry,
	useUpdateProjectRegistry,
} from "../lib/query";

// Helper functions from ConfigEditorPage
function isValidValue(value: any): boolean {
	if (value === undefined || value === null || Number.isNaN(value)) {
		return false;
	}
	if (typeof value === "string" && value.trim() === "") {
		return false;
	}
	return true;
}

function deepClean(obj: any): any {
	if (!isPlainObject(obj)) {
		return isValidValue(obj) ? obj : undefined;
	}

	const cleaned = transform(
		obj,
		(result: any, value, key) => {
			const cleanedValue = deepClean(value);
			if (
				cleanedValue !== undefined &&
				!(isPlainObject(cleanedValue) && isEmpty(cleanedValue))
			) {
				result[key] = cleanedValue;
			}
		},
		{},
	);

	return cleaned;
}

function convertToNestedJSON(formData: Record<string, any>) {
	const { configName, inheritFromGlobal, ...settings } = formData;

	const settingsJSON = transform(
		settings,
		(result, value, key) => {
			set(result, key, value);
		},
		{} as Record<string, any>,
	);

	const cleanedSettings = deepClean(settingsJSON);

	return {
		configName,
		inheritFromGlobal,
		settings: cleanedSettings,
	};
}

type FieldConfig = {
	label: string;
	name: string;
	type: "text" | "number" | "boolean" | "textarea" | "tags" | "select";
	description?: string;
	placeholder?: string;
	options?: string[];
};

type SectionConfig = {
	sectionName: string;
	fields: FieldConfig[];
};

const createFields = (t: (key: string) => string): SectionConfig[] => [
	{
		sectionName: t("configEditor.sections.common"),
		fields: [
			{
				label: "ANTHROPIC_BASE_URL",
				name: "env.ANTHROPIC_BASE_URL",
				type: "text",
				description: "Override the API URL for model requests",
			},
			{
				label: "ANTHROPIC_AUTH_TOKEN",
				name: "env.ANTHROPIC_AUTH_TOKEN",
				type: "text",
				description: "This value will be sent as the Authorization header.",
			},
			{
				label: "ANTHROPIC_MODEL",
				name: "env.ANTHROPIC_MODEL",
				type: "text",
				description: "Name of the model setting to use",
			},
		],
	},
	{
		sectionName: t("configEditor.sections.generalSettings"),
		fields: [
			{
				label: "Model",
				name: "model",
				type: "text",
				description: "Override the default model to use for Claude Code",
				placeholder: "claude-sonnet-4-5-20250929",
			},
			{
				label: "Output Style",
				name: "outputStyle",
				type: "select",
				description: "Configure output style to adjust the system prompt",
				options: ["Default", "Explanatory", "Concise"],
			},
		],
	},
];

export function ProjectConfigEditor() {
	const { t } = useTranslation();
	const { projectPath } = useParams<{ projectPath: string }>();
	const decodedPath = projectPath ? decodeURIComponent(projectPath) : "";

	// Query project settings from PROJECT/.claude/settings.json
	const { data: projectSettings, isLoading: loadingSettings } = useProjectSettings(decodedPath);
	const writeProjectSettings = useWriteProjectSettings();

	// Query registry for metadata (title, inheritFromGlobal)
	const { data: projectRegistry } = useProjectRegistry();
	const updateProjectRegistry = useUpdateProjectRegistry();

	const registryEntry = projectRegistry?.find((entry) => entry.projectPath === decodedPath);

	const fields = createFields(t);

	const defaultValues: Record<string, any> = {
		configName: registryEntry?.title || "",
		inheritFromGlobal: registryEntry?.inheritFromGlobal ?? false,
	};

	if (projectSettings?.settings) {
		fields.forEach((section) => {
			section.fields.forEach((field) => {
				const value = get(projectSettings.settings, field.name);
				if (value !== undefined) {
					defaultValues[field.name] = value;
				}
			});
		});
	}

	const { register, control, handleSubmit, reset, watch } = useForm({
		defaultValues,
	});

	const inheritFromGlobal = watch("inheritFromGlobal");

	useEffect(() => {
		if (projectSettings || registryEntry) {
			reset(defaultValues);
		}
	}, [projectSettings, registryEntry, reset]);

	const onSave = handleSubmit((formValues) => {
		const { configName, inheritFromGlobal, settings } =
			convertToNestedJSON(formValues);

		// Save settings to PROJECT/.claude/settings.json
		writeProjectSettings.mutate({
			projectPath: decodedPath,
			settings: settings,
		});

		// Update registry metadata
		updateProjectRegistry.mutate({
			projectPath: decodedPath,
			title: configName,
			inheritFromGlobal: inheritFromGlobal,
			parentGlobalConfigId: registryEntry?.parentGlobalConfigId || null,
		});
	});

	const isLoading = loadingSettings;

	if (isLoading) {
		return (
			<div className="">
				<div
					className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
					data-tauri-drag-region
				>
					<div data-tauri-drag-region>
						<h3 className="font-bold" data-tauri-drag-region>
							{t("loading")}
						</h3>
					</div>
				</div>
			</div>
		);
	}

	if (!projectSettings?.exists) {
		return (
			<div className="">
				<div
					className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
					data-tauri-drag-region
				>
					<Button asChild variant="ghost" size="sm">
						<Link to="/project-configs">
							<ChevronLeftIcon size={14} className="text-muted-foreground" />
							<span className="text-muted-foreground">Back</span>
						</Link>
					</Button>
				</div>
				<div className="p-8">
					<Alert>
						<AlertDescription>Project .claude/ directory not found</AlertDescription>
					</Alert>
				</div>
			</div>
		);
	}

	return (
		<div className="space-y-4">
			<nav
				className="px-2 py-3 flex items-center justify-between sticky top-0 bg-background z-10 border-b"
				data-tauri-drag-region
			>
				<Button asChild variant="ghost" size="sm">
					<Link to="/project-configs">
						<ChevronLeftIcon size={14} className="text-muted-foreground" />
						<span className="text-muted-foreground">All Project Configs</span>
					</Link>
				</Button>

				<div className="mr-2 flex items-center gap-2">
					<Button
						onClick={onSave}
						disabled={writeProjectSettings.isPending || updateProjectRegistry.isPending}
						size="sm"
					>
						{t("configEditor.save")}
					</Button>
				</div>
			</nav>

			<section className="px-8 space-y-4">
				<div>
					<h3 className="pb-2 font-medium mx-2 text-muted-foreground text-sm">
						{t("configEditor.configName")}
					</h3>
					<input
						{...register("configName")}
						type="text"
						className="text-sm px-2 text-muted-foreground border rounded-sm w-[200px] h-7 bg-background"
					/>
				</div>

				<div className="flex items-center space-x-2 mx-2">
					<Controller
						name="inheritFromGlobal"
						control={control}
						render={({ field }) => (
							<>
								<Switch
									id="inherit-global"
									checked={field.value}
									onCheckedChange={field.onChange}
								/>
								<Label
									htmlFor="inherit-global"
									className="text-sm text-muted-foreground"
								>
									Inherit from global configuration
								</Label>
							</>
						)}
					/>
				</div>

				{inheritFromGlobal && (
					<Alert className="mx-2">
						<AlertDescription className="text-xs">
							This project config will inherit all settings from the active
							global configuration. You can still override specific settings
							below.
						</AlertDescription>
					</Alert>
				)}
			</section>

			<section className="space-y-8 pb-8">
				{fields.map((field) => (
					<div key={field.sectionName}>
						<h3 className="px-10 py-2 font-medium text-muted-foreground text-sm">
							{field.sectionName}
						</h3>
						<div className="mx-8 rounded-lg bg-card p-3 space-y-5 border">
							{field.fields.map((field) => (
								<div className="" key={field.name}>
									<div className="flex gap-2 items-center justify-between">
										<div className="space-y-1">
											<div className="text-muted-foreground text-sm min-w-40 shrink-0">
												{field.label}
											</div>
											{field.description && (
												<p className="text-muted-foreground/50 text-sm line-clamp-1">
													{field.description}
												</p>
											)}
										</div>
										{match({ type: field.type })
											.with({ type: "boolean" }, () => (
												<Controller
													name={field.name}
													control={control}
													render={({ field: { onChange, value } }) => (
														<Select
															value={
																value !== undefined ? String(value) : undefined
															}
															onValueChange={(val) => onChange(val === "true")}
														>
															<SelectTrigger className="w-1/2">
																<SelectValue placeholder="Default" />
															</SelectTrigger>
															<SelectContent>
																<SelectItem value="true">true</SelectItem>
																<SelectItem value="false">false</SelectItem>
															</SelectContent>
														</Select>
													)}
												/>
											))
											.with({ type: "select" }, () => (
												<Controller
													name={field.name}
													control={control}
													render={({ field: { onChange, value } }) => (
														<Select value={value} onValueChange={onChange}>
															<SelectTrigger className="w-1/2">
																<SelectValue
																	placeholder={field.placeholder || "Select..."}
																/>
															</SelectTrigger>
															<SelectContent>
																{field.options?.map((option) => (
																	<SelectItem key={option} value={option}>
																		{option}
																	</SelectItem>
																))}
															</SelectContent>
														</Select>
													)}
												/>
											))
											.with({ type: "textarea" }, () => (
												<Textarea
													{...register(field.name)}
													className="w-1/2 text-sm"
													placeholder={field.placeholder}
												/>
											))
											.with({ type: "number" }, () => (
												<Input
													{...register(field.name, {
														setValueAs: (v) =>
															v === "" ? undefined : Number(v),
													})}
													type="number"
													className="text-sm w-1/2 h-7"
													placeholder={field.placeholder}
												/>
											))
											.otherwise(() => (
												<Input
													{...register(field.name)}
													type="text"
													className="text-sm w-1/2 h-7"
													placeholder={field.placeholder}
												/>
											))}
									</div>
								</div>
							))}
						</div>
					</div>
				))}
			</section>
		</div>
	);
}
