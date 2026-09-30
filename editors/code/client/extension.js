const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
    const config = vscode.workspace.getConfiguration('rpl');
    const command = config.get('serverPath') || 'rpl';

    const serverOptions = {
        run: {
            command: command,
            args: ['lsp'],
            transport: TransportKind.stdio
        },
        debug: {
            command: command,
            args: ['lsp'],
            transport: TransportKind.stdio
        }
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'rpl' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.rpl')
        }
    };

    client = new LanguageClient(
        'rplLanguageServer',
        'RPL Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = {
    activate,
    deactivate
};
