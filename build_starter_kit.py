import zipfile  
from pathlib import Path  
  
print("Building...")  
output = Path("aeonmi_starter_kit.zip")  
  
with zipfile.ZipFile(output, 'w') as zf:  
    zf.write("starter_kit/hello_quantum.ai", "aeonmi_starter_kit/hello_quantum.ai")  
    zf.write("starter_kit/grover_search.ai", "aeonmi_starter_kit/grover_search.ai")  
    zf.write("starter_kit/qft_pattern.ai", "aeonmi_starter_kit/qft_pattern.ai")  
    zf.write("starter_kit/entanglement_demo.ai", "aeonmi_starter_kit/entanglement_demo.ai")  
    zf.write("starter_kit/run.bat", "aeonmi_starter_kit/run.bat")  
    zf.write("starter_kit/README.md", "aeonmi_starter_kit/README.md")  
    zf.write("starter_kit/PRICING.md", "aeonmi_starter_kit/PRICING.md")  
    zf.write(r"C:/RustTarget/release/aeonmi.exe", "aeonmi_starter_kit/aeonmi.exe")  
  
print("SUCCESS!") 
