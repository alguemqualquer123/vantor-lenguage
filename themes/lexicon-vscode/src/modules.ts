import * as vscode from 'vscode';

export interface ModuleMember {
    name: string;
    detail: string;
    documentation: string;
}

export interface Module {
    fullPath: string;
    shortName: string;
    members: ModuleMember[];
}

// Stdlib surface as executed by `lex run` (see demo-api/ + lexicon crates).
// Only APIs the toolchain actually implements are listed here.
export const builtinModules: Module[] = [
    {
        fullPath: 'core::net::Http',
        shortName: 'Http',
        members: [
            { name: 'serve', detail: 'fn serve(address: String)', documentation: 'Starts the real axum HTTP server. The address is literal, e.g. Http::serve("0.0.0.0:3000"). Routes come from @Get/@Post/@Put/@Delete handlers.' },
            { name: 'get', detail: 'fn get(url: String) -> Response', documentation: 'Real blocking HTTP GET request (reqwest).' },
            { name: 'post', detail: 'fn post(url: String, body: String) -> Response', documentation: 'Real blocking HTTP POST request.' },
        ]
    },
    {
        fullPath: 'core::json::Json',
        shortName: 'Json',
        members: [
            { name: 'parse', detail: 'fn parse(json: String) -> JsonValue', documentation: 'Parses a JSON string (serde_json).' },
            { name: 'stringify', detail: 'fn stringify(value: any) -> String', documentation: 'Converts a value to a JSON string.' },
        ]
    },
    {
        fullPath: 'core::env::Env',
        shortName: 'Env',
        members: [
            { name: 'get', detail: 'fn get(key: String) -> String', documentation: 'Reads the REAL process environment. Assign to a variable first: let v = Env::get("PORT");' },
        ]
    },
    {
        fullPath: 'core::db::Db',
        shortName: 'Db',
        members: [
            { name: 'connect', detail: 'fn connect(url: String) -> String', documentation: 'Opens a database connection (e.g. Db::connect("sqlite:./dev.db")). Executes via the lexicon-db runtime (sqlx).' },
            { name: 'query', detail: 'fn query(sql: String) -> String', documentation: 'Runs a SELECT and returns rows as JSON.' },
            { name: 'execute', detail: 'fn execute(sql: String) -> String', documentation: 'Runs DDL/DML (CREATE/INSERT/...) and returns affected info.' },
        ]
    },
    {
        fullPath: 'core::io::Console',
        shortName: 'Console',
        members: [
            { name: 'writeLine', detail: 'fn writeLine(message: String)', documentation: 'Prints a line (evaluated by the runner).' },
            { name: 'write', detail: 'fn write(message: String)', documentation: 'Prints without forcing a newline.' },
        ]
    },
    {
        fullPath: 'core::collections::List',
        shortName: 'List',
        members: [
            { name: 'new', detail: 'fn new<T>() -> List<T>', documentation: 'Creates an empty list.' },
            { name: 'push', detail: 'fn push(&self, item: T)', documentation: 'Appends an item.' },
            { name: 'pop', detail: 'fn pop(&self) -> Option<T>', documentation: 'Removes the last item.' },
            { name: 'get', detail: 'fn get(&self, index: usize) -> Option<T>', documentation: 'Item by index.' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Number of items.' },
            { name: 'map', detail: 'fn map<U>(&self, f: fn(T) -> U) -> List<U>', documentation: 'Maps every element.' },
            { name: 'filter', detail: 'fn filter(&self, f: fn(&T) -> bool) -> List<T>', documentation: 'Keeps matching elements.' },
        ]
    },
    {
        fullPath: 'core::collections::Map',
        shortName: 'Map',
        members: [
            { name: 'new', detail: 'fn new<K, V>() -> Map<K, V>', documentation: 'Creates an empty map.' },
            { name: 'set', detail: 'fn set(&mut self, key: K, value: V)', documentation: 'Inserts a key/value pair.' },
            { name: 'get', detail: 'fn get(&self, key: &K) -> Option<V>', documentation: 'Value by key.' },
            { name: 'has', detail: 'fn has(&self, key: &K) -> bool', documentation: 'Whether the key exists.' },
            { name: 'remove', detail: 'fn remove(&mut self, key: &K) -> Option<V>', documentation: 'Removes a key.' },
            { name: 'len', detail: 'fn len(&self) -> usize', documentation: 'Number of entries.' },
        ]
    },
    {
        fullPath: 'core::option::Option',
        shortName: 'Option',
        members: [
            { name: 'isSome', detail: 'fn isSome(&self) -> bool', documentation: 'Whether it holds a value.' },
            { name: 'isNone', detail: 'fn isNone(&self) -> bool', documentation: 'Whether it is empty.' },
            { name: 'unwrap', detail: 'fn unwrap(self) -> T', documentation: 'Unwraps or fails with E0801.' },
            { name: 'unwrapOr', detail: 'fn unwrapOr(self, default: T) -> T', documentation: 'Unwraps or returns the default.' },
            { name: 'map', detail: 'fn map<U>(self, f: fn(T) -> U) -> Option<U>', documentation: 'Maps the inner value.' },
        ]
    },
    {
        fullPath: 'core::result::Result',
        shortName: 'Result',
        members: [
            { name: 'isOk', detail: 'fn isOk(&self) -> bool', documentation: 'Whether it is a success.' },
            { name: 'isErr', detail: 'fn isErr(&self) -> bool', documentation: 'Whether it is a failure.' },
            { name: 'unwrap', detail: 'fn unwrap(self) -> T', documentation: 'Unwraps success or fails.' },
            { name: 'unwrapOr', detail: 'fn unwrapOr(self, default: T) -> T', documentation: 'Unwraps or returns the default.' },
            { name: 'map', detail: 'fn map<U>(self, f: fn(T) -> U) -> Result<U, E>', documentation: 'Maps success.' },
            { name: 'mapErr', detail: 'fn mapErr<F>(self, f: fn(E) -> F) -> Result<T, F>', documentation: 'Maps the error.' },
        ]
    },
    {
        fullPath: 'core::channel::Channel',
        shortName: 'Channel',
        members: [
            { name: 'new', detail: 'fn new(capacity: usize) -> Channel<T>', documentation: 'Buffered channel with close semantics.' },
            { name: 'unbuffered', detail: 'fn unbuffered() -> Channel<T>', documentation: 'Rendezvous channel.' },
            { name: 'send', detail: 'fn send(&self, value: T) -> Result<(), String>', documentation: 'Sends; fails on closed/full with E0701.' },
            { name: 'recv', detail: 'fn recv(&self) -> Option<T>', documentation: 'Receives; None when closed and drained.' },
            { name: 'close', detail: 'fn close(&self)', documentation: 'Closes the channel.' },
        ]
    },
];

export function getModuleCompletionItems(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];

    for (const mod of builtinModules) {
        const moduleItem = new vscode.CompletionItem(mod.shortName, vscode.CompletionItemKind.Module);
        moduleItem.detail = mod.fullPath;
        moduleItem.documentation = new vscode.MarkdownString(`Module \`${mod.fullPath}\`\n\nImport with: \`import ${mod.fullPath};\``);
        moduleItem.insertText = mod.shortName;
        items.push(moduleItem);

        for (const member of mod.members) {
            const memberItem = new vscode.CompletionItem(
                `${mod.shortName}::${member.name}`,
                vscode.CompletionItemKind.Function
            );
            memberItem.detail = member.detail;
            memberItem.documentation = new vscode.MarkdownString(member.documentation);
            memberItem.insertText = `${mod.shortName}::${member.name}`;
            items.push(memberItem);
        }
    }

    return items;
}

export function getImportCompletions(prefix: string): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    const lowerPrefix = prefix.toLowerCase();

    for (const mod of builtinModules) {
        if (mod.fullPath.toLowerCase().includes(lowerPrefix) ||
            mod.shortName.toLowerCase().includes(lowerPrefix)) {

            const moduleItem = new vscode.CompletionItem(mod.shortName, vscode.CompletionItemKind.Module);
            moduleItem.detail = mod.fullPath;
            moduleItem.documentation = new vscode.MarkdownString(
                `**${mod.fullPath}**\n\nImport with: \`import ${mod.fullPath};\`\n\n**Members:**\n${mod.members.map(m => `- \`${m.name}\`: ${m.documentation}`).join('\n')}`
            );
            moduleItem.insertText = mod.fullPath;
            moduleItem.sortText = '0' + mod.shortName;
            items.push(moduleItem);
        }
    }

    return items;
}

export function findModule(shortName: string): Module | undefined {
    return builtinModules.find(m => m.shortName === shortName);
}
