import json
def h(x): return [int(x[i:i+2],16)/255 for i in (0,2,4)]
def lin(v): return v/12.92 if v<=0.04045 else ((v+0.055)/1.055)**2.4
def lum(c): return 0.2126*lin(c[0])+0.7152*lin(c[1])+0.0722*lin(c[2])
def blend(b,o,a): return [b[i]*(1-a)+o[i]*a for i in range(3)]
def pin(c,t):
    darker=lum(c)>t; to=[0,0,0] if darker else [1,1,1]; lo,hi=0.0,1.0
    for _ in range(32):
        m=(lo+hi)/2; l=lum(blend(c,to,m)); short = l>t if darker else l<t
        if short: lo=m
        else: hi=m
    return blend(c,to,(lo+hi)/2)
def hx(c): return "#%02X%02X%02X"%tuple(round(v*255) for v in c)
wash={'Light':{'cobalt':'A3B6FF','violet':'C5ADFF','fuchsia':'FF9ED9','amber':'FFD58C','terracotta':'FFB18C','slate':'C6DEFF','pine':'8CFFDE','plum':'DCA6FF','ink':'B8B8B8'},
'Dark':{'cobalt':'143695','violet':'3E289F','fuchsia':'871959','amber':'4F3600','terracotta':'712A06','slate':'303843','pine':'02462F','plum':'5C2479','ink':'373737'}}
ident={'Light':{'cobalt':'4062E8','violet':'7C4DF0','fuchsia':'C63B8F','amber':'B57912','terracotta':'B4582C','slate':'5B6470','pine':'0E6E52','plum':'6E3A8F','ink':'6A6A6A'},
'Dark':{'cobalt':'6E8BFF','violet':'8A63F0','fuchsia':'B02E7E','amber':'C0861F','terracotta':'B85A30','slate':'8B94A2','pine':'2FBF8F','plum':'9A63C9','ink':'8A8A8A'}}
base={'Light':'E5E5E2','Dark':'1C1E22'}
steps={'Light':{'subtle':.18,'standard':.36,'rich':.54},'Dark':{'subtle':.12,'standard':.24,'rich':.36}}
out={}
for m in ('Light','Dark'):
  b=h(base[m])
  for t in wash[m]:
    for s,a in steps[m].items():
      out[f"{m}-{t}-{s}"]=hx(pin(blend(b,h(wash[m][t]),a),lum(b)))
print(json.dumps(out,indent=0))
json.dump({'rail':out,'ident':ident},open('/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/tint.json','w'))
