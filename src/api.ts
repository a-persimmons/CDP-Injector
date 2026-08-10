import { invoke } from "@tauri-apps/api/core";

export type ProductProfile = {
  id: string;
  name: string;
  applicationPaths: string[];
  processNames: string[];
  contexts: {
    id: string;
    targetType: string;
    urlPrefixes: string[];
    excludeUrlContains: string[];
  }[];
  preview: {
    supported: boolean;
    restartMessage: string;
  };
};

export type ModuleSummary = {
  id: string;
  name: string;
  version: string;
  description: string;
  capabilities: string[];
  enabledFor: string[];
  hasService: boolean;
  browserAccessible: boolean;
};

export type ModulePackagePreview = {
  id: string;
  name: string;
  version: string;
  description: string;
  capabilities: string[];
  targets: string[];
  hasService: boolean;
};

export type ProductView = {
  profile: ProductProfile;
  modules: ModuleSummary[];
  services: {
    moduleId: string;
    host: string;
    port: number;
  }[];
  status: {
    productId: string;
    phase: string;
    launchMode: "normal" | "injected" | null;
    cdpStatus: "not used" | "connecting" | "connected" | "disconnected";
    moduleErrors: Record<string, string>;
  };
};

export type LaunchPreparation = {
  mode: "normal" | "injected";
  restartRequired: boolean;
};

export function listProducts() {
  return invoke<ProductView[]>("list_products");
}

export function setModuleEnabled(
  productId: string,
  moduleId: string,
  enabled: boolean,
) {
  return invoke<void>("set_module_enabled", { productId, moduleId, enabled });
}

export function prepareLaunch(productId: string) {
  return invoke<LaunchPreparation>("prepare_launch", { productId });
}

export function launchProduct(productId: string) {
  return invoke<void>("launch_product", { productId });
}

export function openModuleService(moduleId: string) {
  return invoke<void>("open_module_service", { moduleId });
}

export function inspectModulePackage(path: string) {
  return invoke<ModulePackagePreview>("inspect_module_package", { path });
}

export function installModulePackage(path: string) {
  return invoke<void>("install_module_package", { path });
}
