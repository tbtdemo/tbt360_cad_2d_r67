from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
menus=(ROOT/'crates/ui-egui/src/menus.rs').read_text()
chrome=(ROOT/'crates/ui-egui/src/chrome.rs').read_text()
pal=(ROOT/'crates/ui-egui/src/palettes.rs').read_text()
vi=(ROOT/'crates/ui-egui/src/i18n/vi.tsv').read_text()
checks=[]
def ck(name, cond): checks.append((name,bool(cond)))
items=[
 ('structural','ui.demo.2d.structural','ui.showcase.2d.structural','structural-plan','2D Structural Plan Demo','structural_plan_demo_creates_grid_columns_beams_and_dimensions'),
 ('rcp','ui.demo.2d.rcp','ui.showcase.2d.rcp','ceiling-lighting-plan','2D Ceiling + Lighting Plan Demo','ceiling_lighting_plan_demo_creates_visible_lighting_layout'),
 ('interior','ui.demo.3d.interior','ui.showcase.3d.interior','interior-lighting','3D Interior + Lighting Demo','interior_lighting_demo_creates_visible_3d_assets'),
]
for short,demo,show,evidence,label,test in items:
    ck(f'menu {demo}', demo in menus and label in menus)
    ck(f'menu {show}', show in menus)
    ck(f'handler {short}', f'if id == "{demo}"' in menus)
    ck(f'evidence {short}', f'"evidence":"{evidence}"' in menus)
    ck(f'toolbar {short}', demo in chrome)
    ck(f'toolset {short}', demo in pal)
    ck(f'test {short}', test in menus)
    ck(f'vi {short}', label in vi)
for name,ok in checks:
    print(('PASS' if ok else 'FAIL')+' | '+name)
print(f'TOTAL {sum(x for _,x in checks)}/{len(checks)}')
raise SystemExit(0 if all(x for _,x in checks) else 1)
