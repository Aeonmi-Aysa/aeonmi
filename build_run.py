import subprocess, sys, os

repo = r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01"
cargo = r"C:\Users\wlwil\.cargo\bin\cargo.exe"
out_file = r"C:\Temp\build_out.txt"

result = subprocess.run(
    [cargo, "build", "--release"],
    cwd=repo,
    capture_output=True,
    text=True,
    encoding="utf-8",
    errors="replace"
)

combined = result.stdout + result.stderr
with open(out_file, "w", encoding="utf-8") as f:
    f.write(combined)
    f.write("\n--- EXIT CODE: {} ---\n".format(result.returncode))

# Print last 40 lines to stdout so we can see them
lines = combined.splitlines()
for line in lines[-40:]:
    print(line)
print("--- EXIT CODE:", result.returncode, "---")
