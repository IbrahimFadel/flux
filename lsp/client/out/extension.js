"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const serverOptions = {
        command: "/home/ibrahim/dev/flux-lsp/target/debug/flux-lsp",
        args: ["server"],
    };
    const clientOptions = {
        documentSelector: [{ scheme: "file", language: "flux" }],
    };
    let client;
    try {
        client = new node_1.LanguageClient("flux", "Flux Language Server", serverOptions, clientOptions);
        // Window.showInformationMessage(`The Flux Language Server has been started.`);
    }
    catch (err) {
        vscode_1.window.showErrorMessage(`The Flux Language Server couldn't be started. See the output channel for details.`);
        return;
    }
    client.start();
}
exports.activate = activate;
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map