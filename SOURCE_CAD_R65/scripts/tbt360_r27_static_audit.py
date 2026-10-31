from pathlib import Path
root=Path(__file__).resolve().parents[1]
engine=(root/'crates/engine/src/cmd/draw.rs').read_text()
blocks=(root/'crates/engine/src/cmd/blocks.rs').read_text()
manager=(root/'crates/ui-egui/src/managers.rs').read_text()
palette=(root/'crates/ui-egui/src/palettes.rs').read_text()
inquiry=(root/'crates/engine/src/cmd/inquiry.rs').read_text()
checks=[]
def add(name, cond): checks.append((name,bool(cond)))
# Point cloud radius engine/UI/read-only regression
for t in ['CommandSpec::new("pointcloud.radius"','fn run_pointcloud_radius','radius must be positive and finite','"matches":total','"readOnly":true','pointcloud_radius_filters_and_sorts_without_mutation']:
    add('radius '+t, t in engine)
add('radius manager action', '"pointcloud.radius"' in manager)
add('radius UI input', 'radius_query' in manager and 'radius_count' in manager)
add('radius capability', 'nearest/radius spatial quer' in inquiry)
# Facet mirror engine/UI/regression
for t in ['CommandSpec::new("facetmirror3d"','fn run_facetmirror3d','normal must be non-zero','arbitrary 3D plane','facet_mirror_reflects_vertices_and_preserves_normal_orientation']:
    add('mirror '+t, t in engine)
add('mirror hidden edge remap', 'f.hidden_edges = ((f.hidden_edges & 0b0001) << 3)' in engine)
add('mirror triangle safe', '[a, c, b, b]' in engine)
add('mirror manager route', 'facetmirror3d' in manager and 'Mirror selected facets' in manager)
add('mirror palette button', '"facetmirror3d", "Mirror Selected Facets"' in palette)
add('mirror capability', 'arbitrary-plane mirror' in inquiry)
# Dynamic block reference editor
for t in ['blocks.reference.set','blocks.reference.grip','Apply property','Move grip through engine','grip_node','grip_part','grip_to']:
    add('dynamic '+t, t in manager)
add('dynamic set command exists', 'CommandSpec::new("blocks.reference.set"' in blocks)
add('dynamic grip command exists', 'CommandSpec::new("blocks.reference.grip"' in blocks)
add('dynamic palette set', '"blocks.reference.set", "Set Dynamic Property"' in palette)
add('dynamic palette grip', '"blocks.reference.grip", "Move Dynamic Grip"' in palette)
add('dynamic authoring boundary', 'Creating new parameter/action graphs is still draft/API-level' in manager)
# Preserve prior R26 workflows
for t in ['pointcloud.nearest','facettransform3d','blocks.dynamic.validate','blocks.reference.grips']:
    add('prior '+t, t in engine+blocks+manager+palette)
# Safety boundaries
add('ACIS still false', '"acis": false' in inquiry)
add('indexed reality still missing', 'indexed reality-capture formats' in inquiry)
passed=sum(v for _,v in checks)
for n,v in checks: print(('PASS' if v else 'FAIL')+' | '+n)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed==len(checks) else 1)
