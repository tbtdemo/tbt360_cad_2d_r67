# CAD 2D - TBT 360 — Roadmap phủ lệnh CAD 2027

## Mục tiêu đã chốt

Ưu tiên số 1 là bổ sung tối đa các lệnh, tùy chọn lệnh và quy trình thao tác tương đương AutoCAD 2027. Chưa ưu tiên tối ưu tốc độ, đa luồng, GPU, bộ nhớ hoặc chỉnh giao diện sâu trong giai đoạn này.

Mỗi lệnh chỉ được đánh dấu hoàn thành khi:

1. Có tên lệnh đầy đủ và alias gần với CAD.
2. Có dạng nhập dòng lệnh và dạng JSON/API khi phù hợp.
3. Có prompt, keyword, Enter, Esc, Undo và thông báo lỗi.
4. Có kiểm thử lệnh cơ bản và hostile-input.
5. Có lưu/đọc dữ liệu nếu lệnh tạo dữ liệu trong bản vẽ.
6. Không ghi là hoàn thành nếu mới chỉ có menu hoặc alias.

## Trạng thái hiện tại

Đã có nền tảng 2D cho LINE, PLINE, CIRCLE, ARC, ELLIPSE, SPLINE, RECTANG, XLINE, RAY, HATCH, TEXT, MTEXT, DIMENSION, MLEADER, BLOCK, INSERT, BEDIT tĩnh, XREF cơ bản, LAYER, PROPERTIES, MATCHPROP, MOVE, COPY, ROTATE, SCALE, MIRROR, OFFSET, TRIM, EXTEND, FILLET, CHAMFER, STRETCH, ARRAY, PEDIT, SPLINEDIT, GROUP, REGION, MLINE, LAYOUT, MVIEW, VPORTS, VPCLIP, PLOT, PDF, AUDIT, RECOVER, PURGE, OVERKILL, SELECTSIMILAR, QSELECT, LISP lõi và nhiều lệnh Inquiry.

MVIEW Restore đã được triển khai và có regression test. Các lệnh hiện có vẫn cần rà soát prompt, alias, keyword và độ tương thích dữ liệu trước khi coi là hoàn tất.

## Giai đoạn 1 — Phủ lệnh CAD 2027 trước

### A. Lệnh nền và vẽ 2D

- [ ] Bổ sung đầy đủ prompt/keyword cho `-LINE`, `-PLINE`, `-ARC`, `-CIRCLE`, `-ELLIPSE`, `-SPLINE`, `-RECTANG`, `-POLYGON`, `-DONUT`, `-XLINE`, `-RAY`.
- [ ] Bổ sung `DIVIDE`, `MEASURE`, `POINT`, `MULTIPOINT`, `REVCLOUD`, `SKETCH`.
- [ ] Hoàn thiện PLINE arc mode, Width, Halfwidth, Length, Undo, Close.
- [ ] Hoàn thiện SPLINE Object, Method, Knots, Tangency, Tolerance, Close/Open.
- [ ] Bổ sung `BOUNDARY`, `BPOLY`, `REGION`, `MASSPROP` cho các trường hợp biên thực tế.
- [ ] Bổ sung `SUPERHATCH`, `WIPEOUT`, `IMAGEFRAME`, `PDFIMPORT`.

### B. Lệnh hiệu chỉnh hình học

- [ ] Hoàn thiện `TRIM`, `EXTEND`, `BREAK`, `BREAKATPOINT`, `JOIN`, `LENGTHEN` với fence, crossing, edge mode và nhiều đối tượng.
- [ ] Hoàn thiện `FILLET`, `CHAMFER`, `BLEND`, `OFFSET` cho ellipse, spline, polyline bulge và các trường hợp giao nhau.
- [ ] Hoàn thiện `STRETCH` với Window, Crossing, CP, WP, Fence, Polygon.
- [ ] Bổ sung `ALIGN`, `3DALIGN` ở mức 2D tương thích prompt.
- [ ] Bổ sung `MOCORO`, `NCOPY`, `EXPLODE`, `BURST`, `FLATTEN`, `OVERKILL` đầy đủ tùy chọn.
- [ ] Bổ sung `JOIN`, `PEDIT`, `SPLINEDIT` toàn bộ tùy chọn còn thiếu.
- [ ] Bổ sung chỉnh sửa Grip cho line, polyline, arc, circle, dimension, hatch, text, MLeader và array.

### C. Array và nhóm đối tượng

- [ ] Hoàn thiện `ARRAYRECT`, `ARRAYPOLAR`, `ARRAYPATH`.
- [ ] Bổ sung associative path array.
- [ ] Bổ sung `ARRAYEDIT`, `ARRAYCLOSE`, xóa từng item bằng Ctrl + click.
- [ ] Bổ sung grip số lượng, khoảng cách, góc, base point và level cho Array.
- [ ] Hoàn thiện `GROUP`, `-GROUP`, `GROUPEDIT`, `UNGROUP`, `PICKSTYLE`.
- [x] Group bounding box/grip metadata, `GROUPDISPLAYMODE` state/query và Copy Group (`GROUPEDIT copy`) nền tảng; còn native canvas grip/dialog/render highlight.

### D. Block và Dynamic Block

- [ ] Hoàn thiện `BLOCK`, `-BLOCK`, `WBLOCK`, `INSERT`, `-INSERT`, `ATTDEF`, `-ATTDEF`.
- [ ] Hoàn thiện `BEDIT`, `BSAVE`, `BSAVEAS`, `BCLOSE`.
- [ ] Bổ sung Dynamic Block Authoring: Parameters, Actions, Constraints, Visibility States, Lookup. Nền tảng hiện có `blocks.dynamic.validate` và `blocks.dynamic.author` tạo/kiểm tra draft graph trong bộ nhớ; chưa ghi ACAD_ENHANCEDBLOCK vào DWG.
- [ ] Bổ sung Dynamic Block grips: Linear, Polar, XY, Rotation, Stretch, Move, Array, Lookup.
- [ ] Hoàn thiện `REFEDIT`, `REFSET`, `REFCLOSE` cho block reference.
- [ ] Hoàn thiện `ATTEDIT`, `-ATTEDIT`, `BATTMAN`, `ATTSYNC`.
- [ ] Bổ sung Verify, Preset, Lock Position và tạo Multiline Attribute.
- [ ] Hoàn thiện `EATTEXT`, `DATAEXTRACTION`, nested block và template `.dxe`.

### E. XREF và liên kết bản vẽ

- [ ] Hoàn thiện `XATTACH`, `-XREF`, `XREF`, `XOPEN`, `XBIND`.
- [ ] Bổ sung `XCLIP`: Rectangular, Polygonal, Invert, Delete, New Boundary.
- [ ] Bổ sung clipping hiển thị, chọn, snap và plot cho XREF.
- [ ] Bổ sung REFEDIT đối với XREF ở chế độ chỉnh sửa tại chỗ.
- [ ] Bổ sung nested reference selection, faded rest of drawing và Save/Discard banner.
- [ ] Hoàn thiện relative path, overlay, unload/reload và layer override của XREF.

### F. Layer, Properties và quản lý bản vẽ

- [ ] Hoàn thiện `LAYER`, `-LAYER`, Layer States, Layer Filters, Layer Walk, Layer Isolate.
- [ ] Bổ sung `LAYTRANS`, `LAYMCH`, `LAYMRG`, `LAYDEL`, `LAYVPI`.
- [ ] Hoàn thiện `PROPERTIES`, `QUICKPROPERTIES`, `MATCHPROP` và Settings dialog.
- [ ] Bổ sung `CHPROP`, `SETBYLAYER`, `CHANGE`, `LIST`, `ID`, `DIST`, `AREA`, `MEASUREGEOM`.
- [ ] Hoàn thiện `PURGE`, `-PURGE`, `AUDIT`, `RECOVER`, `DRAWINGRECOVERY`.
- [ ] Bổ sung `RENAME` cho vật liệu, plot style, viewport configuration và các bảng còn thiếu.

### G. Text, MText, Fields và Annotation

- [ ] Hoàn thiện `TEXT`, `-TEXT`, `MTEXT`, `-MTEXT`, `TEXTEDIT`, `DDEDIT`, `MTEDIT`.
- [ ] Bổ sung MText justification, attachment, columns, tab/indent layout và stacked fractions.
- [ ] Bổ sung `STYLE`, `-STYLE`, SHX font, Big Font và vertical text style.
- [ ] Bổ sung `FIELD`, `UPDATEFIELD`, field trong Text, MText, Dimension và Table.
- [ ] Bổ sung `SPELL` và kiểm tra chính tả đa ngôn ngữ.
- [ ] Hoàn thiện `MLEADER`, `MLEADEREDIT`, `MLEADERALIGN`, `MLEADERCOLLECT`, `MLEADERSTYLE`.
- [ ] Bổ sung nhiều landing, block content, attributes, spline/no-leader style.

### H. Dimension và dung sai

- [ ] Hoàn thiện `DIMLINEAR`, `DIMALIGNED`, `DIMANGULAR`, `DIMARC`, `DIMRADIUS`, `DIMDIAMETER`, `DIMORDINATE`.
- [ ] Hoàn thiện `DIMCONTINUE`, `DIMBASELINE`, `QDIM`, `DIMSPACE`, `DIMCENTER`, `CENTERMARK`, `CENTERLINE`.
- [ ] Bổ sung DIMSTYLE Manager dạng dialog đầy đủ.
- [ ] Bổ sung DIMALTTD, DIMALTTZ, DIMUPT, DIMTXTDIRECTION, jog height, break size.
- [ ] Hoàn thiện associative DIMBREAK khi đối tượng bị di chuyển.
- [ ] Hoàn thiện `DIMEDIT`, `DIMTEDIT`, `DIMOVERRIDE`, `DIMREASSOCIATE`, `DIMDISASSOCIATE`.
- [ ] Bổ sung Geometric Tolerance dialog, LEADER/QLEADER tolerance và chỉnh sửa tolerance frame.

### I. Hatch, Gradient, Region và MLine

- [ ] Mở rộng thư viện Hatch từ 23 mẫu lên bộ mẫu tương thích CAD.
- [ ] Bổ sung Hatch Preview, Inherit Properties, Boundary Preview.
- [ ] Hoàn thiện `HATCH`, `-HATCH`, `HATCHEDIT`, `-HATCHEDIT`, `GRADIENT`.
- [ ] Bổ sung associative hatch khi copy/move boundary.
- [ ] Bổ sung `MPOLYGON` và xử lý vùng phức tạp.
- [ ] Hoàn thiện `MLEDIT`, `MLSTYLE`, `MLINE`.

### J. Layout, Viewport và Plot

- [ ] Hoàn thiện `LAYOUT`, `-LAYOUT`, `MVIEW`, `-VPORTS`, `VPORTS`, `VPCLIP` (engine có offset độc lập viewport/clip boundary).
- [ ] Bổ sung viewport clip boundary di chuyển đồng bộ.
- [ ] Bổ sung viewport annotation scale riêng với model scale.
- [ ] Hoàn thiện `PAGESETUP`, `PLOT`, `-PLOT`, `PUBLISH`, `PUBLISH` nhiều sheet.
- [ ] Bổ sung đọc `.pc3`, `.ctb`, `.stb` nhị phân và cấu hình máy in hệ thống.
- [ ] Hoàn thiện Plot Preview, plot transparency, printable area và margin theo thiết bị.
- [ ] Bổ sung `EXPORTPDF`, `EXPORTPNG`, `EXPORTSVG`, `EXPORTDWG`, `EXPORTDXF` theo version.

### K. Inquiry, Compare và dữ liệu

- [ ] Hoàn thiện `COUNT`, `COUNTLIST`, `COUNTTABLE`, `COMPARE`, `COMPAREEXPORT`.
- [ ] Bổ sung QuickCalc history lưu theo bản vẽ.
- [ ] Hoàn thiện `TIME`, `STATUS`, `DWGPROPS`, `DRAWINGPROPS`.
- [ ] Bổ sung `MEASUREGEOM` đầy đủ Distance, Radius, Angle, Area, Volume.
- [ ] Bổ sung `TABLE`, `TABLEDIT`, `TABLEEXPORT`, `TABLESTYLE`.
- [ ] Bổ sung Formula FIELD, text wrap, custom cell style và break table.
- [ ] Bổ sung Excel Data Link hai chiều; CSV vẫn duy trì.

### L. File format và trao đổi dữ liệu

- [ ] Giữ nguyên unknown DWG entities, objects, XData, reactors, extension dictionaries và colour books.
- [ ] Bổ sung real-file corpus và test mở/lưu/mở lại.
- [ ] Kiểm thử trực tiếp với AutoCAD trước khi công bố tương thích.
- [ ] Bổ sung R12 DXF binary, R14 DWG và Save As version per file.
- [ ] Bổ sung SHX, Big Font, DWF/DWFx, DGN, WMF, EPS, ACIS/STL.
- [ ] Hoàn thiện SVG CSS, `use`, gradient, text và embedded images.
- [ ] DWG import/export cho Web build.

### M. Express Tools, LISP, API và AI

- [ ] Hoàn thiện Express Tools: `SUPERHATCH`, `NCOPY`, `BREAKLINE`, `TXT2MTXT`, `BURST`, `MOCORO`.
- [ ] Bổ sung Visual LISP ActiveX `vlax-*`, `vla-*`, DCL, reactors và Startup Suite.
- [ ] Hoàn thiện `APPLOAD`, `SCRIPT`, `.lsp`, `acad.lsp`, `acaddoc.lsp`.
- [ ] Xây dựng Plug-in API Rust/WASM.
- [ ] Bổ sung CUI/Ribbon/Menu customization và ALIASEDIT lưu lâu dài.
- [ ] Bổ sung Smart Blocks: tìm, gợi ý, thay thế, phát hiện và chuyển đổi.
- [ ] Bổ sung Assistant tích hợp trong ứng dụng.
- [ ] Bổ sung Markup Assist, Markup Import và Trace.
- [ ] Bổ sung Sheet Set Manager, eTransmit, Share và Activity Insights.

### N. 3D — để sau khi phủ lệnh 2D

- [ ] UCS, 3D View, Orbit, ViewCube, Visual Styles (document-level `VISUALSTYLE`/`VSCURRENT` selector foundation exists; renderer integration remains).
- [ ] 3D Solid, Surface, Mesh, Section Plane.
- [ ] UNION, SUBTRACT, INTERSECT 3D.
- [ ] Materials, Lights, Rendering.
- [ ] 3D Dimension, 3D Annotation, Point Cloud và Reality Capture.

## Giai đoạn 2 — Hoàn thiện tương thích dữ liệu

Sau khi phủ lệnh, ưu tiên theo thứ tự:

1. DWG round-trip không mất dữ liệu.
2. XREF, Dynamic Block và Annotation Context.
3. SHX/Big Font, CTB/STB/PC3.
4. PDF/Image Underlay.
5. Table FIELD/Data Link.
6. Bộ file thực tế Việt Nam, châu Á và châu Âu.

## Giai đoạn 3 — Tối ưu sau

Chỉ bắt đầu sau khi lệnh và dữ liệu đã đạt mức sử dụng được:

- Render đa luồng và GPU.
- Spatial index và lazy loading.
- Chia lô xử lý bản vẽ lớn.
- Cache block, hatch, text và dimension.
- Tối ưu DWG 10 MB trở lên.
- Kiểm thử bản vẽ 20.000 ha và tuyến đường 1.000 km.
- Multi-monitor, DPI, WebAssembly và autosave browser.

## Tiêu chí nghiệm thu từng đợt

- Lệnh gọi đúng bằng tên đầy đủ và alias.
- Prompt hiển thị đúng keyword; Enter/Esc/Undo hoạt động.
- Không treo hoặc văng với dữ liệu xấu và dữ liệu lớn trong phạm vi hiện tại.
- Có ít nhất một test tự động cho mỗi nhóm lệnh.
- Mở lại file sau khi lưu không mất đối tượng đã tạo.
- Tài liệu và giao diện không còn tên CADCraft cũ; thương hiệu hiển thị là CAD 2D - TBT 360.

## Thứ tự triển khai kế tiếp

1. Hoàn thiện nhóm lệnh còn thiếu ở A–C.
2. Dynamic Block Authoring và REFEDIT.
3. XCLIP và XREF editing.
4. Text/Field/Dimension/Table nâng cao.
5. Hatch/Region/MLine.
6. Plot/file format.
7. LISP/API/AI/collaboration.
8. Sau cùng mới tối ưu hiệu năng và đa luồng.
