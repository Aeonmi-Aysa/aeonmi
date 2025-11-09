use aeonmi_project::core::compiler::Compiler;

#[test]
fn quantum_and_hieroglyphic_ops() {
    let code = r#"
        ⟨q1⟩ ← |0⟩
        ⟨q2⟩ ← |0⟩
        superpose(⟨q1⟩);
        entangle(⟨q1⟩, ⟨q2⟩);
        𓀀(⟨q1⟩, 42);
        measure(⟨q1⟩);
    "#;

    let out = std::env::temp_dir().join("aeonmi_qglyph_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap())
        .expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");
    assert!(js.contains("superpose"));
    assert!(js.contains("entangle"));
    assert!(js.contains("__glyph('𓀀'"));
    assert!(js.contains("measure"));
}
