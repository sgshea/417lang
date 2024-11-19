import init, { interpret_with_parser_to_string, interpret_to_string, parse_to_string } from "./interpreter/pkg/interpreter.js";
require.config({ paths: { 'vs': 'https://unpkg.com/monaco-editor/min/vs' } });

let snippets = {};

// List of snippet files
const snippetFiles = [
    'snippets/add.417',
    'snippets/factorial.417',
    'snippets/helloworld.417',
    'snippets/cp5ex3.417',
    'snippets/multi_assignment.417',
    // 'snippets/sort.417',
    'snippets/def.417'
];

async function loadSnippets() {
    const snippetPromises = snippetFiles.map(async (file) => {
        const response = await fetch(file);
        const snippetContent = await response.text();
        const snippetName = file.split('/').pop().replace('.417', ''); // Get the file name without extension
        snippets[snippetName] = snippetContent; // Add to snippets object
    });

    await Promise.all(snippetPromises);
}

require(['vs/editor/editor.main'], async function () {
    await init();
    await loadSnippets();

    // Create the Monaco editor for input
    const editor = monaco.editor.create(document.getElementById('editor'), {
        value: snippets.add,
        language: 'javascript',
        automaticLayout: true,
        theme: 'vs-light',
    });

    monaco.languages.typescript.javascriptDefaults.setModeConfiguration({
        completionItems: false
    });

    monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
        validate: false
    });

    // Create the Monaco editor for output (read-only)
    const outputEditor = monaco.editor.create(document.getElementById('outputEditor'), {
        value: '',
        language: 'json',
        readOnly: true,
        automaticLayout: true,
    });

    const dropdown = document.getElementById('snippetDropdown');
    for (const key in snippets) {
        const option = document.createElement('option');
        option.value = key;
        option.textContent = key;
        dropdown.appendChild(option);
    }

    function updateEditorValue() {
        const selectedValue = dropdown.value;
        if (selectedValue) {
            editor.setValue(snippets[selectedValue]);
        }
    }
    dropdown.addEventListener('change', updateEditorValue);

    const runWasm = () => {
        // Get the code from the Monaco editor
        const code = editor.getValue();
        let interpretResult;

        const useParse = document.getElementById('useParseCheckbox').checked;
        const useInterpret = document.getElementById('useInterpretCheckbox').checked;

        const useLexical = document.getElementById('useLexicalScope').checked;

        if (useParse && useInterpret) {
            interpretResult = interpret_with_parser_to_string(code, useLexical);
        } else if (useParse) {
            interpretResult = parse_to_string(code);
        } else if (useInterpret) {
            interpretResult = interpret_to_string(code, useLexical);
        } else {
            interpretResult = "Please select at least one option.";
        }

        // Set the result onto the output editor
        outputEditor.setValue(interpretResult);
    };

    // Add event listener to the button
    document.getElementById('runButton').addEventListener('click', runWasm);
    document.getElementById('editor').addEventListener('keydown', function (event) {
        if (event.altKey && event.key === 'Enter') {
            runWasm();
        }
    })
});
