from pathlib import Path
root=Path(__file__).resolve().parents[1]
files={
 'draw':(root/'crates/engine/src/cmd/draw.rs').read_text(),
 'settings':(root/'crates/engine/src/cmd/settings.rs').read_text(),
 'ui':(root/'crates/ui-egui/src/managers.rs').read_text(),
 'palette':(root/'crates/ui-egui/src/palettes.rs').read_text(),
 'cap':(root/'crates/engine/src/cmd/inquiry.rs').read_text(),
}
checks={
 'density command':'CommandSpec::new("pointcloud.density"' in files['draw'],
 'density engine':'fn run_pointcloud_density' in files['draw'],
 'density UI':'pointcloud.density' in files['ui'] and 'Density' in files['ui'],
 'copy visible command':'CommandSpec::new("pointcloud.copyvisible"' in files['draw'],
 'copy visible engine':'fn run_pointcloud_copyvisible' in files['draw'],
 'copy visible UI':'pointcloud.copyvisible' in files['ui'] and 'Copy visible' in files['ui'],
 'copy visible source preserved':'sourcePreserved' in files['draw'],
 'facet boundary command':'CommandSpec::new("facetboundary3d"' in files['draw'],
 'facet boundary engine':'fn run_facetboundary3d' in files['draw'],
 'facet boundary UI':'facetboundary3d' in files['palette'] and 'Extract Facet Boundary' in files['palette'],
 'facet boundary 3d tag':'facet-boundary' in files['draw'] and 'tag_3d_annotation' in files['draw'],
 'visualstyles alias':'"visualstyles"' in files['settings'],
 'QA diagnostics UI':'QA Diagnostics' in files['palette'] and 'drawing.inspect' in files['palette'] and 'regioncheck' in files['palette'],
 'R31 regression test':'r31_pointcloud_density_copyvisible_and_facet_boundary_workflows' in files['draw'],
 'capability density':'density statistics' in files['cap'],
 'capability boundary':'open-boundary extraction' in files['cap'],
 'acis remains false':'"acis": false' in files['cap'],
}
passed=0
for name,ok in checks.items():
 print(('PASS' if ok else 'FAIL')+' | '+name)
 passed+=bool(ok)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed==len(checks) else 1)
