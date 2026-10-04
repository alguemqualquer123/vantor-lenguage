// Minimal ambient declarations so `tsc -p ./` works WITHOUT node_modules.
// At runtime inside VS Code these modules are provided by the host.
// NOTE: keep in sync with the APIs used in extension.ts (tsc will fail
// loudly if a used member is missing here).

declare const process: any;

declare namespace vscode {
    type ExtensionContext = any;
    type OutputChannel = any;
    type DiagnosticCollection = any;
    type StatusBarItem = any;
    type TextEditor = any;
    type TextDocument = any;
    type Position = any;
    type Range = any;
    type Location = any;
    type Hover = any;
    type MarkdownString = any;
    type CompletionItem = any;
    type DocumentSymbol = any;
    type CodeLens = any;
    type Diagnostic = any;
    type Uri = any;
    type CancellationToken = any;
    type ProviderResult<T> = any;
    type WorkspaceConfiguration = any;

    const window: any;
    const languages: any;
    const commands: any;
    const workspace: any;
    const Position: any;
    const Range: any;
    const Location: any;
    const Hover: any;
    const MarkdownString: any;
    const CompletionItem: any;
    const CompletionItemKind: any;
    const SnippetString: any;
    const TextEdit: any;
    const DocumentSymbol: any;
    const SymbolKind: any;
    const CodeLens: any;
    const Diagnostic: any;
    const DiagnosticSeverity: any;
    const StatusBarAlignment: any;
    const Uri: any;
    const ThemeIcon: any;
    const CodeAction: any;
}

declare module 'vscode' {
    export = vscode;
}

declare module 'child_process' {
    const cp: any;
    export = cp;
}

// Extended API surface used by providers.ts / extension.ts / telemetry.ts /
// extension.web.ts (still ambient stubs only — no node_modules at compile time).
// Every member is `any` so strict tsc passes; at runtime VS Code provides them.
declare namespace vscode {
    const tests: any;
    const tasks: any;
    const debug: any;

    const SignatureHelp: any;
    const SignatureInformation: any;
    const ParameterInformation: any;
    const InlayHint: any;
    const InlayHintKind: any;
    const FoldingRange: any;
    const DocumentColor: any;
    const Color: any;
    const CodeActionKind: any;
    const WorkspaceEdit: any;
    const SemanticTokensLegend: any;
    const SemanticTokensBuilder: any;
    const ShellExecution: any;
    const Task: any;
    const TaskScope: any;
    const TaskGroup: any;
    const TestRunProfileKind: any;
}
