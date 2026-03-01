import * as vscode from 'vscode';
import { builtinModules, getImportCompletions, getModuleCompletionItems } from './modules';

export function activate(context: vscode.ExtensionContext) {
    const selector: vscode.DocumentSelector = { language: 'lexicon' };

    const completionProvider = vscode.languages.registerCompletionItemProvider(
        selector,
        {
            provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
                const line = document.lineAt(position.line).text;
                const beforeCursor = line.substring(0, position.character);
                
                const completionItems: vscode.CompletionItem[] = [];
                
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
                    const importItems = getImportCompletions(prefix);
                    completionItems.push(...importItems);
                }
                
                if (beforeCursor.length < 3) {
                    completionItems.push(...getModuleCompletionItems().slice(0, 8));
                }

                return completionItems;
            },
            resolveCompletionItem(item: vscode.CompletionItem) {
                return item;
            }
        },
        ...['.', ':', ' ', '\n']
    );

    context.subscriptions.push(completionProvider);

    const commandHandler = (command: string, ...args: any[]) => {
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

    context.subscriptions.push(
        vscode.commands.registerCommand('lexicon.run', () => commandHandler('lexicon.run')),
        vscode.commands.registerCommand('lexicon.newProject', () => commandHandler('lexicon.newProject')),
        vscode.commands.registerCommand('lexicon.build', () => commandHandler('lexicon.build'))
    );
}

export function deactivate() {}
