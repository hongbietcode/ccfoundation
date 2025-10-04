import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export type ConfigType =
  | "user"
  | "enterprise_macos"
  | "enterprise_linux"
  | "enterprise_windows"
  | "mcp_macos"
  | "mcp_linux"
  | "mcp_windows";

export interface ConfigFile {
  path: string;
  content: unknown;
  exists: boolean;
}

export interface ConfigStore {
  name: string;
  created_at: number;
  settings: unknown;
  using: boolean;
}

export const useConfigFiles = () => {
  return useQuery({
    queryKey: ["config-files"],
    queryFn: () => invoke<ConfigType[]>("list_config_files"),
  });
};

export const useConfigFile = (configType: ConfigType) => {
  return useQuery({
    queryKey: ["config-file", configType],
    queryFn: () => invoke<ConfigFile>("read_config_file", { configType }),
    enabled: !!configType,
  });
};

export const useWriteConfigFile = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ configType, content }: { configType: ConfigType; content: unknown }) =>
      invoke<void>("write_config_file", { configType, content }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["config-file", variables.configType] });
      queryClient.invalidateQueries({ queryKey: ["config-files"] });
    },
  });
};


export const useBackupClaudeConfigs = () => {
  return useMutation({
    mutationFn: () => invoke<void>("backup_claude_configs"),
  });
};

// Store management hooks

export const useStores = () => {
  return useQuery({
    queryKey: ["stores"],
    queryFn: () => invoke<ConfigStore[]>("get_stores"),
  });
};

export const useCurrentStore = () => {
  return useQuery({
    queryKey: ["current-store"],
    queryFn: () => invoke<ConfigStore | null>("get_current_store"),
  });
};

export const useCreateStore = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ name, settings }: { name: string; settings: unknown }) =>
      invoke<ConfigStore>("create_store", { name, settings }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stores"] });
      queryClient.invalidateQueries({ queryKey: ["current-store"] });
    },
  });
};

export const useDeleteStore = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => invoke<void>("delete_store", { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stores"] });
      queryClient.invalidateQueries({ queryKey: ["current-store"] });
    },
  });
};

export const useSetUsingStore = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => invoke<void>("set_using_store", { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stores"] });
      queryClient.invalidateQueries({ queryKey: ["current-store"] });
      queryClient.invalidateQueries({ queryKey: ["config-file", "user"] });
    },
  });
};

export const useSetCurrentStore = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => invoke<void>("set_using_store", { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stores"] });
      queryClient.invalidateQueries({ queryKey: ["current-store"] });
      queryClient.invalidateQueries({ queryKey: ["config-file", "user"] });
    },
  });
};