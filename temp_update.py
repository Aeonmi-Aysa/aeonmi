import json 
p = "Aeonmi_Master/genesis.json" 
d = json.load(open(p)) 
open(p, "w").write(json.dumps(d, indent=2)) 
print("Updated") 
