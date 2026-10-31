from pathlib import Path
root=Path(__file__).resolve().parents[1]
files={n:(root/n).read_text(errors="ignore") for n in ["crates/ui-egui/src/menus.rs","crates/ui-egui/src/chrome.rs","crates/ui-egui/src/palettes.rs","crates/ui-egui/src/i18n/vi.tsv"]}
m=files["crates/ui-egui/src/menus.rs"]; c=files["crates/ui-egui/src/chrome.rs"]; p=files["crates/ui-egui/src/palettes.rs"]; v=files["crates/ui-egui/src/i18n/vi.tsv"]
checks=[]
def ck(name,cond): checks.append((name,bool(cond)))
for ident in ["ui.demo.2d.section","ui.demo.3d.structuremep","ui.showcase.2d.section","ui.showcase.3d.structuremep"]: ck("menu "+ident, ident in m)
ck("2D handler", 'if id == "ui.demo.2d.section"' in m)
ck("3D handler", 'if id == "ui.demo.3d.structuremep"' in m)
ck("2D hatch evidence", '("hatch", json!' in m and '"evidence":"section"' in m)
ck("3D structure", '"evidence":"structure-mep"' in m and '"systems":["structure","hvac","plumbing"]' in m)
ck("toolbar 2D", '"ui.demo.2d.section", "2D Section Demo"' in c)
ck("toolbar 3D", '"ui.demo.3d.structuremep", "3D Structure + MEP"' in c)
ck("toolset 2D", '"ui.demo.2d.section", "2D Section Demo"' in p)
ck("toolset 3D", '"ui.demo.3d.structuremep", "3D Structure + MEP Demo"' in p)
ck("2D test", 'section_demo_creates_hatch_dimension_and_annotation' in m)
ck("3D test", 'structure_mep_demo_creates_visible_3d_systems' in m)
ck("vi 2D", 'Create 2D Section Demo' in v)
ck("vi 3D", 'Create 3D Structure + MEP Demo' in v)
failed=[n for n,ok in checks if not ok]
for n,ok in checks: print(("PASS" if ok else "FAIL")+" | "+n)
print(f"TOTAL {sum(ok for _,ok in checks)}/{len(checks)}")
raise SystemExit(1 if failed else 0)
