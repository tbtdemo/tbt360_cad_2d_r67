#!/usr/bin/env python3
from pathlib import Path
import sys
root=Path(__file__).resolve().parents[1]
files={
 'draw':(root/'crates/engine/src/cmd/draw.rs').read_text(errors='ignore'),
 'managers':(root/'crates/ui-egui/src/managers.rs').read_text(errors='ignore'),
 'palettes':(root/'crates/ui-egui/src/palettes.rs').read_text(errors='ignore'),
 'render':(root/'crates/render/src/draw.rs').read_text(errors='ignore'),
 'ui_audit':(root/'scripts/tbt360_cad2027_ui_audit.py').read_text(errors='ignore'),
 'status':(root/'CAD-2D-TBT-360-ROADMAP-CAD2027-STATUS.md').read_text(errors='ignore'),
}
checks=[
 ('underlay dialog route', 'matches!(id, "attach" | "pdfattach" | "imageattach")' in files['managers']),
 ('underlay dialog function', 'fn underlay_attach_dialog' in files['managers']),
 ('underlay attach tool group', '"Underlay & Raster"' in files['palettes']),
 ('attach button visible', '"attach", "Attach Underlay / Raster"' in files['palettes']),
 ('pdf attach button visible', '"pdfattach", "Attach PDF Underlay"' in files['palettes']),
 ('image attach button visible', '"imageattach", "Attach Raster Image"' in files['palettes']),
 ('unknown underlay render branch', 'EntityKind::Unknown(u) if u.dxf_type.to_ascii_uppercase().ends_with("UNDERLAY")' in files['render']),
 ('underlay clip frame rendering', 'if clipping && xs.len() >= 3 && xs.len() == ys.len()' in files['render']),
 ('underlay placement diagonals', 'b.polyline(ctx, rgb, lw, &[a, c]);' in files['render']),
 ('ui audit promotes attach commands', "'attach', 'pdfattach', 'imageattach'" in files['ui_audit']),
 ('facet subdivide command', 'CommandSpec::new("facetsubdivide3d"' in files['draw']),
 ('facet subdivide engine', 'fn run_facetsubdivide3d' in files['draw']),
 ('facet subdivide ui', '"facetsubdivide3d", "Subdivide Facets"' in files['palettes']),
 ('facet subdivide test', 'r32_subdivide_preserves_native_face_geometry' in files['draw']),
 ('pointcloud dedup command', 'CommandSpec::new("pointcloud.dedup"' in files['draw']),
 ('pointcloud dedup engine', 'fn run_pointcloud_dedup' in files['draw']),
 ('pointcloud dedup report ui', '"Find dupes"' in files['managers']),
 ('pointcloud dedup delete ui', '"Remove dupes"' in files['managers']),
 ('pointcloud dedup test', 'r32_pointcloud_dedup_reports_then_removes_duplicates' in files['draw']),
 ('status underlay r32', 'R32 có dialog attach' in files['status']),
 ('status subdivide r32', 'FACETSUBDIVIDE3D' in files['status']),
]
failed=[]
for name,ok in checks:
 print(('PASS' if ok else 'FAIL')+' | '+name)
 if not ok: failed.append(name)
print(f'\nSUMMARY {len(checks)-len(failed)}/{len(checks)} PASS')
if failed: print('FAILED:', ', '.join(failed))
sys.exit(1 if failed else 0)
