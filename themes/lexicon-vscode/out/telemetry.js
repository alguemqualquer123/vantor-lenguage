"use strict";
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
exports.initTelemetry = initTelemetry;
exports.isTelemetryEnabled = isTelemetryEnabled;
exports.recordCommand = recordCommand;
exports.getStats = getStats;
exports.resetStats = resetStats;
exports.showStats = showStats;
const vscode = __importStar(require("vscode"));
const counts = new Map();
let output = undefined;
function initTelemetry(channel) {
    output = channel;
}
function isTelemetryEnabled() {
    try {
        const cfg = vscode.workspace.getConfiguration('lexicon');
        return cfg.get('telemetry.enabled', false) === true;
    }
    catch {
        return false;
    }
}
/** Record one invocation of `id`. Always counted in memory; logged only when enabled. */
function recordCommand(id) {
    const next = (counts.get(id) ?? 0) + 1;
    counts.set(id, next);
    if (!isTelemetryEnabled()) {
        return;
    }
    try {
        if (output) {
            output.appendLine(`[telemetry] ${id} x${next}`);
        }
    }
    catch {
        // telemetry must never break the extension
    }
}
function getStats() {
    return [...counts.entries()]
        .map(([command, count]) => ({ command, count }))
        .sort((a, b) => b.count - a.count || (a.command < b.command ? -1 : 1));
}
function resetStats() {
    counts.clear();
}
function showStats() {
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
    }
    catch {
        // never throw from telemetry
    }
}
