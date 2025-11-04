use aeonmi_project::core::lexer::Lexer;

fn main() {
    println!("Testing |psi>");
    let mut lx = Lexer::from_str("|psi>");
    match lx.tokenize() {
        Ok(tokens) => println!("Tokens: {:?}", tokens),
        Err(err) => println!("Error: {}", err),
    }
    
    println!("\nTesting |abc>");
    let mut lx2 = Lexer::from_str("|abc>");
    match lx2.tokenize() {
        Ok(tokens) => println!("Tokens: {:?}", tokens),
        Err(err) => println!("Error: {}", err),
    }
    
    println!("\nTesting |0>");
    let mut lx3 = Lexer::from_str("|0>");
    match lx3.tokenize() {
        Ok(tokens) => println!("Tokens: {:?}", tokens),
        Err(err) => println!("Error: {}", err),
    }
}
