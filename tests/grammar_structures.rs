use aeonmi_project::core::compiler::Compiler;

#[test]
fn parses_and_generates_composite_features() {
    let source = r#"
        struct Point { x = 1, y = 2 }

        class Greeter {
            function greet(name) {
                return "Hello, " + name;
            }
        }

        trait Presenter {
            function present() {
                return "presenting";
            }
        }

        impl Presenter for Greeter {
            function present() {
                return "Greeter";
            }
        }

        impl Greeter {
            function exclaim(message) {
                return message + "!";
            }
        }

        function main() {
            let result = match 3 {
                1 => 100,
                3 => 200,
                _ => 0
            };

            log(result);
        }
    "#;

    let output = std::env::temp_dir().join("aeonmi_composite_out.js");
    if output.exists() {
        std::fs::remove_file(&output).expect("remove stale output");
    }

    let compiler = Compiler::new();
    compiler
        .compile(source, output.to_str().expect("valid path"))
        .expect("compilation should succeed");

    let js = std::fs::read_to_string(&output).expect("generated JS");
    assert!(
        js.contains("function Point(init = {})"),
        "struct should emit constructor"
    );
    assert!(
        js.contains("class Greeter"),
        "class should emit class declaration"
    );
    assert!(
        js.contains("prototype.present"),
        "impl should attach methods to prototype"
    );
    assert!(
        js.contains("const __matchValue"),
        "match expression should lower to runtime dispatch"
    );
}
