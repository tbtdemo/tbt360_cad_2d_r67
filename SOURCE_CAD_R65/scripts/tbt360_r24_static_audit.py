from pathlib import Path

root = Path(__file__).resolve().parents[1]
draw = (root / 'crates/engine/src/cmd/draw.rs').read_text(errors='ignore')
managers = (root / 'crates/ui-egui/src/managers.rs').read_text(errors='ignore')
palettes = (root / 'crates/ui-egui/src/palettes.rs').read_text(errors='ignore')
inquiry = (root / 'crates/engine/src/cmd/inquiry.rs').read_text(errors='ignore')

checks = []
def ck(name, cond):
    checks.append((name, bool(cond)))

for cmd in ['pointcloud.bounds','pointcloud.color','pointcloud.exportxyz','facetorientout3d']:
    ck(f'{cmd} registered', f'CommandSpec::new("{cmd}"' in draw)
for fn in ['run_pointcloud_bounds','run_pointcloud_color','run_pointcloud_exportxyz','run_facetorientout3d']:
    ck(f'{fn} implementation', f'fn {fn}(' in draw)
ck('bounds reports diagonal', '"diagonal":diag' in draw)
ck('bounds reports visibleCount', '"visibleCount":visible' in draw)
ck('color preserves XYZ marker', '"xyzPreserved":true' in draw)
ck('color validates Color::parse', 'Color::parse(v)' in draw)
ck('export visibleOnly', 'visible_only=p.get("visibleOnly")' in draw)
ck('export delimiter validation', 'delimiter must be comma, space or tab' in draw)
ck('export read-only marker', '"readOnly":true' in draw)
ck('orient uses selection centroid', 'selection-centroid outward orientation' in draw)
ck('orient skips degenerate normals', 'n.len()>1e-12' in draw)
ck('orient reports flipped count', '"flipped":flip.len()' in draw)
ck('pointcloud regression test', 'pointcloud_bounds_color_and_export_preserve_xyz' in draw)
ck('facet orient regression test', 'facet_orient_outward_flips_inward_faces' in draw)
for label in ['Bounds','Color','Export XYZ']:
    ck(f'Point Cloud Manager {label}', f'"{label}"' in managers)
ck('cloud color UI state', 'cloud_color' in managers)
ck('export visible only UI', 'export_visible_only' in managers)
ck('export delimiter UI', 'pointcloud_export_delimiter' in managers)
for cmd in ['mview','vpclip','vplayer','layout','pspace','mspace']:
    ck(f'Layout & Output {cmd}', f'"{cmd}"' in palettes)
ck('Facet Orient Outward UI', 'facetorientout3d' in palettes)
ck('capabilities point-cloud bounds/color/export', 'per-cloud color + bounded XYZ text export' in inquiry)
ck('capabilities auto orientation', 'centroid-based outward orientation repair' in inquiry)
ck('ACIS still false', '"acis": false' in inquiry)
ck('indexed point cloud still missing', 'LAS/LAZ/E57/RCP indexed Point Cloud' in inquiry)

passed=sum(v for _,v in checks)
for name, ok in checks:
    print(('PASS' if ok else 'FAIL') + ' | ' + name)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed == len(checks) else 1)
