const vscode = require('vscode');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * Discovers the RPL compiler binary executable.
 */
function findRplBinary() {
    const configPath = vscode.workspace.getConfiguration('rpl').get('serverPath');
    if (configPath && configPath !== 'rpl' && fs.existsSync(configPath)) {
        return configPath;
    }
    // Check known local project targets
    const localTargets = [
        'D:\\Dev\\rpl\\target\\release\\rpl.exe',
        'D:\\Dev\\rpl\\target\\debug\\rpl.exe',
        path.join(__dirname, '..', '..', '..', 'target', 'release', 'rpl.exe'),
        path.join(__dirname, '..', '..', '..', 'target', 'debug', 'rpl.exe')
    ];
    for (const p of localTargets) {
        if (fs.existsSync(p)) {
            return p;
        }
    }
    return 'rpl';
}

/**
 * Minimal JSON-RPC LSP client communicating over stdio.
 * Requires zero external npm dependencies.
 */
class RplLspClient {
    constructor(binPath, outputChannel) {
        this.binPath = binPath;
        this.output = outputChannel;
        this.proc = null;
        this.reqId = 1;
        this.pendingRequests = new Map();
        this.buffer = Buffer.alloc(0);
        this.onDiagnostics = null;
    }

    start() {
        try {
            this.proc = spawn(this.binPath, ['lsp'], { stdio: ['pipe', 'pipe', 'pipe'] });
        } catch (err) {
            this.output.appendLine('[ERROR] Failed to spawn ' + this.binPath + ': ' + err);
            return;
        }

        this.proc.stdout.on('data', (chunk) => this.handleData(chunk));
        this.proc.stderr.on('data', (data) => this.output.appendLine('[stderr] ' + data.toString()));
        this.proc.on('exit', (code) => this.output.appendLine('[LSP exited with code ' + code + ']'));
        this.proc.on('error', (err) => this.output.appendLine('[LSP process error] ' + err));

        // Send LSP initialize handshake
        this.sendRequest('initialize', {
            processId: process.pid,
            capabilities: {
                textDocument: {
                    publishDiagnostics: { relatedInformation: false },
                    hover: { contentFormat: ['markdown', 'plaintext'] }
                }
            }
        }).then(() => {
            this.sendNotification('initialized', {});
            this.output.appendLine('[LSP] RPL Language Server successfully initialized.');
        }).catch(err => {
            this.output.appendLine('[LSP] Initialize failed: ' + JSON.stringify(err));
        });
    }

    sendNotification(method, params) {
        if (!this.proc || !this.proc.stdin.writable) return;
        const msg = JSON.stringify({ jsonrpc: '2.0', method, params });
        const payload = `Content-Length: ${Buffer.byteLength(msg, 'utf8')}\r\n\r\n${msg}`;
        this.proc.stdin.write(payload, 'utf8');
    }

    sendRequest(method, params) {
        if (!this.proc || !this.proc.stdin.writable) {
            return Promise.reject(new Error('LSP process is not running'));
        }
        const id = this.reqId++;
        const msg = JSON.stringify({ jsonrpc: '2.0', id, method, params });
        const payload = `Content-Length: ${Buffer.byteLength(msg, 'utf8')}\r\n\r\n${msg}`;
        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });
            this.proc.stdin.write(payload, 'utf8');
        });
    }

    handleData(chunk) {
        this.buffer = Buffer.concat([this.buffer, chunk]);
        while (true) {
            const headerEnd = this.buffer.indexOf('\r\n\r\n');
            if (headerEnd === -1) break;

            const headerStr = this.buffer.slice(0, headerEnd).toString('utf8');
            const match = headerStr.match(/Content-Length:\s*(\d+)/i);
            if (!match) {
                this.buffer = this.buffer.slice(headerEnd + 4);
                continue;
            }

            const contentLength = parseInt(match[1], 10);
            const totalMsgLength = headerEnd + 4 + contentLength;
            if (this.buffer.length < totalMsgLength) {
                // Incomplete message; wait for next chunk
                break;
            }

            const bodyBuffer = this.buffer.slice(headerEnd + 4, totalMsgLength);
            this.buffer = this.buffer.slice(totalMsgLength);

            try {
                const message = JSON.parse(bodyBuffer.toString('utf8'));
                this.handleMessage(message);
            } catch (e) {
                this.output.appendLine('[LSP JSON parse error] ' + e);
            }
        }
    }

    handleMessage(msg) {
        if (msg.id !== undefined && this.pendingRequests.has(msg.id)) {
            const { resolve, reject } = this.pendingRequests.get(msg.id);
            this.pendingRequests.delete(msg.id);
            if (msg.error) {
                reject(msg.error);
            } else {
                resolve(msg.result);
            }
        } else if (msg.method === 'textDocument/publishDiagnostics') {
            if (this.onDiagnostics) {
                this.onDiagnostics(msg.params);
            }
        }
    }

    stop() {
        if (this.proc) {
            this.proc.kill();
            this.proc = null;
        }
    }
}

/**
 * Activates the RPL extension.
 */
function activate(context) {
    const outputChannel = vscode.window.createOutputChannel('RPL Language Server');
    const binPath = findRplBinary();
    outputChannel.appendLine('[RPL Extension] Initializing with binary: ' + binPath);

    const client = new RplLspClient(binPath, outputChannel);
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('rpl');
    context.subscriptions.push(diagnosticCollection);

    client.onDiagnostics = (params) => {
        try {
            const uri = vscode.Uri.parse(params.uri);
            const diags = (params.diagnostics || []).map(d => {
                const range = new vscode.Range(
                    d.range.start.line, d.range.start.character,
                    d.range.end.line, d.range.end.character
                );
                const severity = d.severity === 1 ? vscode.DiagnosticSeverity.Error :
                                 d.severity === 2 ? vscode.DiagnosticSeverity.Warning :
                                 vscode.DiagnosticSeverity.Information;
                const diag = new vscode.Diagnostic(range, d.message, severity);
                diag.source = d.source || 'rpl';
                return diag;
            });
            diagnosticCollection.set(uri, diags);
        } catch (e) {
            outputChannel.appendLine('[Diagnostics error] ' + e);
        }
    };

    client.start();

    // Notify LSP of existing open RPL documents
    vscode.workspace.textDocuments.forEach(doc => {
        if (doc.languageId === 'rpl') {
            client.sendNotification('textDocument/didOpen', {
                textDocument: {
                    uri: doc.uri.toString(),
                    languageId: 'rpl',
                    version: doc.version,
                    text: doc.getText()
                }
            });
        }
    });

    // Notify LSP on open
    context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(doc => {
        if (doc.languageId === 'rpl') {
            client.sendNotification('textDocument/didOpen', {
                textDocument: {
                    uri: doc.uri.toString(),
                    languageId: 'rpl',
                    version: doc.version,
                    text: doc.getText()
                }
            });
        }
    }));

    // Notify LSP on content change
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(event => {
        if (event.document.languageId === 'rpl') {
            client.sendNotification('textDocument/didChange', {
                textDocument: {
                    uri: event.document.uri.toString(),
                    version: event.document.version
                },
                contentChanges: [{ text: event.document.getText() }]
            });
        }
    }));

    // Clean up diagnostics on close
    context.subscriptions.push(vscode.workspace.onDidCloseTextDocument(doc => {
        if (doc.languageId === 'rpl') {
            diagnosticCollection.delete(doc.uri);
            client.sendNotification('textDocument/didClose', {
                textDocument: { uri: doc.uri.toString() }
            });
        }
    }));

    // Hover Provider
    context.subscriptions.push(vscode.languages.registerHoverProvider('rpl', {
        async provideHover(document, position) {
            try {
                const res = await client.sendRequest('textDocument/hover', {
                    textDocument: { uri: document.uri.toString() },
                    position: { line: position.line, character: position.character }
                });
                if (res && res.contents) {
                    const md = typeof res.contents === 'string' ? res.contents : res.contents.value;
                    if (md) {
                        return new vscode.Hover(new vscode.MarkdownString(md));
                    }
                }
            } catch (e) {
                // Silently ignore hover timeout / error
            }
            return null;
        }
    }));

    context.subscriptions.push({
        dispose: () => client.stop()
    });
}

function deactivate() {
    // Subscriptions handle cleanup
}

module.exports = {
    activate,
    deactivate
};
