// Lexicon project templates (item 15): main.lex + .env.dev/.prod +
// run-dev scripts, modeled on demo-api/. Writes via vscode.workspace.fs
// (web-safe); the caller registers the `lexicon.newProject` command id.

import * as vscode from 'vscode';

const TEMPLATES: Record<string, { label: string; main: string; port: string }> = {
    default: {
        label: 'Default (console hello)',
        port: '',
        main: 'pub fn main() -> void {\n    Console::writeLine("Hello, Lexicon!");\n}\n',
    },
    api: {
        label: 'REST API (@Get + serve)',
        port: '3000',
        main: '@Get("/")\npub fn hello() -> String {\n    return "Hello from Lexicon REST API!";\n}\n\npub fn main() -> void {\n    Http::serve("0.0.0.0:3000");\n}\n',
    },
    service: {
        label: 'Service (struct + main)',
        port: '50051',
        main: 'struct User {\n    id: i32;\n    name: String;\n}\n\npub fn main() -> void {\n    Console::writeLine("service up");\n}\n',
    },
};

const ENV_DEV = (port: string): string =>
    `APP_ENV=dev\nLOG_LEVEL=debug\n${port ? `PORT=${port}\n` : ''}DATABASE_URL=sqlite://./dev.db\n`;

const ENV_PROD = (port: string): string =>
    `APP_ENV=prod\nLOG_LEVEL=info\n${port ? `PORT=${port}\n` : ''}DATABASE_URL=sqlite://./prod.db\n`;

const RUN_PS1 = 'Get-Content .env.dev | ForEach-Object { if ($_ -match "^(.*?)=(.*)$") { Set-Item -Path ("env:" + $Matches[1]) -Value $Matches[2] } }\nlex run --watch src/main.lex\n';

const RUN_SH = '#!/bin/sh\nset -a\n. ./.env.dev\nset +a\nexec lex run --watch src/main.lex\n';

const MANIFEST = (name: string, template: string): string =>
    `[project]\nname = "${name}"\nversion = "0.1.0"\ntemplate = "${template}"\n\n[dependencies]\ncore = "0.1.0"\n`;

function enc(s: string): Uint8Array {
    const out = new Uint8Array(s.length);
    for (let i = 0; i < s.length; i++) {
        out[i] = s.charCodeAt(i) & 0xff;
    }
    return out;
}

export async function runNewProjectFlow(): Promise<void> {
    const templatePick = await vscode.window.showQuickPick(
        Object.keys(TEMPLATES).map((k) => ({ label: TEMPLATES[k].label, key: k })),
        { placeHolder: 'Lexicon project template' }
    );
    if (!templatePick) {
        return;
    }
    const name = await vscode.window.showInputBox({ prompt: 'Project name', value: 'my_lex_app' });
    if (!name) {
        return;
    }
    const folders = vscode.workspace.workspaceFolders;
    let dir: vscode.Uri | undefined;
    if (folders && folders.length === 1) {
        dir = folders[0].uri;
    } else {
        const picked = await vscode.window.showOpenDialog({ canSelectFiles: false, canSelectFolders: true, openLabel: 'Create project here' });
        dir = picked && picked[0];
    }
    if (!dir) {
        return;
    }
    const tpl = TEMPLATES[(templatePick as { key: string }).key];
    const root = vscode.Uri.joinPath(dir, name);
    const fs = vscode.workspace.fs;
    await fs.createDirectory(vscode.Uri.joinPath(root, 'src'));
    const files: Array<[string, string]> = [
        ['src/main.lex', tpl.main],
        ['lexicon.toml', MANIFEST(name, (templatePick as { key: string }).key)],
        ['.env.dev', ENV_DEV(tpl.port)],
        ['.env.prod', ENV_PROD(tpl.port)],
        ['run-dev.ps1', RUN_PS1],
        ['run-dev.sh', RUN_SH],
    ];
    for (const [rel, content] of files) {
        await fs.writeFile(vscode.Uri.joinPath(root, rel), enc(content));
    }
    const main = vscode.Uri.joinPath(root, 'src/main.lex');
    await vscode.window.showTextDocument(main, { preview: false });
    void vscode.window.showInformationMessage(`Lexicon: project '${name}' created (main.lex + .env.dev/.prod + run-dev scripts).`);
}
