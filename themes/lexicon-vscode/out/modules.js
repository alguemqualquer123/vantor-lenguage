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
exports.builtinModules = void 0;
exports.getModuleCompletionItems = getModuleCompletionItems;
exports.getImportCompletions = getImportCompletions;
const vscode = __importStar(require("vscode"));
exports.builtinModules = [
    {
        fullPath: 'core.net.Http',
        shortName: 'Http',
        members: [
            { name: 'root', detail: 'fn root() -> String', documentation: 'Retorna mensagem de boas-vindas da API' },
            { name: 'hello', detail: 'fn hello() -> String', documentation: 'Retorna saudação do servidor HTTP' },
            { name: 'users', detail: 'fn users() -> String', documentation: 'Retorna lista de usuários de exemplo' },
            { name: 'stats', detail: 'fn stats(count: i32) -> String', documentation: 'Retorna estatísticas com count' },
        ]
    },
    {
        fullPath: 'core.io.Console',
        shortName: 'Console',
        members: [
            { name: 'log', detail: 'fn log(message: String)', documentation: 'Imprime mensagem no console' },
            { name: 'error', detail: 'fn error(message: String)', documentation: 'Imprime mensagem de erro' },
            { name: 'warn', detail: 'fn warn(message: String)', documentation: 'Imprime aviso' },
            { name: 'info', detail: 'fn info(message: String)', documentation: 'Imprime informação' },
        ]
    },
    {
        fullPath: 'core.json.Json',
        shortName: 'Json',
        members: [
            { name: 'parse', detail: 'fn parse(input: String) -> JsonValue', documentation: 'Parseia string JSON para objeto' },
            { name: 'stringify', detail: 'fn stringify(value: JsonValue) -> String', documentation: 'Converte objeto para string JSON' },
            { name: 'get', detail: 'fn get(json: JsonValue, key: String) -> JsonValue', documentation: 'Obtém valor de chave' },
            { name: 'set', detail: 'fn set(json: JsonValue, key: String, value: JsonValue)', documentation: 'Define valor de chave' },
        ]
    },
    {
        fullPath: 'core.env.Env',
        shortName: 'Env',
        members: [
            { name: 'get', detail: 'fn get(key: String) -> Option<String>', documentation: 'Obtém variável de ambiente' },
            { name: 'set', detail: 'fn set(key: String, value: String)', documentation: 'Define variável de ambiente' },
            { name: 'vars', detail: 'fn vars() -> Map<String, String>', documentation: 'Lista todas variáveis de ambiente' },
        ]
    },
    {
        fullPath: 'core.collections.List',
        shortName: 'List',
        members: [
            { name: 'new', detail: 'fn new<T>() -> List<T>', documentation: 'Cria nova lista vazia' },
            { name: 'push', detail: 'fn push<T>(list: &List<T>, item: T)', documentation: 'Adiciona item à lista' },
            { name: 'pop', detail: 'fn pop<T>(list: &List<T>) -> Option<T>', documentation: 'Remove último item da lista' },
            { name: 'map', detail: 'fn map<T, U>(list: List<T>, f: fn(T) -> U) -> List<U>', documentation: 'Aplica função a todos os elementos' },
            { name: 'filter', detail: 'fn filter<T>(list: List<T>, f: fn(&T) -> bool) -> List<T>', documentation: 'Filtra elementos' },
        ]
    },
    {
        fullPath: 'core.net.Grpc',
        shortName: 'Grpc',
        members: [
            { name: 'createServer', detail: 'fn createServer(config: GrpcConfig) -> GrpcServer', documentation: 'Cria servidor gRPC' },
            { name: 'createClient', detail: 'fn createClient(config: GrpcConfig) -> GrpcClient', documentation: 'Cria cliente gRPC' },
        ]
    },
    {
        fullPath: 'core.wasm.Env',
        shortName: 'Wasm',
        members: [
            { name: 'memory', detail: 'var memory: WebAssembly.Memory', documentation: 'Memória WebAssembly' },
            { name: 'alloc', detail: 'fn alloc(size: usize) -> pointer', documentation: 'Aloca memória WASM' },
            { name: 'dealloc', detail: 'fn dealloc(ptr: pointer)', documentation: 'Desaloca memória WASM' },
        ]
    },
];
function getModuleCompletionItems() {
    const items = [];
    for (const mod of exports.builtinModules) {
        const moduleItem = new vscode.CompletionItem(mod.shortName, vscode.CompletionItemKind.Module);
        moduleItem.detail = mod.fullPath;
        moduleItem.documentation = new vscode.MarkdownString(`Módulo ${mod.fullPath}\n\nImporte com: \`import ${mod.fullPath};\``);
        moduleItem.insertText = mod.fullPath;
        items.push(moduleItem);
        for (const member of mod.members) {
            const memberItem = new vscode.CompletionItem(member.name, vscode.CompletionItemKind.Function);
            memberItem.detail = `${mod.fullPath}.${member.name}: ${member.detail}`;
            memberItem.documentation = member.documentation;
            memberItem.additionalTextEdits = [
                vscode.TextEdit.insert(new vscode.Position(0, 0), `import ${mod.fullPath};\n`)
            ];
            items.push(memberItem);
        }
    }
    return items;
}
function getImportCompletions(prefix) {
    const items = [];
    const lowerPrefix = prefix.toLowerCase();
    for (const mod of exports.builtinModules) {
        if (mod.fullPath.toLowerCase().includes(lowerPrefix) ||
            mod.shortName.toLowerCase().includes(lowerPrefix)) {
            const moduleItem = new vscode.CompletionItem(mod.shortName, vscode.CompletionItemKind.Module);
            moduleItem.detail = mod.fullPath;
            moduleItem.documentation = new vscode.MarkdownString(`**${mod.fullPath}**\n\nImporte com: \`import ${mod.fullPath};\`\n\n**Membros:**\n${mod.members.map(m => `- \`${m.name}\`: ${m.documentation}`).join('\n')}`);
            moduleItem.insertText = mod.fullPath;
            moduleItem.sortText = '0' + mod.shortName;
            items.push(moduleItem);
        }
        for (const member of mod.members) {
            if (member.name.toLowerCase().includes(lowerPrefix)) {
                const memberItem = new vscode.CompletionItem(member.name, vscode.CompletionItemKind.Function);
                memberItem.detail = `${mod.fullPath}.${member.name}: ${member.detail}`;
                memberItem.documentation = member.documentation;
                memberItem.insertText = `${mod.fullPath}.${member.name}`;
                memberItem.sortText = '1' + member.name;
                items.push(memberItem);
            }
        }
    }
    return items;
}
