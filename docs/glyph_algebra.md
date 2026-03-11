\# Aeonmi Glyph Algebra



Aeonmi programs are composed through glyph algebra.



Rather than traditional syntax trees, Aeonmi expressions

form symbolic graphs.



Core glyphs:



⧉   collection

⟨⟩   slice / projection

…   expansion

⊗   tensor combination

↦   symbolic binding



These operators allow compact representation of

large mathematical state spaces.



Example:



bell ← ⧉0.707‥0‥0‥0.707⧉



ψ ↦ bell ⊗ bell



ψ is symbolically bound to the tensor composition

without immediately expanding it.



This enables the QUBE optimizer to compress or

transform symbolic structures before evaluation.

