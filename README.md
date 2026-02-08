# Rustboard

A drawing application built with Svelte and Rust (compiled to WebAssembly).

## Prerequisites

- Rust (with `wasm32-unknown-unknown` target)
- Node.js and npm
- wasm-pack: `cargo install wasm-pack`

## Project Structure

```
rustboard/
├── editor/              # Core Rust logic
│   └── src/
├── frontend/
│   ├── wasm/            # Rust translation layer (wasm-bindgen)
│   │   └── src/
│   └── src/
│       ├── lib/
│       │   ├── components/      # Svelte components
│       │   ├── stores/          # State management
│       │   ├── utils/           # Utility functions
│       │   │   └── canvas-operations/
│       │   └── wasm.ts          # WASM module loader
│       └── routes/              # SvelteKit routes
└── frontend/src/lib/wasm/pkg/  # Generated Wasm package
```

## Setup

1. Install dependencies:
   ```bash
   cargo install wasm-pack
   cd frontend && npm install
   ```

2. Build the Wasm module:
   ```bash
   npm run build:wasm:dev
   ```

3. Start the dev server:
   ```bash
   npm start
   ```

## Mobile UI Notes

- On phone-sized screens, the toolbar is docked at the bottom and becomes horizontally scrollable to avoid overflow.
- The style panel is shown as a bottom sheet with a limited height and internal scroll, so it does not block most of the canvas.
- Desktop layout remains unchanged.
