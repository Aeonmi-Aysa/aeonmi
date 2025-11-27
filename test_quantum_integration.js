const __aeonmi_len = (value) => {
    if (typeof value === "string") { return value.length; }
    if (Array.isArray(value)) { return value.length; }
    if (value && typeof value === "object") { return Object.keys(value).length; }
    if (value === null || value === undefined) { return 0; }
    throw new Error("len: unsupported type");
};
let q1 = "|0⟩"; // Classical quantum variable
superpose(q1);
let result1 = measure(q1);
console.log("Test 1 - Basic qubit:");
console.log(result1);
let zero_state = "|0⟩";
let one_state = "|1⟩";
let plus_state = "|+⟩";
console.log("Test 2 - Zero state:");
console.log(zero_state);
console.log("Test 2 - One state:");
console.log(one_state);
console.log("Test 2 - Plus state:");
console.log(plus_state);
let qbit_a = "|0⟩"; // Classical quantum variable
let qbit_b = "|1⟩"; // Superposition quantum variable
console.log("Test 3 - Hieroglyphic vars:");
console.log(qbit_a);
console.log(qbit_b);
let q2 = "|0⟩"; // Classical quantum variable
superpose(q2);
let entangled = entangle(q2, q1);
console.log("Test 4 - Entanglement result:");
console.log(entangled);
let qarray = ["|0⟩", "|1⟩", "|+⟩"];
console.log("Test 5 - Quantum array length:");
console.log(__aeonmi_len(qarray));
console.log("Test 5 - First element:");
console.log(qarray[0]);
console.log("Quantum Integration Test Complete!");
