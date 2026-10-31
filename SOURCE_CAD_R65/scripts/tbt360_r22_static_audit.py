from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
files={k:(ROOT/v).read_text() for k,v in {
 "palette":"crates/ui-egui/src/palettes.rs",
 "manager":"crates/ui-egui/src/managers.rs",
 "draw":"crates/engine/src/cmd/draw.rs",
 "inquiry":"crates/engine/src/cmd/inquiry.rs",
 "table":"crates/engine/src/cmd/table.rs",
 "tablecells":"crates/engine/src/cmd/table/cells.rs",
 "hatch":"crates/engine/src/cmd/hatch.rs",
 "draworder":"crates/engine/src/cmd/draworder.rs",
 "props":"crates/engine/src/cmd/props.rs",
}.items()}
checks=[]
def ck(name,cond): checks.append((name,bool(cond)))
# 2D UI depth
for label in ["Tables & Fields","Hatch & Draw Order","Geometry Cleanup","Layer & Properties","Inspect & Measure"]:
    ck("UI group "+label, f'"{label}"' in files["palette"])
for cmd in ["table","tablestyle","tabledit","table.recalc","table.formula","tableexport","dataextraction","field","updatefield","hatch","hatchedit","boundary","hatchtoback","region","hatch.patterns"]:
    ck("2D UI command "+cmd, f'"{cmd}"' in files["palette"])
# verify underlying commands really exist
for cmd,src in [
    ("table", "table"),("tabledit","tablecells"),("table.recalc","tablecells"),("table.formula","tablecells"),("tableexport","tablecells"),
    ("hatch","hatch"),("hatchedit","hatch"),("hatchtoback","draworder"),("tablestyle","props")]:
    ck("engine 2D "+cmd, f'CommandSpec::new("{cmd}"' in files[src])
# 3D point cloud transform
for cmd in ["pointcloud.translate","pointcloud.scale","facetflip3d"]:
    ck("engine spec "+cmd, f'CommandSpec::new("{cmd}"' in files["draw"])
ck("pointcloud shared point lookup","fn pointcloud_points" in files["draw"])
ck("pointcloud translate function","fn run_pointcloud_translate" in files["draw"] and 'delta' in files["draw"])
ck("pointcloud scale function","fn run_pointcloud_scale" in files["draw"] and 'centroid' in files["draw"])
ck("pointcloud transform manager",'Translate' in files["manager"] and 'Scale' in files["manager"] and 'pointcloud.translate' in files["manager"] and 'pointcloud.scale' in files["manager"])
ck("pointcloud transform manager state",'move_delta' in files["manager"] and 'scale_factor' in files["manager"])
ck("pointcloud transform regression","pointcloud_translate_and_scale_preserve_group_points" in files["draw"])
# facet orientation repair
ck("facet flip function","fn run_facetflip3d" in files["draw"])
ck("facet flip triangle-safe",'(d - c).len() <= 1e-12' in files["draw"] and '[a, c, b, b]' in files["draw"])
ck("facet flip UI",'"facetflip3d"' in files["palette"] and '3D Inspect & Repair' in files["palette"])
ck("facet flip regression","facet_flip_reverses_face_normal" in files["draw"])
# capability truthfulness
ck("capability cloud transform truth","persistent translate/scale" in files["inquiry"])
ck("capability facet repair truth","facet orientation repair" in files["inquiry"])
ck("ACIS remains false",'"acis": false' in files["inquiry"])
failed=[n for n,v in checks if not v]
for n,v in checks: print(("PASS" if v else "FAIL")+"  "+n)
print(f"\nTOTAL {sum(v for _,v in checks)}/{len(checks)} PASS")
if failed:
    print("FAILED: "+", ".join(failed)); raise SystemExit(1)
