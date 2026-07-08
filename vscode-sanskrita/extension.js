"use strict";
const vscode = require("vscode");
const { devanagarify } = require("./converter");

function fullRange(doc) {
  return new vscode.Range(doc.positionAt(0), doc.positionAt(doc.getText().length));
}

function activate(context) {
  // Command: संस्कृता: Convert roman → Devanagari  (Cmd+Shift+D)
  context.subscriptions.push(
    vscode.commands.registerCommand("sanskrita.convert", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;
      const doc = editor.document;
      const converted = devanagarify(doc.getText());
      if (converted === doc.getText()) {
        vscode.window.setStatusBarMessage("संस्कृता: पूर्वमेव देवनागरी ✓ (already Devanagari)", 3000);
        return;
      }
      editor.edit(b => b.replace(fullRange(doc), converted)).then(() => {
        vscode.window.setStatusBarMessage("संस्कृता: देवनागरीकृतम् ✓ (converted)", 3000);
      });
    })
  );

  // Convert-on-save for .सं / .sam files
  context.subscriptions.push(
    vscode.workspace.onWillSaveTextDocument(e => {
      if (e.document.languageId !== "sanskrita") return;
      const on = vscode.workspace.getConfiguration("sanskrita").get("convertOnSave", true);
      if (!on) return;
      const converted = devanagarify(e.document.getText());
      if (converted === e.document.getText()) return;
      e.waitUntil(Promise.resolve([vscode.TextEdit.replace(fullRange(e.document), converted)]));
    })
  );
}

function deactivate() {}

module.exports = { activate, deactivate };
