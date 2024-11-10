import init, { interpret_with_parser_to_string, interpret_to_string, parse_to_string } from "./interpreter/pkg/interpreter.js";
require.config({ paths: { 'vs': 'https://unpkg.com/monaco-editor/min/vs' }});

const snippets = {
    add:
`{
    add(1, 2)
}
`,
    fact:
`{
    // Factorial function
    def fact λ(n) {
        cond 
            (zero?(n) => 1) 
            (true => mul(n, fact(sub(n, 1))))
    };
    fact(10)
}`,
    helloworld:
`{
    // Showcasing string functions
    let hello "hello";
    let world "world";
    concat(hello, " ", to_uppercase(world))
}
`,
    cp5ex3:
`// Under dynamic scope rules, 'incr' is applied in an environment
// in which 'amt' has a global binding, and is bound to the value 1.
//
// The result here should be 6 under dynamic scope.
//
// Under lexical scope rules, applying 'incr' throws an error,
// because there is no binding for 'amt' in the environment in
// which 'incr' is bound (i.e. where the RHS lambda expression
// is evaluated).
//
{
  let incr lambda(n) {add(amt,n)};
  let amt 1;
  incr(5)
}
`
};

require(['vs/editor/editor.main'], async function() {
    await init();

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
    document.getElementById('editor').addEventListener('keydown', function(event) {
        if (event.altKey && event.key === 'Enter') {
            runWasm();
        }
    })
});
