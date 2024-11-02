import init, { interpret_with_parser_to_string } from "./pkg/interpreter.js";
require.config({ paths: { 'vs': 'https://unpkg.com/monaco-editor/min/vs' }});
require(['vs/editor/editor.main'], async function() {
    // Create the Monaco editor for input
    const editor = monaco.editor.create(document.getElementById('editor'), {
        value: `{\n\tadd(1, 2)\n}`,
    });

    // Create the Monaco editor for output (read-only)
    const outputEditor = monaco.editor.create(document.getElementById('outputEditor'), {
        value: '',
        readOnly: true
    });

    const runWasm = async () => {
        // Instantiate wasm module
        await init();

        // Get the code from the Monaco editor
        const code = editor.getValue();
        const interpretResult = interpret_with_parser_to_string(code);

        // Set the result onto the output editor
        outputEditor.setValue(interpretResult);
    };

    // Add event listener to the button
    document.getElementById('runButton').addEventListener('click', runWasm);
});
