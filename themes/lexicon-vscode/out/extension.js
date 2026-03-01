"use strict";
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
const modules_1 = require("./modules");
function activate(context) {
    const selector = { language: 'lexicon' };
    const completionProvider = vscode.languages.registerCompletionItemProvider(selector, {
        provideCompletionItems(document, position) {
            const line = document.lineAt(position.line).text;
            const beforeCursor = line.substring(0, position.character);
            const completionItems = [];
            const keywords = [
                'fn', 'let', 'mut', 'const', 'class', 'struct', 'enum', 'pub', 'private',
                'protected', 'static', 'async', 'await', 'if', 'else', 'match', 'for',
                'while', 'loop', 'return', 'break', 'continue', 'import', 'module',
                'type', 'trait', 'impl', 'self', 'super', 'crate', 'mod', 'use',
                'true', 'false', 'nil', 'try', 'catch', 'throw', 'throws'
            ];
            for (const kw of keywords) {
                const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
                item.detail = `keyword: ${kw}`;
                completionItems.push(item);
            }
            const types = [
                'String', 'i8', 'i16', 'i32', 'i64', 'i128', 'u8', 'u16', 'u32', 'u64', 'u128',
                'f32', 'f64', 'bool', 'char', 'byte', 'List', 'Map', 'Set', 'Option', 'Result',
                'Vec', 'Box', 'Rc', 'Arc', 'Cell', 'RefCell', 'HashMap', 'HashSet'
            ];
            for (const t of types) {
                const item = new vscode.CompletionItem(t, vscode.CompletionItemKind.TypeParameter);
                item.detail = `type: ${t}`;
                completionItems.push(item);
            }
            const isInImport = beforeCursor.trim().startsWith('import');
            const isInImportPath = /import\s+[\w.]*$/.test(beforeCursor.trim());
            if (isInImport || isInImportPath || beforeCursor.includes('http') || beforeCursor.includes('io') || beforeCursor.includes('json')) {
                const prefixMatch = beforeCursor.match(/([\w.]+)$/);
                const prefix = prefixMatch ? prefixMatch[1] : '';
                const importItems = (0, modules_1.getImportCompletions)(prefix);
                completionItems.push(...importItems);
            }
            if (beforeCursor.length < 3) {
                completionItems.push(...(0, modules_1.getModuleCompletionItems)().slice(0, 8));
            }
            return completionItems;
        },
        resolveCompletionItem(item) {
            return item;
        }
    }, ...['.', ':', ' ', '\n']);
    context.subscriptions.push(completionProvider);
    const commandHandler = (command, ...args) => {
        switch (command) {
            case 'lexicon.run':
                vscode.window.showInformationMessage('Running Lexicon file...');
                break;
            case 'lexicon.newProject':
                vscode.window.showInformationMessage('Creating new Lexicon project...');
                break;
            case 'lexicon.build':
                vscode.window.showInformationMessage('Building Lexicon project...');
                break;
        }
    };
    context.subscriptions.push(vscode.commands.registerCommand('lexicon.run', () => commandHandler('lexicon.run')), vscode.commands.registerCommand('lexicon.newProject', () => commandHandler('lexicon.newProject')), vscode.commands.registerCommand('lexicon.build', () => commandHandler('lexicon.build')));
}
function deactivate() { }
