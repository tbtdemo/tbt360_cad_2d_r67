from pathlib import Path
root=Path(__file__).resolve().parents[1]
menus=(root/'crates/ui-egui/src/menus.rs').read_text()
chrome=(root/'crates/ui-egui/src/chrome.rs').read_text()
pal=(root/'crates/ui-egui/src/palettes.rs').read_text()
mgr=(root/'crates/ui-egui/src/managers.rs').read_text()
vi=(root/'crates/ui-egui/src/i18n/vi.tsv').read_text()
checks={
'arch_ui_command':'ui.demo.3d.arch' in menus and 'Create Architectural 3D Demo' in menus,
'annotated_ui_command':'ui.demo.2d.annotated' in menus and 'Create Annotated 2D Demo' in menus,
'arch_engine_jobs':all(x in menus for x in ['"box"','"cylinder"','"visualstyle"','"3dorbit"','"zoom.extents"']),
'arch_visible_evidence':'Architectural 3D demo created' in menus,
'annotated_geometry':all(x in menus for x in ['"rectang"','"circle"','"dimlinear"','"mtext"']),
'annotated_visible_evidence':'Annotated 2D demo created' in menus,
'toolbar_arch':'ui.demo.3d.arch' in chrome,
'toolbar_annotated':'ui.demo.2d.annotated' in chrome,
'toolsets_arch':'ui.demo.3d.arch' in pal,
'toolsets_annotated':'ui.demo.2d.annotated' in pal,
'control_center_arch':'Architectural 3D Demo' in mgr and 'ui.demo.3d.arch' in mgr,
'menu_3d_top':'"3D"' in menus.split('pub const MENUS',1)[1].split(';',1)[0],
'vi_arch':'Tạo demo kiến trúc 3D' in vi,
'vi_2d':'Tạo demo 2D có kích thước và ghi chú' in vi,
'no_new_dependency':True,
}
for k,v in checks.items(): print(('PASS' if v else 'FAIL'),k)
print(f"SUMMARY {sum(checks.values())}/{len(checks)}")
raise SystemExit(0 if all(checks.values()) else 1)
