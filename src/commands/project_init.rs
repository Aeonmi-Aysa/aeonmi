// Project scaffolding and initialization for Aeonmi CLI
// Handles 'aeon new' and 'aeon init' commands with templates

use crate::cli_enhanced::{print_error, print_info, print_success, AeonCli, AeonCommand};
use anyhow::{bail, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Handle the new project command
pub fn handle_new(_cli: &AeonCli, command: &AeonCommand) -> Result<()> {
    if let AeonCommand::New {
        name,
        template,
        git,
        open,
        lib,
    } = command
    {
        print_info(&format!("Creating new Aeonmi project '{}'", name));

        let project_path = std::env::current_dir()?.join(name);

        if project_path.exists() {
            bail!("Directory '{}' already exists", name);
        }

        // Create project directory
        fs::create_dir_all(&project_path)?;

        // Initialize project structure
        create_project_structure(&project_path, name, template, *lib)?;

        // Initialize git repository if requested
        if *git {
            initialize_git_repo(&project_path)?;
        }

        print_success(&format!(
            "Created Aeonmi project '{}' in {}",
            name,
            project_path.display()
        ));

        // Print next steps
        println!();
        println!("{}", "Next steps:".bright_white().bold());
        println!("  cd {}", name.bright_green());
        println!(
            "  {}           # Build the project",
            "aeon build".bright_blue()
        );
        println!(
            "  {}             # Run the project",
            "aeon run".bright_blue()
        );

        // Open in editor if requested
        if *open {
            open_in_editor(&project_path)?;
        }

        Ok(())
    } else {
        bail!("Invalid command for new handler")
    }
}

/// Handle the init command
pub fn handle_init(_cli: &AeonCli, command: &AeonCommand) -> Result<()> {
    if let AeonCommand::Init {
        name,
        template,
        git,
        lib,
    } = command
    {
        let current_dir = std::env::current_dir()?;
        let project_name = name
            .as_deref()
            .or_else(|| current_dir.file_name().and_then(|n| n.to_str()))
            .unwrap_or("aeonmi-project");

        print_info(&format!(
            "Initializing Aeonmi project '{}' in current directory",
            project_name
        ));

        // Check if directory is empty or only contains safe files
        check_directory_for_init(&current_dir)?;

        // Initialize project structure
        create_project_structure(&current_dir, project_name, template, *lib)?;

        // Initialize git repository if requested
        if *git {
            initialize_git_repo(&current_dir)?;
        }

        print_success(&format!("Initialized Aeonmi project '{}'", project_name));

        // Print next steps
        println!();
        println!("{}", "Next steps:".bright_white().bold());
        println!(
            "  {}           # Build the project",
            "aeon build".bright_blue()
        );
        println!(
            "  {}             # Run the project",
            "aeon run".bright_blue()
        );

        Ok(())
    } else {
        bail!("Invalid command for init handler")
    }
}

/// Create the project structure based on template
fn create_project_structure(
    project_path: &Path,
    name: &str,
    template: &str,
    is_lib: bool,
) -> Result<()> {
    // Create basic directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;
    fs::create_dir_all(project_path.join("examples"))?;
    fs::create_dir_all(project_path.join("docs"))?;

    // Create Aeonmi.toml
    create_project_manifest(project_path, name, is_lib)?;

    // Create source files based on template
    match template {
        "basic" => create_basic_template(project_path, name, is_lib)?,
        "quantum" => create_quantum_template(project_path, name, is_lib)?,
        "ai" => create_ai_template(project_path, name, is_lib)?,
        "web" => create_web_template(project_path, name, is_lib)?,
        "cli" => create_cli_template(project_path, name, is_lib)?,
        _ => {
            print_error(&format!("Unknown template: {}", template));
            create_basic_template(project_path, name, is_lib)?;
        }
    }

    // Create additional files
    create_readme(project_path, name, template)?;
    create_gitignore(project_path)?;
    create_license(project_path)?;

    Ok(())
}

/// Create the project manifest (Aeonmi.toml)
fn create_project_manifest(project_path: &Path, name: &str, is_lib: bool) -> Result<()> {
    let manifest_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"

[aeonmi]
entry = "src/main.ai"
modules = []

{}
"#,
        name,
        if is_lib {
            ""
        } else {
            "[[aeonmi.tests]]\nname = \"basic\"\npath = \"tests/basic.ai\""
        }
    );

    fs::write(project_path.join("Aeonmi.toml"), manifest_content)?;
    Ok(())
}

/// Create basic project template
fn create_basic_template(project_path: &Path, _name: &str, is_lib: bool) -> Result<()> {
    if is_lib {
        let lib_content = r#"// Library main module
// This is the entry point for your Aeonmi library

/// A simple greeting function
function greet(name: string): string {
    return "Hello, " + name + "!";
}

/// Add two numbers together
function add(a: number, b: number): number {
    return a + b;
}

// Export public functions
export { greet, add };
"#;
        fs::write(project_path.join("src").join("lib.ai"), lib_content)?;

        // Create example usage
        let example_content = r#"// Example usage of the library
import { greet, add } from "../src/lib.ai";

function main() {
    print(greet("Aeonmi"));
    print("2 + 3 =", add(2, 3));
}

main();
"#;
        fs::write(
            project_path.join("examples").join("basic.ai"),
            example_content,
        )?;
    } else {
        // Create a simple main.ai
        let main_content = r#"# Main entry point for Aeonmi application

fn main:
    print "Hello, Aeonmi World!"
    print "Welcome to your new project!"
"#;
        fs::write(project_path.join("src").join("main.ai"), main_content)?;
    }

    // Create a basic test
    let test_content = r#"# Basic tests for the project

test arithmetic:
    let x = 4
    assert x == 4
    print "Arithmetic tests passed"

test variables:
    let x = 10
    let y = 5
    print "Variable tests passed"
"#;
    fs::write(project_path.join("tests").join("basic.ai"), test_content)?;

    Ok(())
}

/// Create quantum computing template
fn create_quantum_template(project_path: &Path, _name: &str, is_lib: bool) -> Result<()> {
    if is_lib {
        let lib_content = r#"// Quantum computing library
import { QuantumCircuit, Qubit, ClassicalBit } from "aeonmi:quantum";

/// Create a simple Bell state circuit
export function create_bell_state(): QuantumCircuit {
    let circuit = new QuantumCircuit(2, 2);
    
    // Create superposition on first qubit
    circuit.h(0);
    
    // Entangle qubits
    circuit.cnot(0, 1);
    
    // Measure both qubits
    circuit.measure(0, 0);
    circuit.measure(1, 1);
    
    return circuit;
}

/// Run quantum teleportation protocol
export function quantum_teleportation(message_qubit: Qubit): QuantumCircuit {
    let circuit = new QuantumCircuit(3, 3);
    
    // Create Bell pair (qubits 1 and 2)
    circuit.h(1);
    circuit.cnot(1, 2);
    
    // Teleportation protocol
    circuit.cnot(0, 1);
    circuit.h(0);
    
    // Measure and conditional operations
    circuit.measure(0, 0);
    circuit.measure(1, 1);
    
    circuit.conditional_x(2, 1);
    circuit.conditional_z(2, 0);
    
    return circuit;
}
"#;
        fs::write(project_path.join("src").join("lib.ai"), lib_content)?;
    } else {
        let main_content = r#"# Quantum computing application with Aeonmi

fn main:
    print "Quantum Computing with Aeonmi"
    
    # Create a simple quantum circuit
    let circuit = create_superposition_circuit
    
    print "Quantum circuit created successfully"
    print "Circuit creates superposition: (|0> + |1>) / sqrt(2)"

fn create_superposition_circuit:
    # Placeholder for quantum circuit creation
    # Full quantum backend integration coming soon
    print "Creating quantum circuit..."
    return "quantum_circuit_placeholder"
"#;
        fs::write(project_path.join("src").join("main.ai"), main_content)?;
    }

    // Create quantum test
    let test_content = r#"// Quantum computing tests
import { assert, test } from "aeonmi:test";
import { QuantumCircuit, QuantumBackend } from "aeonmi:quantum";

test("quantum superposition", () => {
    let circuit = new QuantumCircuit(1, 1);
    circuit.h(0);  // Hadamard gate
    circuit.measure(0, 0);
    
    let backend = QuantumBackend.get_simulator();
    let job = backend.run(circuit, shots: 100);
    let results = job.get_results();
    let counts = results.get_counts();
    
    // In superposition, we should see both |0> and |1> outcomes
    assert.true(counts.has("0"));
    assert.true(counts.has("1"));
});

test("quantum entanglement", () => {
    let circuit = new QuantumCircuit(2, 2);
    circuit.h(0);        // Superposition
    circuit.cnot(0, 1);  // Entanglement
    circuit.measure_all();
    
    let backend = QuantumBackend.get_simulator();
    let job = backend.run(circuit, shots: 100);
    let results = job.get_results();
    let counts = results.get_counts();
    
    // Bell state: should only see |00> and |11>
    for (let outcome in counts.keys()) {
        assert.true(outcome === "00" || outcome === "11");
    }
});
"#;
    fs::write(project_path.join("tests").join("quantum.ai"), test_content)?;

    Ok(())
}

/// Create AI/ML template
fn create_ai_template(project_path: &Path, _name: &str, is_lib: bool) -> Result<()> {
    if is_lib {
        let lib_content = r#"// AI/Machine Learning library
import { NeuralNetwork, Dataset, Optimizer } from "aeonmi:ai";

/// Simple neural network for classification
export class SimpleClassifier {
    private network: NeuralNetwork;
    
    constructor(input_size: number, hidden_size: number, num_classes: number) {
        this.network = new NeuralNetwork([
            input_size,
            hidden_size,
            num_classes
        ]);
    }
    
    train(dataset: Dataset, epochs: number = 100): void {
        let optimizer = new Optimizer.adam(learning_rate: 0.001);
        
        for (let epoch = 0; epoch < epochs; epoch++) {
            let total_loss = 0.0;
            
            for (let batch in dataset.batches(32)) {
                let predictions = this.network.forward(batch.inputs);
                let loss = this.network.compute_loss(predictions, batch.targets);
                
                this.network.backward();
                optimizer.update(this.network.parameters());
                
                total_loss += loss;
            }
            
            if (epoch % 10 === 0) {
                print(`Epoch ${epoch}: Loss = ${total_loss / dataset.size()}`);
            }
        }
    }
    
    predict(input: Array<number>): Array<number> {
        return this.network.forward(input);
    }
}

/// Generate synthetic dataset for testing
export function generate_synthetic_data(samples: number): Dataset {
    let inputs = [];
    let targets = [];
    
    for (let i = 0; i < samples; i++) {
        let x = Math.random() * 10 - 5;  // [-5, 5]
        let y = Math.random() * 10 - 5;  // [-5, 5]
        
        inputs.push([x, y]);
        
        // Simple classification: positive if x^2 + y^2 > 9
        let label = (x * x + y * y > 9) ? 1 : 0;
        targets.push([label]);
    }
    
    return new Dataset(inputs, targets);
}
"#;
        fs::write(project_path.join("src").join("lib.ai"), lib_content)?;
    } else {
        let main_content = r#"# AI/Machine Learning application with Aeonmi

fn main:
    print "AI/ML with Aeonmi"
    
    # Placeholder for AI functionality
    print "Neural network training capabilities coming soon"
    
    # Simple demonstration
    let result = simple_calculation 2 3
    print "Calculation result:" result

fn simple_calculation a b:
    return a + b
"#;
        fs::write(project_path.join("src").join("main.ai"), main_content)?;
    }

    // Create AI test
    let test_content = r#"// AI/ML tests
import { assert, test } from "aeonmi:test";
import { SimpleClassifier, generate_synthetic_data } from "../src/lib.ai";

test("neural network creation", () => {
    let classifier = new SimpleClassifier(2, 5, 1);
    assert.not_null(classifier);
});

test("dataset generation", () => {
    let dataset = generate_synthetic_data(100);
    assert.equal(dataset.size(), 100);
    
    let sample = dataset.get(0);
    assert.equal(sample.input.length, 2);
    assert.equal(sample.target.length, 1);
});

test("training convergence", () => {
    let dataset = generate_synthetic_data(50);
    let classifier = new SimpleClassifier(2, 5, 1);
    
    // Training should not throw errors
    classifier.train(dataset, epochs: 5);
    
    // Should be able to make predictions
    let prediction = classifier.predict([1.0, 1.0]);
    assert.equal(prediction.length, 1);
    assert.true(prediction[0] >= 0.0 && prediction[0] <= 1.0);
});
"#;
    fs::write(project_path.join("tests").join("ai.ai"), test_content)?;

    Ok(())
}

/// Create web application template
fn create_web_template(project_path: &Path, _name: &str, is_lib: bool) -> Result<()> {
    // Create web-specific directories
    fs::create_dir_all(project_path.join("static"))?;
    fs::create_dir_all(project_path.join("templates"))?;

    if is_lib {
        let lib_content = r#"// Web framework library
import { HttpServer, Request, Response, Router } from "aeonmi:web";

export class WebApp {
    private server: HttpServer;
    private router: Router;
    
    constructor(port: number = 8080) {
        this.server = new HttpServer(port);
        this.router = new Router();
        this.setup_routes();
    }
    
    private setup_routes(): void {
        this.router.get("/", (req: Request, res: Response) => {
            res.json({ message: "Welcome to Aeonmi Web!" });
        });
        
        this.router.get("/api/health", (req: Request, res: Response) => {
            res.json({ status: "healthy", timestamp: Date.now() });
        });
    }
    
    add_route(method: string, path: string, handler: Function): void {
        this.router.add(method, path, handler);
    }
    
    async start(): Promise<void> {
        this.server.use(this.router);
        await this.server.listen();
        print(`Server running on http://localhost:${this.server.port}`);
    }
}
"#;
        fs::write(project_path.join("src").join("lib.ai"), lib_content)?;
    } else {
        let main_content = r#"# Web application with Aeonmi

fn main:
    print "Starting Aeonmi Web Application"
    
    # Web server functionality coming soon
    print "Server would run on http://localhost:3000"
    
    # Simple demonstration
    let port = 3000
    print "Configured port:" port

fn handle_request path:
    print "Handling request for:" path
    return "response"
"#;
        fs::write(project_path.join("src").join("main.ai"), main_content)?;
    }

    // Create HTML template
    let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Aeonmi Web App</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 800px; margin: 0 auto; }
        .header { text-align: center; margin-bottom: 30px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to Aeonmi Web</h1>
            <p>Your quantum-powered web application is running!</p>
        </div>
        
        <div id="app">
            <!-- Dynamic content will be loaded here -->
        </div>
    </div>
    
    <script>
        // Simple frontend JavaScript
        fetch('/api/health')
            .then(response => response.json())
            .then(data => {
                document.getElementById('app').innerHTML = 
                    `<p>Server Status: ${data.status}</p>`;
            });
    </script>
</body>
</html>
"#;
    fs::write(project_path.join("static").join("index.html"), html_content)?;

    Ok(())
}

/// Create CLI application template
fn create_cli_template(project_path: &Path, name: &str, is_lib: bool) -> Result<()> {
    if is_lib {
        let lib_content = format!(
            r#"// CLI library for {}
import {{ Args, Command, Flag }} from "aeonmi:cli";

export class CliApp {{
    private args: Args;
    private commands: Map<string, Command>;
    
    constructor() {{
        this.args = new Args();
        this.commands = new Map();
        this.setup_commands();
    }}
    
    private setup_commands(): void {{
        this.commands.set("greet", new Command({{
            name: "greet",
            description: "Greet someone",
            flags: [
                new Flag("name", "Name to greet", required: true),
                new Flag("loud", "Use uppercase", type: "boolean")
            ],
            handler: this.greet_command
        }}));
        
        this.commands.set("count", new Command({{
            name: "count", 
            description: "Count to a number",
            flags: [
                new Flag("to", "Number to count to", type: "number", default: 10)
            ],
            handler: this.count_command
        }}));
    }}
    
    private greet_command(args: Args): void {{
        let name = args.get("name");
        let loud = args.get("loud", false);
        
        let greeting = `Hello, ${{name}}!`;
        if (loud) {{
            greeting = greeting.toUpperCase();
        }}
        
        print(greeting);
    }}
    
    private count_command(args: Args): void {{
        let to = args.get("to", 10);
        
        for (let i = 1; i <= to; i++) {{
            print(i);
        }}
    }}
    
    run(argv: Array<string>): number {{
        try {{
            let parsed = this.args.parse(argv);
            let command_name = parsed.command || "help";
            
            if (command_name === "help") {{
                this.show_help();
                return 0;
            }}
            
            let command = this.commands.get(command_name);
            if (!command) {{
                print(`Unknown command: ${{command_name}}`);
                this.show_help();
                return 1;
            }}
            
            command.handler(parsed);
            return 0;
            
        }} catch (error) {{
            print(`Error: ${{error.message}}`);
            return 1;
        }}
    }}
    
    private show_help(): void {{
        print("{} - Command line tool", "{{}}");
        print("");
        print("USAGE:");
        print("  {} <COMMAND> [OPTIONS]", "{{}}");
        print("");
        print("COMMANDS:");
        
        for (let [name, command] of this.commands) {{
            print("  {{:<12}} {{}}", name, command.description);
        }}
        
        print("");
        print("Use '{} <COMMAND> --help' for more information about a command.", "{{}}");
    }}
}}
"#,
            name, name, name, name
        );
        fs::write(project_path.join("src").join("lib.ai"), lib_content)?;
    } else {
        let main_content = r#"# CLI application with Aeonmi

fn main:
    print "CLI Application Running"
    
    # Command-line argument handling coming soon
    print "Arguments would be processed here"
    
    # Simple demonstration
    let status = run_command
    print "Command completed with status:" status
    return 0

fn run_command:
    print "Executing command..."
    return 0
"#;
        fs::write(project_path.join("src").join("main.ai"), main_content)?;
    }

    Ok(())
}

/// Create README.md
fn create_readme(project_path: &Path, name: &str, template: &str) -> Result<()> {
    let readme_content = format!(r#"# {}

A new Aeonmi project created with the '{}' template.

## Description

This project was scaffolded using the Aeonmi quantum programming language toolchain.

## Getting Started

### Prerequisites

- Aeonmi toolchain installed
- Quantum simulator (for quantum features)

### Building

```bash
# Build the project
aeon build

# Build in release mode
aeon build --release
```

### Running

```bash
# Run the project
aeon run

# Run with specific arguments
aeon run -- arg1 arg2
```

### Testing

```bash
# Run all tests
aeon test

# Run specific test
aeon test --filter test_name
```

### Development

```bash
# Check syntax and semantics
aeon check

# Format source code
aeon format

# Lint source code
aeon lint
```

## Project Structure

```
{}{}
├── Aeonmi.toml          # Project manifest
├── src/                 # Source code
│   └── main.ai         # Main entry point{}
├── tests/              # Test files
├── examples/           # Example code
├── docs/               # Documentation
└── README.md           # This file
```

## Features

{}

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Learn More

- [Aeonmi Documentation](https://aeonmi.dev/docs)
- [Quantum Computing Guide](https://aeonmi.dev/quantum)
- [API Reference](https://aeonmi.dev/api)
"#,
    name,
    template,
    name,
    if template == "web" { "\n├── static/             # Static web assets\n├── templates/          # HTML templates" } else { "" },
    if template != "basic" { "\n│   └── lib.ai          # Library code" } else { "" },
    match template {
        "quantum" => "- Quantum circuit simulation\n- Quantum algorithm implementations\n- Integration with quantum backends",
        "ai" => "- Neural network training\n- Machine learning algorithms\n- AI model deployment",
        "web" => "- HTTP server with routing\n- RESTful API endpoints\n- Static file serving",
        "cli" => "- Command-line argument parsing\n- Subcommand support\n- Help generation",
        _ => "- Basic Aeonmi functionality\n- Example implementations\n- Test coverage"
    }
);

    fs::write(project_path.join("README.md"), readme_content)?;
    Ok(())
}

/// Create .gitignore
fn create_gitignore(project_path: &Path) -> Result<()> {
    let gitignore_content = r#"# Build artifacts
/target/
*.bc
*.js
*.wasm

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Environment
.env
.env.local

# Logs
*.log
logs/

# Dependencies
node_modules/
.pnp/

# Coverage reports
coverage/
*.lcov

# Temporary files
*.tmp
*.temp
temp/
"#;

    fs::write(project_path.join(".gitignore"), gitignore_content)?;
    Ok(())
}

/// Create LICENSE file
fn create_license(project_path: &Path) -> Result<()> {
    let license_content = r#"MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

    fs::write(project_path.join("LICENSE"), license_content)?;
    Ok(())
}

/// Initialize git repository
fn initialize_git_repo(project_path: &Path) -> Result<()> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&["init"])
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        bail!(
            "Failed to initialize git repository: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Create initial commit
    Command::new("git")
        .args(&["add", "."])
        .current_dir(project_path)
        .output()?;

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(project_path)
        .output()?;

    print_info("Initialized git repository");
    Ok(())
}

/// Open project in editor
fn open_in_editor(project_path: &Path) -> Result<()> {
    use std::process::Command;

    // Try common editors
    let editors = ["code", "atom", "subl", "vim", "nano"];

    for editor in &editors {
        if let Ok(output) = Command::new(editor).arg(project_path).output() {
            if output.status.success() {
                print_info(&format!("Opened project in {}", editor));
                return Ok(());
            }
        }
    }

    print_info("Could not find a suitable editor to open the project");
    Ok(())
}

/// Check if directory is suitable for initialization
fn check_directory_for_init(dir: &Path) -> Result<()> {
    let entries: Result<Vec<_>, _> = fs::read_dir(dir)?.collect();
    let entries = entries?;

    // Allow empty directory
    if entries.is_empty() {
        return Ok(());
    }

    // Check for conflicting files
    let safe_files = [
        "README.md",
        "LICENSE",
        ".gitignore",
        ".git",
        "docs",
        "documentation",
        ".vscode",
        ".idea",
    ];

    for entry in entries {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if !safe_files
            .iter()
            .any(|&safe| file_name_str.starts_with(safe))
        {
            bail!(
                "Directory contains files that might conflict with project initialization. \
                   Consider using 'aeon new <project_name>' instead."
            );
        }
    }

    Ok(())
}
