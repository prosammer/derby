import type { ClassValue } from "clsx";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { invoke } from "@tauri-apps/api";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function readEnvVariable(variableName: string): Promise<string> {
  return invoke("get_env_var", { key: variableName }).then((response: unknown) => {
    return response as string;
  }).catch((error) => {
    throw new Error(error);
  });
}