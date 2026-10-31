from pathlib import Path

root = Path(__file__).resolve().parents[1]
draw = (root / 'crates/engine/src/cmd/draw.rs').read_text(errors='ignore')
managers = (root / 'crates/ui-egui/src/managers.rs').read_text(errors='ignore')
palettes = (root / 'crates/ui-egui/src/palettes.rs').read_text(errors='ignore')
inquiry = (root / 'crates/engine/src/cmd/inquiry.rs').read_text(errors='ignore')

checks=[]
def ck(name, cond): checks.append((name, bool(cond)))

for cmd in ['pointcloud.rotate3d','pointcloud.filterz','facetcleanup3d']:
    ck(f'{cmd} registered', f'CommandSpec::new("{cmd}"' in draw)
for fn in ['run_pointcloud_rotate3d','run_pointcloud_filterz','run_facetcleanup3d']:
    ck(f'{fn} implementation', f'fn {fn}(' in draw)
ck('rotate3d uses Rodrigues formula', 'k.cross(q)*sn+k*(k.dot(q)*(1.0-cs))' in draw)
ck('rotate3d rejects zero axis', 'axis must be non-zero' in draw)
ck('rotate3d defaults to centroid', 'pointcloud.rotate3d")?}else{centroid}' in draw)
ck('filterz is non destructive', '"nonDestructive":true' in draw and 'run_pointcloud_filterz' in draw)
ck('filterz validates finite range', 'minZ and maxZ must be finite' in draw)
ck('facet cleanup tolerance validation', 'facetcleanup3d","tolerance must be positive and finite' in draw)
ck('facet cleanup optional delete', 'let delete=p.get("delete")' in draw)
ck('facet cleanup reports handles', '"handles":bad_handles.iter()' in draw)
ck('pointcloud R25 regression', 'pointcloud_rotate3d_and_filterz_are_real_geometry_workflows' in draw)
ck('facet cleanup regression', 'facet_cleanup_reports_without_deleting_by_default' in draw)
for label in ['Rotate 3D','Filter Z']:
    ck(f'Point Cloud Manager {label}', f'"{label}"' in managers)
ck('Point Cloud 3D axis UI', 'Rotate 3D axis' in managers)
ck('Point Cloud elevation UI', 'Elevation filter Z' in managers)
ck('Point Cloud R25 help text', 'R25 keeps native XYZ POINT data' in managers)
for cmd in ['text','mtext','textedit','find','style','spell','field','updatefield','mleader']:
    ck(f'Text & Annotation {cmd}', f'"{cmd}"' in palettes)
for cmd in ['audit','recover','drawingrecovery','compare','units','rename','purge','saveas']:
    ck(f'Drawing Maintenance {cmd}', f'"{cmd}"' in palettes)
ck('Facet cleanup UI', 'facetcleanup3d' in palettes)
ck('capabilities arbitrary axis rotate', 'arbitrary-axis rotate' in inquiry)
ck('capabilities elevation filter', 'elevation filter' in inquiry)
ck('capabilities facet cleanup', 'degenerate-facet cleanup' in inquiry)
ck('ACIS still false', '"acis": false' in inquiry)
ck('indexed point cloud still missing', 'LAS/LAZ/E57/RCP indexed Point Cloud' in inquiry)

passed=sum(v for _,v in checks)
for name,ok in checks: print(('PASS' if ok else 'FAIL')+' | '+name)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed==len(checks) else 1)
