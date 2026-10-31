from pathlib import Path
root=Path(__file__).resolve().parents[1]
engine=(root/'crates/engine/src/cmd/draw.rs').read_text()
manager=(root/'crates/ui-egui/src/managers.rs').read_text()
palette=(root/'crates/ui-egui/src/palettes.rs').read_text()
inquiry=(root/'crates/engine/src/cmd/inquiry.rs').read_text()
constraints=(root/'crates/engine/src/cmd/constraints.rs').read_text()
checks=[]
def add(n,c): checks.append((n,bool(c)))
for t in ['CommandSpec::new("facettriangulate3d"','fn run_facettriangulate3d','native triangle 3DFACE','triangulate_and_weld_facets']:
    add('triangulate '+t,t in engine)
for t in ['CommandSpec::new("facetweld3d"','fn run_facetweld3d','movedVertices','tolerance must be positive and finite']:
    add('weld '+t,t in engine)
for t in ['"facettriangulate3d", "Triangulate Facets"','"facetweld3d", "Weld Facet Vertices"']:
    add('facet ui '+t,t in palette)
for t in ['CommandSpec::new("pointcloud.zstats"','fn run_pointcloud_zstats','histogram','meanZ','pointcloud_zstats_and_ply_are_read_only']:
    add('zstats '+t,t in engine)
for t in ['"pointcloud.zstats"','Elevation statistics','zstats_bins']:
    add('zstats ui '+t,t in manager)
for t in ['CommandSpec::new("pointcloud.exportply"','fn run_pointcloud_exportply','format ascii 1.0','maxPoints']:
    add('ply '+t,t in engine)
for t in ['"Constraint Diagnostics"','"constraints.inspect", "Inspect Constraints"','"constraints.dof", "Degrees of Freedom"','"constraintsettings", "Constraint Settings"','"constraintinfer", "Infer Constraints"','"constraintbar", "Constraint Bars"']:
    add('constraint ui '+t,t in palette)
for t in ['CommandSpec::new("geomconstraint"','CommandSpec::new("dimconstraint"','CommandSpec::new("constraintsettings"','CommandSpec::new("constraints.inspect"','CommandSpec::new("constraints.dof"']:
    add('constraint engine '+t,t in constraints)
add('capability ply','ASCII PLY export' in inquiry)
add('capability weld','vertex welding' in inquiry)
add('acis false','"acis": false' in inquiry)
passed=sum(v for _,v in checks)
for n,v in checks: print(('PASS' if v else 'FAIL')+' | '+n)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed==len(checks) else 1)
