from pathlib import Path
root=Path(__file__).resolve().parents[1]
engine=(root/'crates/engine/src/cmd/draw.rs').read_text()
manager=(root/'crates/ui-egui/src/managers.rs').read_text()
palette=(root/'crates/ui-egui/src/palettes.rs').read_text()
inquiry=(root/'crates/engine/src/cmd/inquiry.rs').read_text()
table=''.join(p.read_text() for p in (root/'crates/engine/src/cmd/table').glob('*.rs'))
checks=[]
def add(name,cond): checks.append((name,bool(cond)))
for t in ['CommandSpec::new("pointcloud.voxel"','fn run_pointcloud_voxel','voxelSize','xyzPreserved','pointcloud_voxel_and_clipplane_are_non_destructive_filters']:
    add('voxel '+t,t in engine)
for t in ['CommandSpec::new("pointcloud.clipplane"','fn run_pointcloud_clipplane','keepPositive','nonDestructive']:
    add('clipplane '+t,t in engine)
add('voxel manager', '"pointcloud.voxel"' in manager and 'Voxel display' in manager)
add('clipplane manager', '"pointcloud.clipplane"' in manager and 'Clip by plane' in manager)
for t in ['CommandSpec::new("facetdedup3d"','fn run_facetdedup3d','duplicateCount','facet_dedup_reports_then_deletes_duplicate_faces']:
    add('dedup '+t,t in engine)
add('dedup manager','Report duplicate facets' in manager and 'Delete duplicate facets' in manager)
add('dedup palette','"facetdedup3d", "Remove Duplicate Facets"' in palette)
for t in ['CommandSpec::new("datalink"','CommandSpec::new("datalinkupdate"','CommandSpec::new("tabledit"','CommandSpec::new("table.break"']:
    add('2d data engine '+t,t in table)
for t in ['"Data Link & Table Automation"','"datalink", "Data Link"','"datalinkupdate", "Update Data Links"','"table.break", "Table Break"','"dataextraction", "Data Extraction"']:
    add('2d data ui '+t,t in palette)
add('capability voxel','voxel display/arbitrary clip-plane filter' in inquiry)
add('capability dedup','duplicate-facet cleanup' in inquiry)
add('ACIS remains false','"acis": false' in inquiry)
add('indexed formats remain missing','indexed reality-capture formats' in inquiry)
passed=sum(v for _,v in checks)
for n,v in checks: print(('PASS' if v else 'FAIL')+' | '+n)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed==len(checks) else 1)
