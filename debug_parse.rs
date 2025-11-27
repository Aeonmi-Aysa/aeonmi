use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser as AeParser;
use aeonmi_project::core::code_actions::suggest_actions;

fn main() {
    let src = "x = 1\n";
    let mut lex = Lexer::from_str(src);
    let toks = lex.tokenize().unwrap();
    println!("Tokens: {:?}", toks);

    let mut p = AeParser::new(toks);
    let ast = p.parse().unwrap();
    println!("AST: {:#?}", ast);

    let acts = suggest_actions(&ast);
    println!("Code actions: {:?}", acts);
}