import { join } from "path";
import { ExtensionContext, window as Window } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const command = context.asAbsolutePath('flux-lsp');
  const serverOptions: ServerOptions = {
    command,
    args: ["server"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "flux" }],
  };

  let client: LanguageClient;
	try {
		client = new LanguageClient(
      "flux",
      "Flux Language Server",
      serverOptions,
      clientOptions
    );
    // Window.showInformationMessage(`The Flux Language Server has been started.`);
	} catch (err) {
		Window.showErrorMessage(`The Flux Language Server couldn't be started. See the output channel for details.`);
		return;
	}
  

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
} 