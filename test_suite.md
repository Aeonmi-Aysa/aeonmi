# Show version
aeonmi --version

# Test 1: Hello World
aeonmi run examples/hello.ai

# Test 2: Quantum operations
aeonmi run examples/quantum.ai

# Test 3: Shard bootstrap compiler
aeonmi run shard/src/main.ai

# Test 4: Full Shard with all syntax
aeonmi run shard/src/main_full.ai

# Test 5: Quantum glyph rendering
aeonmi run examples/quantum_glyph.ai

# Test 6: Closures
aeonmi run examples/closures.ai

# Test 7: Control flow
aeonmi run examples/control_flow.ai

# Test 8: QUBE quantum circuits
aeonmi qube run examples/demo.qube

# Test 9: Vault init
aeonmi vault init

# Test 10: NFT minting
aeonmi mint examples/hello.ai

# Test 11: Full Rust test suite
cargo test --release --lib -- --test-threads=1

# Test 12: Show the Shard source is pure .ai
type shard\src\main.ai
type shard\src\main_full.ai
type shard\src\lexer.ai