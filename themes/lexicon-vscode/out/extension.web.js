"use strict";
// Lexicon web entry (vscode.dev / web) — vscode-only, NO node imports.
// Registers the same pure providers as the desktop entry. Binary-backed
// commands (anything that shells out to `lex`) degrade gracefully with an
// info message pointing at the desktop extension; local-only commands
// (snippets catalog, env, telemetry) work for real.
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const providers = __importStar(require("./providers"));
const Telemetry = __importStar(require("./telemetry"));
const DESKTOP_MSG = 'needs the desktop app with the `lex` binary on PATH. Install the Lexicon desktop extension and set `lexicon.path`.';
function desktopOnly(id) {
    void vscode.window.showInformationMessage(`Lexicon: '${id}' ${DESKTOP_MSG}`);
}
function activate(context) {
    const output = vscode.window.createOutputChannel('Lexicon');
    context.subscriptions.push(output);
    Telemetry.initTelemetry(output);
    // Providers are best-effort: a single throwing factory must never abort
    // activate() and orphan `lexicon.run` ("command not found").
    try {
        const semanticLegend = providers.createSemanticTokensLegend();
        const safe = (fn) => {
            try {
                context.subscriptions.push(fn());
            }
            catch {
                // skip providers unsupported by this host
            }
        };
        safe(() => vscode.languages.registerHoverProvider(providers.LANG_ID, providers.createHoverProvider()));
        safe(() => vscode.languages.registerCompletionItemProvider(providers.LANG_ID, providers.createCompletionProvider(), '.', ':'));
        safe(() => vscode.languages.registerDefinitionProvider(providers.LANG_ID, providers.createDefinitionProvider()));
        safe(() => vscode.languages.registerDocumentSymbolProvider(providers.LANG_ID, providers.createDocumentSymbolProvider()));
        safe(() => vscode.languages.registerCodeLensProvider(providers.LANG_ID, providers.createCodeLensProvider()));
        safe(() => vscode.languages.registerSignatureHelpProvider(providers.LANG_ID, providers.createSignatureHelpProvider(), '(', ','));
        safe(() => vscode.languages.registerInlayHintsProvider(providers.LANG_ID, providers.createInlayHintsProvider()));
        safe(() => vscode.languages.registerFoldingRangeProvider(providers.LANG_ID, providers.createFoldingRangeProvider()));
        safe(() => vscode.languages.registerDocumentColorProvider(providers.LANG_ID, providers.createDocumentColorProvider()));
        safe(() => vscode.languages.registerReferenceProvider(providers.LANG_ID, providers.createReferenceProvider()));
        safe(() => vscode.languages.registerRenameProvider(providers.LANG_ID, providers.createRenameProvider()));
        safe(() => vscode.languages.registerDocumentSemanticTokensProvider(providers.LANG_ID, providers.createSemanticTokensProvider(), semanticLegend));
        safe(() => vscode.languages.registerCodeActionsProvider(providers.LANG_ID, providers.createQuickFixProvider(), {
            providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
        }));
    }
    catch {
        // ignore provider failures on web
    }
    // Same command IDs as desktop; binary-backed ones degrade gracefully.
    const binaryBacked = [
        'lexicon.run', 'lexicon.runWithArgs', 'lexicon.check', 'lexicon.vet',
        'lexicon.lint', 'lexicon.fmt', 'lexicon.doc', 'lexicon.trace',
        'lexicon.test', 'lexicon.build', 'lexicon.debug', 'lexicon.newProject',
        'lexicon.serve', 'lexicon.testNet', 'lexicon.dnsCheck', 'lexicon.tlsCheck',
    ];
    for (const id of binaryBacked) {
        const thisId = id;
        try {
            context.subscriptions.push(vscode.commands.registerCommand(thisId, () => {
                Telemetry.recordCommand(thisId);
                desktopOnly(thisId);
            }));
        }
        catch {
            // a single duplicate must not orphan the remaining commands
        }
    }
    context.subscriptions.push(vscode.commands.registerCommand('lexicon.showProblems', async () => {
        Telemetry.recordCommand('lexicon.showProblems');
        await vscode.commands.executeCommand('workbench.actions.view.problems');
    }), vscode.commands.registerCommand('lexicon.showSnippets', async () => {
        Telemetry.recordCommand('lexicon.showSnippets');
        const doc = await vscode.workspace.openTextDocument(vscode.Uri.joinPath(context.extensionUri, 'SNIPPETS.md'));
        await vscode.window.showTextDocument(doc, { preview: true });
    }), vscode.commands.registerCommand('lexicon.env', async () => {
        Telemetry.recordCommand('lexicon.env');
        const cfg = vscode.workspace.getConfiguration('lexicon');
        output.appendLine(`features   : ${cfg.get('features') || '(none)'}`);
        output.appendLine(`target     : ${cfg.get('target') || '(native)'}`);
        output.appendLine(`note       : web entry — no local lex binary (${DESKTOP_MSG})`);
        output.show();
    }), vscode.commands.registerCommand('lexicon.telemetry.show', async () => {
        Telemetry.recordCommand('lexicon.telemetry.show');
        Telemetry.showStats();
    }), vscode.commands.registerCommand('lexicon.telemetry.reset', async () => {
        Telemetry.recordCommand('lexicon.telemetry.reset');
        Telemetry.resetStats();
    }));
}
function deactivate() {
    // nothing to dispose beyond subscriptions
}
