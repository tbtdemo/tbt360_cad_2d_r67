from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
files={k:(ROOT/v).read_text() for k,v in {
 "palette":"crates/ui-egui/src/palettes.rs",
 "manager":"crates/ui-egui/src/managers.rs",
 "draw":"crates/engine/src/cmd/draw.rs",
 "inquiry":"crates/engine/src/cmd/inquiry.rs",
}.items()}
checks=[]
def ck(name,cond): checks.append((name,bool(cond)))
for label in ["Layer & Properties","Draw Order & Clipboard","Construction & Boundaries","3D Inspect"]: ck("UI group "+label, f'"{label}"' in files["palette"])
for cmd in ["layiso","layuniso","layoff","layon","layerp","laymrg","laydel","matchprop","copybase","pasteorig","draworder.front","draworder.back","draworder.above","draworder.under","xline","ray","boundary","mline","wipeout","revcloud","ellipse","spline","massprops3d","facetcheck3d"]: ck("UI command "+cmd, f'"{cmd}"' in files["palette"])
for cmd in ["pointcloud.crop","massprops3d","facetcheck3d"]: ck("engine spec "+cmd, f'CommandSpec::new("{cmd}"' in files["draw"])
ck("pointcloud crop function","fn run_pointcloud_crop" in files["draw"])
ck("pointcloud crop non-destructive",'"nonDestructive":true' in files["draw"])
ck("pointcloud crop test","pointcloud_crop_is_non_destructive_and_invertible" in files["draw"])
ck("pointcloud crop manager","Crop box" in files["manager"] and 'pointcloud.crop' in files["manager"])
ck("pointcloud crop bounds UI",'crop_min' in files["manager"] and 'crop_max' in files["manager"] and 'crop_invert' in files["manager"])
ck("massprops implementation","fn run_massprops3d" in files["draw"] and 'surfaceArea' in files["draw"] and 'signedVolume' in files["draw"])
ck("massprops test","facet_massprops_reports_area_volume_and_bounds" in files["draw"])
ck("facet shell implementation","fn run_facetcheck3d" in files["draw"] and 'boundaryEdges' in files["draw"] and 'nonManifoldEdges' in files["draw"])
ck("facet shell test","facet_shell_check_finds_boundary_edges" in files["draw"])
ck("capability crop truth","non-destructive LOD/crop box" in files["inquiry"])
ck("capability inspect truth","3DFACE mass properties + shell topology check" in files["inquiry"])
ck("ACIS remains false",'"acis": false' in files["inquiry"])
failed=[n for n,v in checks if not v]
for n,v in checks: print(("PASS" if v else "FAIL")+"  "+n)
print(f"\nTOTAL {sum(v for _,v in checks)}/{len(checks)} PASS")
if failed: print("FAILED: "+", ".join(failed)); raise SystemExit(1)
