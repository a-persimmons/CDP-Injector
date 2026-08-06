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
  enabledFor: string[];
};

export type ProductView = {
  profile: ProductProfile;
  modules: ModuleSummary[];
  status: {
    productId: string;
    phase: string;
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
