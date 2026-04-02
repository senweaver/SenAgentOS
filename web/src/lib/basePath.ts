// Runtime base path for API requests.
// In dev mode: proxies through Vite dev server
// In production: served by the Rust gateway

const isDev = import.meta.env.DEV;

export const basePath = "";

export const apiOrigin = ""; // Use Vite proxy in dev, empty in prod

export const wsOrigin = isDev ? "ws://localhost:42617" : "";
