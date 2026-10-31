from pathlib import Path
root=Path(__file__).resolve().parents[1]
menus=(root/'crates/ui-egui/src/menus.rs').read_text()
chrome=(root/'crates/ui-egui/src/chrome.rs').read_text()
pal=(root/'crates/ui-egui/src/palettes.rs').read_text()
draw=(root/'crates/engine/src/cmd/draw.rs').read_text()
checks={
 'top_level_3d_menu':'"3D", "Window"' in menus,
 '3d_workspace_command':'ui.mode.3d' in menus and 'toolset_tab = "Modeling"' in menus,
 '3d_demo_command':'ui.demo.3d' in menus and '3D demo scene created' in menus,
 'toolbar_3d_workspace':'"ui.mode.3d", "3D Workspace"' in chrome,
 'toolbar_3d_demo':'"ui.demo.3d", "Create 3D Demo Scene"' in chrome,
 'duplicate_engine_3d_menu':'c.menu[0] == "Draw" && c.menu[1] == "3D Modeling"' in menus,
 'modeling_toolset': 'pub const MODELING' in pal,
 'box_engine': 'CommandSpec::new("box"' in draw,
 'cylinder_engine': 'CommandSpec::new("cylinder"' in draw,
 'sphere_engine': 'CommandSpec::new("sphere"' in draw,
 'torus_engine': 'CommandSpec::new("torus"' in draw,
 'demo_shaded':'ShadedWithEdges' in menus,
 'demo_iso':'"preset":"seiso"' in menus,
 'demo_zoom_extents':'zoom.extents' in menus,
}
failed=[k for k,v in checks.items() if not v]
for k,v in checks.items(): print(('PASS' if v else 'FAIL'), k)
print(f'\n{len(checks)-len(failed)}/{len(checks)} PASS')
raise SystemExit(1 if failed else 0)
