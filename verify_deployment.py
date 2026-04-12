#!/usr/bin/env python3
"""Verify all deployment files are ready"""

import os
import zipfile
from pathlib import Path

def check_file(path, min_size_kb=None):
    """Check if file exists and meets size requirement"""
    p = Path(path)
    if not p.exists():
        return False, f"❌ Missing: {path}"
    
    size_kb = p.stat().st_size / 1024
    if min_size_kb and size_kb < min_size_kb:
        return False, f"❌ Too small: {path} ({size_kb:.1f} KB < {min_size_kb} KB)"
    
    return True, f"✅ Found: {path} ({size_kb:.1f} KB)"

def check_zip_contents(zip_path, expected_files):
    """Verify zip contains expected files"""
    try:
        with zipfile.ZipFile(zip_path, 'r') as zf:
            contents = zf.namelist()
            missing = [f for f in expected_files if f not in contents