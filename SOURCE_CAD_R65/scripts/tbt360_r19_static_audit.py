from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
files={
 'palette':(ROOT/'crates/ui-egui/src/palettes.rs').read_text(),
 'manager':(ROOT/'crates/ui-egui/src/managers.rs').read_text(),
 'draw':(ROOT/'crates/engine/src/cmd/draw.rs').read_text(),
 'render':(ROOT/'crates/render/src/draw.rs').read_text(),
}
checks=[]
def ck(name, cond): checks.append((name,bool(cond)))
# 2D parallel exposure
for cmd in ['overkill','join','pedit','splinedit','audit','-purge','flatten','dist','area','id','measuregeom','list','qselect','properties','divide','measure']:
    ck('2D UI '+cmd, f'"{cmd}"' in files['palette'])
# 3D point cloud LOD
ck('pointcloud.sample spec', 'CommandSpec::new("pointcloud.sample"' in files['draw'])
ck('pointcloud.sample engine', 'fn run_pointcloud_sample' in files['draw'])
ck('pointcloud.sample non-destructive visibility', 'e.common.visible=show' in files['draw'])
ck('pointcloud.sample test', 'pointcloud_sampling_is_non_destructive' in files['draw'])
ck('pointcloud LOD 1/5 UI', 'LOD 1/5' in files['manager'])
ck('pointcloud LOD 1/20 UI', 'LOD 1/20' in files['manager'])
ck('pointcloud Full UI', '"Full"' in files['manager'])
# section cut edges
ck('sectioncutstyle spec', 'CommandSpec::new("sectioncutstyle"' in files['draw'])
ck('sectioncutstyle engine', 'fn run_sectioncutstyle' in files['draw'])
ck('section cut points helper', 'fn section_cut_points' in files['render'])
ck('section cut render gate', 'SECTIONCUTLINES' in files['render'])
ck('section cut hairline', 'b.hairline(ctx,rgb' in files['render'])
ck('section cut UI', 'Toggle cut edges' in files['manager'])
ck('section cut state test', 'section_cut_lines_state_persists_in_header' in files['draw'])
# architecture invariants
ck('sample max clamp', 'clamp(1,1000)' in files['draw'])
ck('sample preserves entities', 'remove_entity' not in files['draw'][files['draw'].find('fn run_pointcloud_sample'):files['draw'].find('fn run_sectioncutstyle')])
ck('section marker not self clipped', '!section_marker(e)' in files['render'])
ck('point cloud doc-state path', 'TBT360_POINTCLOUD' in files['draw'] and 'TBT360_POINTCLOUD' in files['manager'])
ck('2D cleanup label', '"Geometry Cleanup"' in files['palette'])
ck('inspect measure label', '"Inspect & Measure"' in files['palette'])
failed=[n for n,v in checks if not v]
for n,v in checks: print(('PASS' if v else 'FAIL')+'  '+n)
print(f'\nTOTAL {sum(v for _,v in checks)}/{len(checks)} PASS')
if failed:
    print('FAILED: '+', '.join(failed)); raise SystemExit(1)
