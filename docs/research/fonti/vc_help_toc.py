import json,re,sys,subprocess
B=sys.argv[1]; out=sys.argv[2]
def get(key):
    s=subprocess.run(["curl","-sS","-f",f"{B}/whxdata/{key}.new.js"],capture_output=True,text=True,check=True).stdout
    return json.loads(re.search(r'var toc\s*=\s*(\[.*\]);',s,re.S).group(1))
lines=[]
def walk(key,d):
    for e in get(key):
        if e["type"]=="book":
            lines.append("  "*d+"# "+e["name"]); 
            if "key" in e: walk(e["key"],d+1)
        else: lines.append("  "*d+"- "+e["name"]+" | "+e.get("url",""))
walk("toc",0); open(out,"w").write("\n".join(lines)); print(len(lines))
