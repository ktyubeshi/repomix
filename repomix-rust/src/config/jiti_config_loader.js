// repomix-rust/src/config/jiti_config_loader.js
// This script is used by repomix-rust to load JS/TS config files using jiti.

import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

// Dynamically import jiti from the current node_modules.
// We assume this script is run from the root of the repomix (Node.js) project,
// or that jiti is resolvable from the process's working directory.
const jitiPath = path.resolve(process.cwd(), 'node_modules', 'jiti', 'dist', 'jiti.mjs');

// Fallback for older jiti versions or different structures
let createJiti;
try {
  const { createJiti: jitiFn } = await import(jitiPath);
  createJiti = jitiFn;
} catch (e) {
  // Try CJS path or global if not found in default ESM path
  const cjsJitiPath = path.resolve(process.cwd(), 'node_modules', 'jiti');
  try {
    createJiti = (await import(cjsJitiPath)).default;
  } catch (e2) {
    console.error(`Error: Could not load jiti from node_modules. Is jiti installed? (${e.message} / ${e2.message})`);
    process.exit(1);
  }
}


if (!createJiti) {
  console.error("Error: createJiti function not found after dynamic import.");
  process.exit(1);
}

const configFilePath = process.argv[2]; // Path to the user's config file

if (!configFilePath) {
  console.error("Error: No config file path provided.");
  process.exit(1);
}

const jiti = createJiti(fileURLToPath(import.meta.url), {
  moduleCache: false, // Disable cache to ensure fresh config loads
  interopDefault: true, // Automatically use default export
});

async function loadConfig() {
  try {
    // Resolve the config file path relative to the current working directory
    const resolvedConfigPath = path.resolve(process.cwd(), configFilePath);
    
    // Use jiti to import the config module
    const configModule = await jiti.import(pathToFileURL(resolvedConfigPath).href);
    
    // Ensure configModule.default is used if it's an ESM default export, otherwise use configModule itself
    const config = configModule.default || configModule;

    console.log(JSON.stringify(config));
  } catch (error) {
    console.error("Error loading config:", error.message);
    process.exit(1);
  }
}

loadConfig();
