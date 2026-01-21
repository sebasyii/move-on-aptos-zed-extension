# Zed Move Extension

This extension adds support for the [Move language](https://move-language.github.io/move/) to the [Zed editor](https://zed.dev), specifically tailored for Aptos development.

## Features

- **Language Server Protocol (LSP)**: Integrates with the `aptos-language-server` to provide:
  - Diagnostics
  - Code completion (where supported)
  - Go to definition
  - Find references
- **Grammar**: Uses the [tree-sitter-move-on-aptos](https://github.com/aptos-labs/tree-sitter-move-on-aptos) grammar for syntax highlighting (in progress).

## Installation

This extension can be installed directly from the Zed extensions menu.

## Development

To work on this extension:

1.  Clone the repository.
2.  Open in Zed.
3.  Use the `Extensions: Install Dev Extension` command in Zed to load the local version.

## Configuration

The extension attempts to find `aptos-language-server` in your PATH. If not found, it will automatically download the appropriate binary from the [aptos-labs/move-vscode-extension](https://github.com/aptos-labs/move-vscode-extension) releases.
