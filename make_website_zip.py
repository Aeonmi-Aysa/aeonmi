import zipfile
from pathlib import Path

zip_path = Path("website_deploy.zip")
website_dir = Path("Aeonmi_Master/website")

print(f"Creating {zip_path}...")

with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
    for file_path in website_dir.rglob('*'):
        if file_path.is_file():
            arcname = file_path.relative_to(website_dir)
            print(f"  Adding: {arcname}")
            zf.write(file_path, arcname)

print(f"/nCreated {zip_path}")
print(f"Size: {zip_path.stat().st_size / 1024:.1f} KB")