from pathlib import Path
root=Path(__file__).resolve().parents[1]
files={
 'draw':(root/'crates/engine/src/cmd/draw.rs').read_text(),
 'pal':(root/'crates/ui-egui/src/palettes.rs').read_text(),
 'mgr':(root/'crates/ui-egui/src/managers.rs').read_text(),
}
checks=[
 ('pointcloud box command','CommandSpec::new("pointcloud.box3d"' in files['draw']),
 ('pointcloud box engine','fn run_pointcloud_box3d' in files['draw']),
 ('pointcloud box 12 edges','let edges=[(0,1),(1,2),(2,3),(3,0),(4,5),(5,6),(6,7),(7,4),(0,4),(1,5),(2,6),(3,7)]' in files['draw']),
 ('pointcloud box native line','EntityKind::Line(Line{a:v[a],b:v[b]})' in files['draw']),
 ('pointcloud manager 3d bounds','"3D Bounds"' in files['mgr'] and 'pointcloud.box3d' in files['mgr']),
 ('facet stats command','CommandSpec::new("facetstats3d"' in files['draw']),
 ('facet stats alias','"3dstats"' in files['draw'] and '"facetstats"' in files['draw']),
 ('facet stats engine','fn run_facetstats3d' in files['draw']),
 ('facet stats area','"totalArea"' in files['draw']),
 ('facet stats edge lengths','"minEdge"' in files['draw'] and '"maxEdge"' in files['draw'] and '"meanEdge"' in files['draw']),
 ('facet stats toolset','"facetstats3d", "Facet Statistics"' in files['pal']),
 ('pdf clip tool','"pdfclip", "Clip PDF Underlay"' in files['pal']),
 ('image clip tool','"imageclip", "Clip Raster Image"' in files['pal']),
]
failed=[n for n,ok in checks if not ok]
for n,ok in checks: print(('PASS' if ok else 'FAIL')+': '+n)
print(f'\nsummary={len(checks)-len(failed)}/{len(checks)} PASS')
raise SystemExit(1 if failed else 0)
