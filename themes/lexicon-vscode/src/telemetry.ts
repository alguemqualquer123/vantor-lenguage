// Lexicon local-only telemetry — NO network ever (no http/fetch/socket imports).
//
// What it does:
//   - counts command invocations in memory (per session, never persisted),
//   - appends one line per invocation to the Lexicon OutputChannel, but ONLY
//     when the `lexicon.telemetry.enabled` setting is true (default false).
// What it never does: send anything anywhere, write files, read env, PII.
//
// Commands (registered by the entries): `lexicon.telemetry.show` prints the
// table, `lexicon.telemetry.reset` clears the counters.

import * as vscode from 'vscode';

const counts = new Map<string, number>();
let output: any = undefined;

export function initTelemetry(channel: any): void {
    output = channel;
}

export function isTelemetryEnabled(): boolean {
    try {
        const cfg = vscode.workspace.getConfiguration('lexicon');
        return cfg.get('telemetry.enabled', false) === true;
    } catch {
        return false;
    }
}

/** Record one invocation of `id`. Always counted in memory; logged only when enabled. */
export function recordCommand(id: string): void {
    const next = (counts.get(id) ?? 0) + 1;
    counts.set(id, next);
    if (!isTelemetryEnabled()) {
        return;
    }
    try {
        if (output) {
            output.appendLine(`[telemetry] ${id} x${next}`);
        }
    } catch {
        // telemetry must never break the extension
    }
}

export function getStats(): Array<{ command: string; count: number }> {
    return [...counts.entries()]
        .map(([command, count]) => ({ command, count }))
        .sort((a, b) => b.count - a.count || (a.command < b.command ? -1 : 1));
}

export function resetStats(): void {
    counts.clear();
}

export function showStats(): void {
    try {
        if (!output) {
            return;
        }
        output.appendLine('--- Lexicon command stats (this session, local only) ---');
        const stats = getStats();
        if (stats.length === 0) {
            output.appendLine('(no commands recorded yet)');
        }
        for (const s of stats) {
            output.appendLine(`${s.command}: ${s.count}`);
        }
        output.show();
    } catch {
        // never throw from telemetry
    }
}
