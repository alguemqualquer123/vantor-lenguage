# Vantor Dark Pro

A professional dark theme for Visual Studio Code with modern identity, optimal contrast for long coding sessions, and comprehensive support for modern programming languages.

![Vantor Dark Pro](https://via.placeholder.com/800x450/0F1117/5DA9FF?text=Vantor+Dark+Pro)

## Overview

Vantor Dark Pro is a carefully crafted dark theme designed for developers who spend long hours coding. It features:

- **Balanced Contrast**: Easy on the eyes, perfect for extended coding sessions
- **Clear Syntax Highlighting**: Distinct colors for keywords, types, functions, variables, strings, and comments
- **Modern Palette**: Professional color scheme inspired by top themes like Dracula, One Dark Pro, and Nord
- **Excellent Readability**: Optimized for both IPS and OLED monitors
- **Font Ligature Support**: Works beautifully with Fira Code, JetBrains Mono, and other ligature-enabled fonts

## Supported Languages

- TypeScript / JavaScript
- Java
- C# / .NET
- Rust
- Python
- Go
- C / C++
- HTML / CSS / SCSS / Less
- JSON / YAML / TOML
- Markdown
- SQL
- Shell / Bash / PowerShell
- Ruby
- PHP
- Swift
- Kotlin
- Scala
- And many more...

## Color Palette

| Element | Color | Hex |
|---------|-------|-----|
| Background | Dark Navy | `#0F1117` |
| Sidebar | Dark Blue | `#151823` |
| Panel | Dark Gray | `#1A1D29` |
| Primary Accent | Sky Blue | `#5DA9FF` |
| Secondary Accent | Purple | `#A277FF` |
| Success | Green | `#4CD97B` |
| Warning | Orange | `#FFB454` |
| Error | Red | `#FF5C7A` |
| Comment | Gray | `#5C6370` |
| String | Light Green | `#A3E88D` |
| Function | Blue | `#82AAFF` |
| Keyword | Purple | `#C792EA` |
| Type | Yellow | `#FFCB6B` |

## Installation

### From VS Code Marketplace

1. Open VS Code
2. Go to Extensions (`Ctrl+Shift+X` or `Cmd+Shift+X`)
3. Search for "Vantor Dark Pro"
4. Click Install

### From VSIX File

1. Download the `.vsix` file
2. In VS Code, go to Extensions (`Ctrl+Shift+X` or `Cmd+Shift+X`)
3. Click the `...` menu in the top-right
4. Select "Install from VSIX..."
5. Choose the downloaded file

### Manual Installation (Development)

1. Clone this repository:
   ```bash
   git clone https://github.com/vantor/vantor-dark-pro.git
   ```

2. Copy the theme folder to VS Code extensions directory:
   - **Windows**: `%USERPROFILE%\.vscode\extensions`
   - **macOS**: `~/.vscode/extensions`
   - **Linux**: `~/.vscode/extensions`

3. Restart VS Code

4. Go to Settings → Color Theme and select "Vantor Dark Pro"

## Development

### Prerequisites

- Node.js (v18 or higher)
- npm or yarn

### Setup

```bash
cd vantor-dark-pro
npm install
```

### Testing Locally

1. Press `F5` in VS Code to launch the Extension Development Host
2. The theme will be automatically loaded
3. Go to Settings → Color Theme and select "Vantor Dark Pro"

### Building VSIX Package

1. Install vsce (VS Code Extension Manager):
   ```bash
   npm install -g @vscode/vsce
   ```

2. Build the package:
   ```bash
   vsce package
   ```

3. This will generate a `.vsix` file that can be installed in VS Code

## Publishing to Marketplace

### Prerequisites

- Microsoft account
- Personal Access Token (PAT) from Azure DevOps

### Publishing Steps

1. Update the version in `package.json`

2. Package the extension:
   ```bash
   vsce package
   ```

3. Publish to marketplace:
   ```bash
   vsce publish
   ```

4. Or upload manually:
   - Go to [VS Code Marketplace Publisher Portal](https://marketplace.visualstudio.com/manage)
   - Click "New Extension"
   - Upload the `.vsix` file

### Versioning

Follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Update version in `package.json` before publishing

## Configuration

### Recommended Settings

For the best experience, add these settings to your `settings.json`:

```json
{
  "editor.fontFamily": "'Fira Code', 'JetBrains Mono', 'Consolas', monospace",
  "editor.fontLigatures": true,
  "editor.fontSize": 14,
  "editor.lineHeight": 24,
  "editor.renderLineHighlight": "all",
  "editor.cursorBlinking": "smooth",
  "editor.cursorSmoothCaretAnimation": "on",
  "editor.smoothScrolling": true,
  "workbench.colorTheme": "Vantor Dark Pro"
}
```

## Customization

You can customize specific colors by adding overrides in your `settings.json`:

```json
{
  "workbench.colorCustomizations": {
    "[Vantor Dark Pro]": {
      "editor.background": "#0F1117",
      "editorCursor.foreground": "#5DA9FF"
    }
  }
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Inspired by:
- Dracula Theme
- One Dark Pro
- GitHub Dark
- Nord
- Monokai Pro

---

**Enjoy coding with Vantor Dark Pro!**
