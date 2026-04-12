import shutil
import zipfile
from pathlib import Path

print("=== Building Starter Kit v1.1 ===/n")

# 1. Create clean directory
deploy = Path("deploy_v1.1")
if deploy.exists():
    shutil.rmtree(deploy)
deploy.mkdir()

print("✓ Created deploy directory")

# 2. Copy existing starter files
src = Path("website upgdates & starter kits/starter")
if src.exists():
    shutil.copytree(src, deploy / "starter")
    print(f"✓ Copied {len(list((deploy/'starter').glob('*')))} files from starter")
else:
    print("✗ Source starter not found, creating from scratch")
    (deploy / "starter").mkdir()
    
    # Copy from starter_kit instead
    for f in Path("starter_kit").glob("*.ai"):
        shutil.copy(f, deploy / "starter")
    for f in Path("starter_kit").glob("*.md"):
        shutil.copy(f, deploy / "starter")
    shutil.copy("starter_kit/run.bat", deploy / "starter")
    shutil.copy("target/release/aeonmi.exe", deploy / "starter")
    print(f"✓ Created starter from starter_kit")

# 3. Create MGKS directory
mgks_dir = deploy / "starter" / "mgks"
mgks_dir.mkdir(exist_ok=True)

# 4. Copy MGKS files
mgks_src = Path("uploads/mgks_extracted")
if mgks_src.exists():
    for f in mgks_src.glob("*.ai"):
        shutil.copy(f, mgks_dir)
    if (mgks_src / "README.md").exists():
        shutil.copy(mgks_src / "README.md", mgks_dir)
    print(f"✓ Copied {len(list(mgks_dir.glob('*.ai')))} MGKS files")
else:
    print("✗ MGKS source not found")

# 5. Create simple MGKS demo
demo_content = '''⍝ MGKS Simple Demo

function main() {
    print("=== MGKS System ===");
    print("Glyph-based memory architecture");
    print("Status: Implemented in Aeonmi");
    print("");
    print("Features:");
    print("  • High-density semantic compression");
    print("  • Multi-layer memory (episodic, semantic, procedural)");
    print("  • Agent hive coordination");
    print("  • Quantum-inspired retrieval");
    print("");
    print("See other .ai files in mgks/ folder");
    return 0;
}

main();'''

(mgks_dir / "demo.ai").write_text(demo_content)
print("✓ Created demo.ai")

# 6. Create ZIP
zip_path = Path("deploy_v1.1/aeonmi_starter_kit_v1.1.zip")
with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
    for file in (deploy / "starter").rglob("*"):
        if file.is_file():
            arcname = file.relative_to(deploy / "starter")
            zf.write(file, arcname)
            
print(f"✓ Created ZIP: {zip_path}")

# 7. Verify ZIP contents
with zipfile.ZipFile(zip_path, 'r') as zf:
    files = zf.namelist()
    print(f"/n=== ZIP Contents ({len(files)} files) ===")
    for f in sorted(files):
        info = zf.getinfo(f)
        print(f"  {f} ({info.file_size} bytes)")

# 8. Test extraction
test_dir = Path("test_extract_v1.1")
if test_dir.exists():
    shutil.rmtree(test_dir)
test_dir.mkdir()

with zipfile.ZipFile(zip_path, 'r') as zf:
    zf.extractall(test_dir)
    
print(f"/n✓ Extracted to {test_dir}")

# 9. Copy to final location
final_dir = Path("website upgdates & starter kits")
final_path = final_dir / "aeonmi_starter_kit_v1.1_COMPLETE.zip"
shutil.copy(zip_path, final_path)
print(f"/n✓ Final package: {final_path}")
print(f"  Size: {final_path.stat().st_size / 1024 / 1024:.2f} MB")

print("/n=== BUILD COMPLETE ===")