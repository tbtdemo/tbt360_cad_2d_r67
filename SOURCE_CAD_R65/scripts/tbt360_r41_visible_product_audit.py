from pathlib import Path
root=Path(__file__).resolve().parents[1]
files={n:(root/n).read_text(errors="ignore") for n in ["crates/ui-egui/src/menus.rs","crates/ui-egui/src/chrome.rs","crates/ui-egui/src/palettes.rs","crates/ui-egui/src/i18n/vi.tsv"]}
m=files["crates/ui-egui/src/menus.rs"]; c=files["crates/ui-egui/src/chrome.rs"]; p=files["crates/ui-egui/src/palettes.rs"]; v=files["crates/ui-egui/src/i18n/vi.tsv"]
checks=[]
def ck(name,cond): checks.append((name,bool(cond)))
for ident in ["ui.demo.2d.elevation","ui.demo.2d.mep","ui.demo.3d.site","ui.showcase.2d.elevation","ui.showcase.2d.mep","ui.showcase.3d.site"]: ck("menu "+ident, ident in m)
ck("elevation handler", 'if id == "ui.demo.2d.elevation"' in m)
ck("mep handler", 'if id == "ui.demo.2d.mep"' in m)
ck("site handler", 'if id == "ui.demo.3d.site"' in m)
ck("elevation evidence", '"evidence":"elevation"' in m and 'FRONT ELEVATION / MAT DUNG' in m)
ck("mep evidence", '"evidence":"mep-plan"' in m and 'MEP PLAN / MAT BANG HE THONG' in m)
ck("site evidence", '"evidence":"site-landscape"' in m and 'SITE + LANDSCAPE / CANH QUAN' in m)
ck("toolbar elevation", '"ui.demo.2d.elevation", "2D Elevation Demo"' in c)
ck("toolbar mep", '"ui.demo.2d.mep", "2D MEP Plan Demo"' in c)
ck("toolbar site", '"ui.demo.3d.site", "3D Site + Landscape"' in c)
ck("toolset elevation", '"ui.demo.2d.elevation", "2D Elevation Demo"' in p)
ck("toolset mep", '"ui.demo.2d.mep", "2D MEP Plan Demo"' in p)
ck("toolset site", '"ui.demo.3d.site", "3D Site + Landscape Demo"' in p)
ck("elevation test", 'elevation_demo_creates_visible_2d_facade_evidence' in m)
ck("mep test", 'mep_plan_demo_creates_visible_2d_systems' in m)
ck("site test", 'site_landscape_demo_creates_visible_3d_assets' in m)
ck("vi elevation", 'Create 2D Elevation Demo' in v)
ck("vi mep", 'Create 2D MEP Plan Demo' in v)
ck("vi site", 'Create 3D Site + Landscape Demo' in v)
failed=[n for n,ok in checks if not ok]
for n,ok in checks: print(("PASS" if ok else "FAIL")+" | "+n)
print(f"TOTAL {sum(ok for _,ok in checks)}/{len(checks)}")
raise SystemExit(1 if failed else 0)
