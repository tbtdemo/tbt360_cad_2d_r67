from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
files={
 'palette':(ROOT/'crates/ui-egui/src/palettes.rs').read_text(),
 'manager':(ROOT/'crates/ui-egui/src/managers.rs').read_text(),
 'settings':(ROOT/'crates/engine/src/cmd/settings.rs').read_text(),
 'draw':(ROOT/'crates/engine/src/cmd/draw.rs').read_text(),
 'render':(ROOT/'crates/render/src/draw.rs').read_text(),
 'inquiry':(ROOT/'crates/engine/src/cmd/inquiry.rs').read_text(),
}
checks=[]
def ck(name, cond): checks.append((name,bool(cond)))
# 2D parallel exposure: drafting precision + selection/display
for cmd in ['ortho','osnap','polar','otrack','grid','snap','dynmode','dsettings']:
    ck('2D Drafting Precision '+cmd, f'"{cmd}"' in files['palette'])
for cmd in ['selectioncycling','qpmode','lwdisplay','transparencydisplay','isodraft','isoplane']:
    ck('2D Selection/Display '+cmd, f'"{cmd}"' in files['palette'])
ck('2D Drafting Precision group', '"Drafting Precision"' in files['palette'])
ck('2D Selection & Display group', '"Selection & Display"' in files['palette'])
# Existing R19 gates retained
for cmd in ['overkill','join','pedit','splinedit','audit','-purge','flatten','dist','area','id','measuregeom','list','qselect','properties','divide','measure']:
    ck('R19 retained '+cmd, f'"{cmd}"' in files['palette'])
# Back-face culling engine/render/UI/test
ck('backface spec', 'CommandSpec::new("backfacecull"' in files['settings'])
ck('backface alias', '"bfcull"' in files['settings'])
ck('backface engine', 'fn run_backfacecull' in files['settings'])
ck('backface persisted header', 'BACKFACECULL' in files['settings'])
ck('backface renderer gate', 'header.i64("BACKFACECULL"' in files['render'])
ck('backface view normal depth', 'fn view_normal_depth' in files['render'])
ck('backface UI', '"backfacecull"' in files['palette'])
ck('backface regression', 'backface_culling_persists_and_toggles' in files['settings'])
# Point-cloud display style engine/render/UI/test
ck('pointcloud.style spec', 'CommandSpec::new("pointcloud.style"' in files['draw'])
ck('pointcloud.style engine', 'fn run_pointcloud_style' in files['draw'])
ck('pointcloud size persisted', 'POINTCLOUDSIZE' in files['draw'])
ck('pointcloud mode persisted', 'POINTCLOUDMODE' in files['draw'])
ck('pointcloud renderer tag', 'fn pointcloud_entity' in files['render'])
ck('pointcloud renderer size', 'POINTCLOUDSIZE' in files['render'])
ck('pointcloud renderer mode', 'POINTCLOUDMODE' in files['render'])
ck('pointcloud manager style', 'Apply display style' in files['manager'])
ck('pointcloud style route', 'id == "pointcloud" || id == "pointcloud.style"' in files['manager'])
ck('pointcloud style ToolSet', '"pointcloud.style"' in files['palette'])
ck('pointcloud style validation test', 'pointcloud_display_style_persists_and_validates' in files['draw'])
# Section + capability truthfulness
ck('section cut ToolSet', '"sectioncutstyle"' in files['palette'])
ck('capability backface', 'optional back-face culling' in files['inquiry'])
ck('capability point display', 'point display style' in files['inquiry'])
ck('capability 3D dimension truth', '3D distance/angular/radius dimensions' in files['inquiry'])
ck('capability 3D leader truth', '3D leaders with dogleg' in files['inquiry'])
ck('ACIS remains false', '"acis": false' in files['inquiry'])
failed=[n for n,v in checks if not v]
for n,v in checks: print(('PASS' if v else 'FAIL')+'  '+n)
print(f'\nTOTAL {sum(v for _,v in checks)}/{len(checks)} PASS')
if failed:
    print('FAILED: '+', '.join(failed)); raise SystemExit(1)
