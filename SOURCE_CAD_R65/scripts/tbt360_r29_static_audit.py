from pathlib import Path
root=Path(__file__).resolve().parents[1]
engine=(root/'crates/engine/src/cmd/draw.rs').read_text()
manager=(root/'crates/ui-egui/src/managers.rs').read_text()
palette=(root/'crates/ui-egui/src/palettes.rs').read_text()
inquiry=(root/'crates/engine/src/cmd/inquiry.rs').read_text()
layer=(root/'crates/engine/src/cmd/layer.rs').read_text()
compare=(root/'crates/engine/src/cmd/file/compare.rs').read_text()+(root/'crates/engine/src/cmd/file/compare/export.rs').read_text()
exchange=(root/'crates/engine/src/cmd/exchange.rs').read_text()
layout=(root/'crates/engine/src/cmd/layout.rs').read_text()
plotters=(root/'crates/engine/src/cmd/plotters.rs').read_text()
checks=[]
def add(name,cond): checks.append((name,bool(cond)))
for t in ['CommandSpec::new("pointcloud.sliceplane"','fn run_pointcloud_sliceplane','halfThickness','nonDestructive','pointcloud_sliceplane_is_non_destructive_and_bounded']:
    add('slice '+t,t in engine)
for t in ['"pointcloud.sliceplane"','Slice band','slice_half','slice_invert']:
    add('slice ui '+t,t in manager)
for t in ['CommandSpec::new("facetoffset3d"','fn run_facetoffset3d','skippedDegenerate','warning','facet_offset_moves_face_along_its_normal']:
    add('facet offset '+t,t in engine)
for t in ['"facetoffset3d"','Offset selected facets','offset_distance']:
    add('facet offset ui '+t,t in manager)
add('facet offset palette','"facetoffset3d", "Offset Facets Along Normal"' in palette)
for t in ['CommandSpec::new("layerstate.save"','CommandSpec::new("layerstate.restore"','CommandSpec::new("layerstate.list"']:
    add('layer state engine '+t,t in layer)
for t in ['CommandSpec::new("compare"','CommandSpec::new("compare.next"','CommandSpec::new("compare.previous"']:
    add('compare engine '+t,t in compare)
for t in ['"Layer States & Compare"','"layerstate.save", "Save Layer State"','"compare", "Compare Drawings"','"compareexport", "Export Comparison"']:
    add('layer compare ui '+t,t in palette)
for t in ['CommandSpec::new("publish"','CommandSpec::new("pdfbook"']:
    add('publish engine '+t,t in exchange)
add('plot engine','CommandSpec::new("plot"' in layout)
for t in ['CommandSpec::new("plotwindow"','CommandSpec::new("plottermanager"','CommandSpec::new("plotterinfo"']:
    add('plotter engine '+t,t in plotters)
for t in ['"Publish & Output"','"plot", "Plot / Print"','"publish", "Batch Publish"','"pdfbook", "Sheet Books to PDF"','"plotwindow", "Plot Window"']:
    add('publish ui '+t,t in palette)
add('capability slice','plane-slice band' in inquiry)
add('capability facet offset','per-face normal offset' in inquiry)
add('ACIS remains false','"acis": false' in inquiry)
add('BRep remains missing','B-Rep/ACIS solids' in inquiry)
passed=sum(v for _,v in checks)
for n,v in checks: print(('PASS' if v else 'FAIL')+' | '+n)
print(f'\nSUMMARY {passed}/{len(checks)} PASS')
raise SystemExit(0 if passed==len(checks) else 1)
