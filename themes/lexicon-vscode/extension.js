const vscode = require('vscode');

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    const pipeDecoration = vscode.window.createTextEditorDecorationType({
        after: {
            contentText: '▷',
            color: '#58A6FF',
            fontWeight: 'bold',
            margin: '0 0 0 -1ch'
        },
        textDecoration: 'none; display: none;'
    });

    const arrowDecoration = vscode.window.createTextEditorDecorationType({
        after: {
            contentText: '→',
            color: '#FF7B72',
            margin: '0 0 0 -1ch'
        },
        textDecoration: 'none; display: none;'
    });

    let activeEditor = vscode.window.activeTextEditor;

    function updateDecorations() {
        if (!activeEditor || activeEditor.document.languageId !== 'lexicon') {
            return;
        }

        const regExPipe = /\|>/g;
        const regExArrow = /->/g;
        const text = activeEditor.document.getText();
        
        const pipes = [];
        const arrows = [];

        let match;
        while ((match = regExPipe.exec(text))) {
            const startPos = activeEditor.document.positionAt(match.index);
            const endPos = activeEditor.document.positionAt(match.index + match[0].length);
            const range = new vscode.Range(startPos, endPos);
            pipes.push(range);
        }

        while ((match = regExArrow.exec(text))) {
            const startPos = activeEditor.document.positionAt(match.index);
            const endPos = activeEditor.document.positionAt(match.index + match[0].length);
            const range = new vscode.Range(startPos, endPos);
            arrows.push(range);
        }

        activeEditor.setDecorations(pipeDecoration, pipes);
        activeEditor.setDecorations(arrowDecoration, arrows);
    }

    if (activeEditor) {
        updateDecorations();
    }

    vscode.window.onDidChangeActiveTextEditor(editor => {
        activeEditor = editor;
        if (editor) {
            updateDecorations();
        }
    }, null, context.subscriptions);

    vscode.workspace.onDidChangeTextDocument(event => {
        if (activeEditor && event.document === activeEditor.document) {
            updateDecorations();
        }
    }, null, context.subscriptions);

    // --- HOVER PROVIDER ---
    const hoverProvider = vscode.languages.registerHoverProvider('lexicon', {
        provideHover(document, position, token) {
            const range = document.getWordRangeAtPosition(position);
            if (!range) return null;
            const word = document.getText(range);
            const text = document.getText();

            // 1. Check for Language Keywords/Tokens
            const lexiconTokens = {
                'import': { title: 'Keyword: import', desc: 'Used to bring modules or specific symbols into the current scope.' },
                'module': { title: 'Keyword: module', desc: 'Declares a new module namespace.' },
                'pub': { title: 'Modifier: pub', desc: 'Makes a symbol public and accessible from other modules.' },
                'fn': { title: 'Keyword: fn', desc: 'Used to declare a function or method.' },
                'let': { title: 'Keyword: let', desc: 'Declares an immutable variable (by default).' },
                'var': { title: 'Keyword: var', desc: 'Declares a mutable variable.' },
                'mut': { title: 'Modifier: mut', desc: 'Used with `let` to allow mutation of a variable.' },
                'const': { title: 'Keyword: const', desc: 'Declares a compile-time constant.' },
                'class': { title: 'Keyword: class', desc: 'Defines a new class with properties and methods.' },
                'struct': { title: 'Keyword: struct', desc: 'Defines a data structure (value type).' },
                'enum': { title: 'Keyword: enum', desc: 'Defines an enumeration type.' },
                'async': { title: 'Keyword: async', desc: 'Marks a function or block as asynchronous.' },
                'await': { title: 'Keyword: await', desc: 'Pauses execution until an asynchronous operation completes.' },
                'macro': { title: 'Keyword: macro', desc: 'Defines a metaprogramming macro for code generation.' },
                'if': { title: 'Keyword: if', desc: 'Starts a conditional block.' },
                'else': { title: 'Keyword: else', desc: 'Defines an alternative block for an `if` statement.' },
                'for': { title: 'Keyword: for', desc: 'Starts a loop over a range or collection.' },
                'while': { title: 'Keyword: while', desc: 'Starts a loop that runs while a condition is true.' },
                'return': { title: 'Keyword: return', desc: 'Exits a function and optionally returns a value.' },
                'break': { title: 'Keyword: break', desc: 'Exits the innermost loop immediately.' },
                'continue': { title: 'Keyword: continue', desc: 'Skips the rest of the current loop iteration.' },
                'match': { title: 'Keyword: match', desc: 'Starts a pattern matching block. Used for control flow based on the shape of data (ADTs).' },
                'Shape': { title: 'ADT: Shape', desc: 'Custom Algebraic Data Type defined in current scope.', from: 'local' },
                'Result': { title: 'ADT: Result', desc: 'Standard sum type for error handling.', from: 'core.prelude' },
                'Circle': { title: 'Variant: Circle', desc: 'Enum variant carrying a radius (f64).' },
                'Rectangle': { title: 'Variant: Rectangle', desc: 'Enum variant carrying width and height (f64, f64).' },
                'Square': { title: 'Variant: Square', desc: 'Enum variant carrying a side length (f64).' },
                'Ok': { title: 'Variant: Ok', desc: 'Success variant of the Result type.' },
                'Err': { title: 'Variant: Err', desc: 'Failure variant of the Result type.' },
                'true': { title: 'Literal: true', desc: 'Boolean true value.' },
                'false': { title: 'Literal: false', desc: 'Boolean false value.' },
                'null': { title: 'Literal: null', desc: 'Represents the absence of a value.' },
                'self': { title: 'Keyword: self', desc: 'Refers to the current instance of a class or struct.' },
                'super': { title: 'Keyword: super', desc: 'Refers to the parent class or module.' },
                'type': { title: 'Keyword: type', desc: 'Used to define a type alias.' },
                'interface': { title: 'Keyword: interface', desc: 'Defines a contract for classes or structs to implement.' },
                'implements': { title: 'Keyword: implements', desc: 'Declares that a class or struct implements an interface.' },
                'extends': { title: 'Keyword: extends', desc: 'Declares that a class inherits from another class.' },
                'i32': { title: 'Type: i32', desc: '32-bit signed integer.' },
                'i64': { title: 'Type: i64', desc: '64-bit signed integer.' },
                'u32': { title: 'Type: u32', desc: '32-bit unsigned integer.' },
                'u64': { title: 'Type: u64', desc: '64-bit unsigned integer.' },
                'f32': { title: 'Type: f32', desc: '32-bit floating point number.' },
                'f64': { title: 'Type: f64', desc: '64-bit floating point number.' },
                'bool': { title: 'Type: bool', desc: 'Boolean type (true or false).' },
                'char': { title: 'Type: char', desc: 'Unicode character type.' },
                'String': { title: 'Type: String', desc: 'UTF-8 encoded string type.' },
                'void': { title: 'Type: void', desc: 'Represents no value or return.' },
                'Console': { title: 'Core Library: Console', desc: 'Standard input/output module for terminal interaction.', from: 'core.io' },
                'Http': { title: 'Core Library: Http', desc: 'Module for handling HTTP requests and servers.', from: 'core.net' },
                'Json': { title: 'Core Library: Json', desc: 'Module for JSON serialization and parsing.', from: 'core.json' },
                'Env': { title: 'Core Library: Env', desc: 'Module for environment variable management.', from: 'core.env' },
                'List': { title: 'Core Library: List', desc: 'Dynamic array collection type.', from: 'core.collections' },
                'Map': { title: 'Core Library: Map', desc: 'Key-value pair collection type.', from: 'core.collections' },
                'Set': { title: 'Core Library: Set', desc: 'Unique value collection type.', from: 'core.collections' },
                'Path': { title: 'Core Library: Path', desc: 'Module for file system path manipulation.', from: 'core.fs' },
                'File': { title: 'Core Library: File', desc: 'Module for file read/write operations.', from: 'core.fs' },
                'Process': { title: 'Core Library: Process', desc: 'Module for system process interaction.', from: 'core.sys' },
                'Thread': { title: 'Core Library: Thread', desc: 'Module for multi-threading support.', from: 'core.sys' }
            };

            if (lexiconTokens[word]) {
                const token = lexiconTokens[word];
                const hoverContent = new vscode.MarkdownString();
                hoverContent.appendMarkdown(`### ${token.title}\n`);
                if (token.from) {
                    hoverContent.appendMarkdown(`*From: \`${token.from}\`*\n\n`);
                }
                hoverContent.appendMarkdown(`${token.desc}`);
                return new vscode.Hover(hoverContent);
            }

            // 2. Check for Decorators (starts with @)
            const fullRange = document.getWordRangeAtPosition(position, /@[a-zA-Z0-9_]+/);
            if (fullRange) {
                const decorator = document.getText(fullRange);
                const decorators = {
                    '@Test': 'Marks a function as a unit test to be picked up by `lex test`.',
                    '@Configuration': 'Marks a class as a source of dependency injection beans.',
                    '@Bean': 'Used within a @Configuration class to register a dependency.',
                    '@Getter': 'Auto-generates getter methods for class fields.',
                    '@Setter': 'Auto-generates setter methods for class fields.'
                };
                if (decorators[decorator]) {
                    const hoverContent = new vscode.MarkdownString();
                    hoverContent.appendMarkdown(`### Decorator: ${decorator}\n---\n${decorators[decorator]}`);
                    return new vscode.Hover(hoverContent);
                }
            }

            // 3. Search for definitions in current file (Original logic)
            const patterns = [
                {
                    // Function: fn name(...) -> type
                    regex: new RegExp(`fn\\s+${word}\\s*\\(([^)]*)\\)\\s*(->\\s*([a-zA-Z0-9_?]+))?`, 'g'),
                    type: 'function',
                    format: (m) => `**fn** ${word}(${m[1] || ''}) ${m[3] ? '-> ' + m[3] : ''}`
                },
                {
                    // Class: class name
                    regex: new RegExp(`class\\s+${word}`, 'g'),
                    type: 'class',
                    format: () => `**class** ${word}`
                },
                {
                    // Enum: enum name
                    regex: new RegExp(`enum\\s+${word}`, 'g'),
                    type: 'enum',
                    format: () => `**enum** ${word}`
                },
                {
                    // Variable: let/var name: type
                    regex: new RegExp(`(let|var)\\s+(mut\\s+)?${word}\\s*:\\s*([a-zA-Z0-9_?]+)`, 'g'),
                    type: 'variable',
                    format: (m) => `**${m[1]}** ${word}: ${m[3]}`
                }
            ];

            for (const p of patterns) {
                let match;
                while ((match = p.regex.exec(text))) {
                    // Found a definition! Now look for comments above it
                    const matchStartPos = document.positionAt(match.index);
                    const docInfo = p.format(match);
                    
                    // Look for comments in previous lines
                    let comments = [];
                    let lineNum = matchStartPos.line - 1;
                    
                    while (lineNum >= 0) {
                        const line = document.lineAt(lineNum).text.trim();
                        if (line.startsWith('///')) {
                            comments.unshift(line.substring(3).trim());
                        } else if (line.startsWith('//')) {
                            comments.unshift(line.substring(2).trim());
                        } else if (line.endsWith('*/')) {
                            // Basic support for block comments
                            let blockLine = lineNum;
                            while (blockLine >= 0) {
                                const bLine = document.lineAt(blockLine).text.trim();
                                if (bLine.startsWith('/**') || bLine.startsWith('/*')) {
                                    break;
                                }
                                let clean = bLine.startsWith('*') ? bLine.substring(1).trim() : bLine;
                                if (!clean.endsWith('*/')) comments.unshift(clean);
                                blockLine--;
                            }
                            break;
                        } else {
                            break;
                        }
                        lineNum--;
                    }

                    const hoverContent = new vscode.MarkdownString();
                    hoverContent.appendCodeblock(docInfo, 'lexicon');
                    if (comments.length > 0) {
                        hoverContent.appendMarkdown('---\n' + comments.join('\n\n'));
                    }
                    
                    return new vscode.Hover(hoverContent);
                }
            }

            return null;
        }
    });

    context.subscriptions.push(hoverProvider);

    // --- COMPLETION PROVIDER ---
    const completionProvider = vscode.languages.registerCompletionItemProvider('lexicon', {
        provideCompletionItems(document, position) {
            const completions = [];

            // Keywords
            const keywords = ['fn', 'class', 'struct', 'enum', 'let', 'var', 'mut', 'const', 'if', 'else', 'for', 'while', 'return', 'import', 'module', 'pub', 'async', 'await', 'macro'];
            keywords.forEach(k => {
                const item = new vscode.CompletionItem(k, vscode.CompletionItemKind.Keyword);
                completions.push(item);
            });

            // Common types
            const types = ['i32', 'i64', 'f32', 'f64', 'bool', 'char', 'String', 'void', 'List', 'Http', 'Console'];
            types.forEach(t => {
                const item = new vscode.CompletionItem(t, vscode.CompletionItemKind.Class);
                completions.push(item);
            });

            // Simple snippets
            const fnSnippet = new vscode.CompletionItem('fn', vscode.CompletionItemKind.Snippet);
            fnSnippet.insertText = new vscode.SnippetString('fn ${1:name}(${2:params}) -> ${3:void} {\n\t$0\n}');
            completions.push(fnSnippet);

            return completions;
        }
    }, '.');

    // --- DEFINITION PROVIDER ---
    const definitionProvider = vscode.languages.registerDefinitionProvider('lexicon', {
        provideDefinition(document, position, token) {
            const range = document.getWordRangeAtPosition(position);
            const word = document.getText(range);
            const text = document.getText();

            // Simple regex to find definition in the same file
            const regex = new RegExp(`\\b(fn|class|struct|enum|let|var|mut|const)\\s+${word}\\b`, 'g');
            let match;
            while ((match = regex.exec(text))) {
                const startPos = document.positionAt(match.index);
                const endPos = document.positionAt(match.index + match[0].length);
                return new vscode.Location(document.uri, new vscode.Range(startPos, endPos));
            }
            return null;
        }
    });

    context.subscriptions.push(completionProvider, definitionProvider);

    // --- CODE LENS PROVIDER ---
    const codeLensProvider = vscode.languages.registerCodeLensProvider('lexicon', {
        provideCodeLenses(document) {
            const lenses = [];
            const text = document.getText();
            const mainRegex = /\b(pub\s+)?fn\s+main\s*\(/g;
            
            let match;
            while ((match = mainRegex.exec(text))) {
                const startPos = document.positionAt(match.index);
                const line = document.lineAt(startPos.line);
                const range = line.range;
                
                lenses.push(new vscode.CodeLens(range, {
                    title: "$(play) Run",
                    tooltip: "Run this file with 'lex run'",
                    command: "lexicon.run",
                    arguments: [document.uri]
                }));

                lenses.push(new vscode.CodeLens(range, {
                    title: "$(debug-start) Debug",
                    tooltip: "Debug this file",
                    command: "lexicon.debug",
                    arguments: [document.uri]
                }));
            }
            
            return lenses;
        }
    });

    // --- COMMANDS ---
    const runCommand = vscode.commands.registerCommand('lexicon.run', (uri) => {
        const terminal = vscode.window.terminals.find(t => t.name === 'Lexicon') || vscode.window.createTerminal('Lexicon');
        terminal.show();
        terminal.sendText(`lex run "${uri.fsPath}"`);
    });

    const debugCommand = vscode.commands.registerCommand('lexicon.debug', (uri) => {
        const terminal = vscode.window.terminals.find(t => t.name === 'Lexicon Debug') || vscode.window.createTerminal('Lexicon Debug');
        terminal.show();
        // Since we don't have a real debugger yet, we'll run it with a simulated debug flag or just 'run'
        terminal.sendText(`lex run "${uri.fsPath}" --debug`);
        vscode.window.showInformationMessage('Debugging Lexicon starting...');
    });

    context.subscriptions.push(codeLensProvider, runCommand, debugCommand);
}

function deactivate() {}

module.exports = {
    activate,
    deactivate
};
