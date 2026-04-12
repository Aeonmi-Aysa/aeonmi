//! Error Memory Demo — Shows how Mother learns from mistakes

use aeonmi::mother::ErrorMemory;

fn main() {
    println!("=== ERROR MEMORY DEMO ===/n");
    
    // Create error memory database
    let mut em = ErrorMemory::new("mother_errors.db")
        .expect("Failed to create error memory");
    
    println!("1. Logging some errors.../n");
    
    // Simulate some common errors
    em.log_error("parsing", "unexpected token '}'").unwrap();
    em.log_error("parsing", "unexpected token '}'").unwrap(); // Same error again
    em.log_error("parsing", "unexpected token '