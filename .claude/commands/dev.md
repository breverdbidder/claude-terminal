# Start development environment

Start the Tauri dev server with hot-reload by running `npm run tauri dev -- --features devtools` in the project root. The `devtools` feature enables Chrome DevTools in the webview (disabled in production builds for security).

If the command fails, diagnose the error:
- Missing dependencies → run `npm install` first
- Rust compilation errors → show the error and suggest a fix
- Port conflicts → identify the conflicting process
