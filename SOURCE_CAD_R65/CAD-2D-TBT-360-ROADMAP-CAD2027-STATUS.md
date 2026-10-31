# CAD 2D - TBT 360 — Trạng thái triển khai đa nhóm

## Quy tắc đợt hiện tại

Đợt 1 ưu tiên phủ lệnh CAD 2027 và prompt/alias/test. Chưa ưu tiên tối ưu đa luồng, GPU, bộ nhớ hoặc renderer. Chỉ đánh dấu một chức năng đã làm khi mã nguồn có triển khai thật; không dùng alias để giả nhận là đã có tính năng.

## Đã thực hiện trong đợt 1

- `BREAKATPOINT` có alias `BRP` và quy trình tương tác chọn đối tượng rồi chọn điểm tách.
- `POINT` có lệnh nhiều điểm qua `MULTIPOINT`/`MPOINT`; `REVCLOUD` có `RC`/`RCLOUD`.
- `ARRAY`, `ARRAYRECT`, `ARRAYPOLAR`, `ARRAYEDIT`, `ARRAYCLOSE` và Group được bổ sung alias CAD gần hơn: `ARRAYCLASSIC`, `ARRECT`, `ARPOL`, `AE`, `AEC`, `UNG`, `GED`.
- `TEXTEDIT`/`DDEDIT`/`MTEDIT` hỗ trợ chỉnh nhiều đối tượng qua `handles` ở dạng JSON/API và trả số lượng đối tượng đã thay đổi.
- XREF/Layer có thêm alias thao tác: `XREFATTACH`, `XO`, `LAYMC`, `LAYMATCH`, `LAYUNISOL`.
- `HATCHEDIT` có thể tạo lại biên dạng `REGION` thật từ hatch; đã thêm regression test.
- `PAGESETUP` có alias `PSETUP`; `EXPORTPDF` có alias `PDFOUT`.
- `MVIEW Restore` đã có triển khai và regression test từ đợt trước.

## Đợt rà soát đa luồng tiếp theo — đã triển khai phần nền an toàn

- `XCLIP`/`XC`: rectangular XCLIP thật cho XREF reference, lưu boundary bằng XData `TBT360_XCLIP`, renderer cắt các primitive được hỗ trợ, có xóa clip và test. Polygonal XCLIP chưa có.
- `blocks.dynamic.validate`: kiểm tra graph Dynamic Block, node trùng/mất, action tham chiếu sai, visibility và lookup table; bảo toàn dữ liệu chưa hiểu, chưa phải Dynamic Block Authoring đầy đủ.
- `ARRAYGRIPS`/`ARRAYG` + `ARRAYGRIPSET`: metadata grip và editor UI trực tiếp cho associative Rectangular/Polar Array; Ctrl+click suppression đã có. Còn thiếu một số semantics Key point/base point và parity 3D levels.
- `IMAGEATTACH`/`IMA`/`IMAGE`, `IMAGECLIP`/`ICLIP`: lưu metadata ảnh, clip polygon pixel và xóa clip; R32 có dialog attach trực tiếp từ Tool Sets và frame placement trên canvas; renderer vẫn chưa nạp bitmap pixel trực tiếp.
- PC3/CTB/STB: validation phần mở rộng và phát hiện dữ liệu binary, trả lỗi rõ ràng; chưa đọc/ghi binary Autodesk đầy đủ.
- `CAPABILITIES`/`CAPS`: ma trận read-only nêu chính xác trạng thái AI/3D.
- `SMARTBLOCKS`/`SMARTBLOCK`/`AIBLOCKS`: sinh đề xuất Smart Block xác định được từ hình học lặp, chỉ đọc và không tự ghi đè bản vẽ.
- `FIELD`/`INSERTFIELD` và `UPDATEFIELD`/`UF`: field liên kết cơ bản tới thuộc tính entity (handle, layer, length, area, radius, diameter, circumference), có cache metadata và cập nhật lại Text/MText.
- `blocks.dynamic.author`: draft authoring graph an toàn trong bộ nhớ với inspect/replace/addParameter/addAction, giới hạn kích thước và validation; chưa persist vào DWG.
- `ARRAYGRIPSET`/`ARRAYGS`: chỉnh thông số associative Rectangular/Polar Array qua API, cập nhật lại metadata/item count và kiểm tra loại tham số.
- `PDFATTACH`/`PDFA`/`ATTACHPDF`, `PDFCLIP`/`PDFC`: lưu metadata underlay PDF và clip polygon; R32 có dialog attach + khung placement/clip hiển thị trên canvas; chưa rasterize/hiển thị nội dung trang PDF.
- `PLOTTERINFO`/`PINFO`: mô tả profile plotter và báo rõ PC3/CTB/STB binary chưa hỗ trợ.
- Đã thêm script `scripts/cad2027_command_audit.py`: rà command/alias và phát hiện alias trùng. Lượt audit hiện không có alias collision.

## Chưa được giả nhận là hoàn tất

- XCLIP polygonal, REFEDIT đầy đủ cho XREF: cần mở rộng selection/snap và giao diện làm việc tại chỗ.
- Dynamic Block Authoring: Parameter, Action, Constraint, Visibility, Lookup.
- Associative ARRAYPATH, grip số lượng/khoảng cách và xóa item bằng Ctrl+click.
- Group bounding box/single grip và Copy Group tạo nhóm mới.
- PC3/CTB/STB nhị phân, PDF raster renderer, IMAGEATTACH bitmap renderer, SHX/Big Font.
- Smart Blocks AI xếp hạng/convert tự động, Sheet Set Manager, eTransmit và 3D solid/surface/mesh/UCS/visual styles.

## Nhóm triển khai tiếp theo

1. Phủ nốt prompt/keyword/Undo/Esc của lệnh vẽ và hiệu chỉnh.
2. Dynamic Block Authoring và REFEDIT.
3. XCLIP và XREF editing.
4. Text/Field/Dimension/Table nâng cao.
5. Hatch/Region/MLine nâng cao.
6. Plot và file format.
7. LISP/Express Tools/API/AI.
8. Sau cùng mới tối ưu đa luồng, GPU, cache và bản vẽ cực lớn.

## Đợt tiếp theo đã rà soát

- `ATTEDIT`/`EATTEDIT` nhận thêm vị trí, chiều cao, góc xoay, kiểu chữ, layer và màu cho thuộc tính; vẫn giữ kiểm tra tag và không ghi nếu dữ liệu màu sai.
- Các khoảng trống còn lại được giữ nguyên trạng thái rõ ràng, không đăng ký alias giả để đánh dấu hoàn thành.
- Audit hiện tại: 440 command specs, 246 aliases, không có alias collision; nhóm 2D/blocks/annotation/layout/automation không còn mục command coverage trong ma trận tĩnh.

## Đợt chạy song song tiếp theo

- `REFEDIT`/`REFSET`/`REFCLOSE`: bổ sung alias CAD và hỗ trợ block reference scale X/Y không đồng nhất; XREF nguồn ngoài vẫn không ghi ngược.
- `GROUPEDIT copy`: sao chép định nghĩa Group an toàn mà không nhân đôi hình học.
- `ATTACH`: điều phối thật sang IMAGE/PDF và lưu metadata DWF/DGN/WMF/EPS; renderer các định dạng này vẫn chưa có.
- `HATCHEDIT inheritFrom`: kế thừa pattern/scale/angle/style/gradient/origin/background mà không thay loop đích.
- `TABLESTYLE`/Cell Style: bổ sung `wrap` và truyền chiều rộng ô vào MText layout để tự xuống dòng.
- `SPELL`/`SP`: quét TEXT/MTEXT theo dictionary, chỉ báo cáo và không tự sửa bản vẽ.
- Audit sau đợt: 441 command specs, 250 aliases, không có alias collision.

## Đợt chạy song song hiện tại

- `LASSO`/`SELECTLASSO`: chọn vùng polygon tự do trong engine, hỗ trợ crossing và cập nhật selection.
- `SELECTALLLAYOUTS`/`LAYOUT.SELECTALL`: chọn toàn bộ layout theo thứ tự tab để PUBLISH/PLOT/UI dùng tiếp.
- `INTERSECT`: giao hai Region polygon lồi; các vùng lõm/lỗ/đường cong tổng quát vẫn bị từ chối an toàn.
- `STYLESMANAGER validate` và `PLOTTERMANAGER validate`: kiểm tra profile JSON/CADCraft CTB/STB/plotter; binary Autodesk vẫn báo chưa hỗ trợ.
- `UCS`/`UC`: save/set/restore/list/delete/world và lưu bảng UCS; renderer chưa chuyển đổi WCS theo UCS.
- `DIMALTTD`/`DIMALTTZ`: model, DXF round-trip và renderer tolerance cho alternate units.
- Audit sau đợt: 444 command specs, 253 aliases, không có alias collision; nhóm 3D còn thiếu mesh, sectionplane, solid, surface, visualstyles.

## Đợt tiếp theo hiện tại

- `ARRAYPATH` associative: model `PathArray` giữ path handle, điểm phân bố, rotation theo path và round-trip serialization; có `ARRAYPATHREFRESH` thủ công sau khi sửa path, nhưng chưa auto-refresh theo dependency graph.
- `DIMUPT` và `DIMTXTDIRECTION`: model, renderer và DXF group codes đã bổ sung.
- `UNION`/`SUBTRACT` Region: hỗ trợ vùng chữ nhật trục song song chồng lấn; hình học cong/lõm/lỗ vẫn từ chối an toàn.
- `STYLESMANAGER convert` và `PLOTTERMANAGER convert`: chuẩn hóa profile JSON CADCraft; binary PC3/CTB/STB Autodesk vẫn chưa đọc trực tiếp.
- `GROUPS.BOUNDS`: bounding box, center và single-grip metadata cho Group.
- `UCS` đã có từ đợt trước; `3DFACE` tạo entity 3D thật với Z và hidden edges.
- Audit sau đợt: 446 command specs, 254 aliases, không có alias collision; nhóm 3D còn thiếu mesh, sectionplane, solid, surface, visualstyles.

## Đợt tiếp theo hiện tại

- `ARRAYPATHREFRESH`/`ARRAYPATHR`: cập nhật lại điểm phân bố và rotation theo path sau khi path thay đổi.
- `MESH`: tạo mesh dạng facet từ vertices/faces, kiểm tra chỉ số và lưu qua các mặt 3DFACE; R32 bổ sung FACETSUBDIVIDE3D để chia triangle/quad thành 4 facet native; vẫn chưa phải ACIS/subdivision-surface kernel riêng.
- `VISUALSTYLE`/`VSCURRENT`/`VSTYLE`: lưu và truy vấn các visual style chuẩn trong header/sysvar; renderer 3D chưa áp dụng đầy đủ.
- `UNION`/`SUBTRACT`/`INTERSECT`: giữ giới hạn vùng chữ nhật trục song song/polygon lồi và báo lỗi cụ thể cho hình học chưa hỗ trợ.
- PC3/CTB/STB: nhận diện binary signature và trả cảnh báo có cấu trúc; JSON profile vẫn hỗ trợ validate/convert.
- Audit sau đợt: 449 command specs, 257 aliases, không có alias collision; còn thiếu sectionplane, solid, surface và renderer visual styles đầy đủ.

## Đợt Hatch/Viewport/Group/Attribute/Data Extraction hiện tại

- Hatch library tăng lên 28 mẫu; `PURGE` hiển thị rõ hỗ trợ `mlinestyles`.
- `REGIONCHECK`/`REGIONDIAG`: kiểm tra loop, diện tích, bounding box, self-intersection và giới hạn chống treo.
- UNION/SUBTRACT/INTERSECT: hỗ trợ nhiều loop disjoint/nested và rectangle/convex an toàn; trường hợp phức tạp trả chẩn đoán cụ thể.
- `GROUPDISPLAYMODE`/`GROUPDISPLAY`, `GROUPS.BOUNDS`, `GROUPEDIT copy`: trạng thái, bbox/grip và copy membership đã có API.
- `VPCLIP`: offset độc lập boundary/viewport/both.
- `SELECTALLLAYOUTS` và PUBLISH đã dùng danh sách layout được chọn.
- `ATTDEF`/`BATTMAN`/`ATTSYNC`: Verify, Preset, Lock Position và multiline attribute.
- `DATAEXTRACTION`: nested blocks, transform vị trí, giới hạn đệ quy và `.dxe` JSON template.
- Audit sau đợt: 451 command specs, 259 aliases, không alias collision; 2D/layout/blocks coverage đầy đủ theo ma trận tĩnh.

## Đợt Plot / In ấn / Xuất nhập file và regression hiện tại

- `STYLESMANAGER` và `PLOTTERMANAGER`: profile JSON hợp lệ được validate/convert/round-trip; JSON hỏng trả diagnostic có cấu trúc; binary PC3/CTB/STB được nhận diện và cảnh báo, không giả giải mã.
- STL: đọc ASCII/binary, xuất binary từ Face3d, kiểm tra kích thước/count và giới hạn 1.000.000 tam giác.
- DXF binary/R12/R14: giữ detection/round-trip hiện có; ACIS/B-Rep vẫn giữ foreign/raw data do chưa có kernel hình học.
- `DWG.DIAGNOSTICS`/`DWG.INFO`/`DWGCAPABILITIES`: nhận diện magic/version/kích thước và báo rõ Web chưa decode DWG.
- SVG: CSS class `display:none`/`hidden` được xử lý khi đọc; các CSS paint rule phức tạp vẫn báo giới hạn.
- Regression audit: giới hạn DATAEXTRACTION 1.000.000 reference, alias DX/EATTEXT/ARRAYPATHR, Region serde/NaN validation.
- Audit sau đợt: 454 command specs, 267 aliases, không alias collision; còn thiếu sectionplane, solid, surface và renderer visual styles đầy đủ.

## Giới hạn kiểm chứng

Môi trường hiện tại không có `cargo`/`rustc`, nên chưa thể biên dịch source hoặc tạo `.exe` mới. Các thay đổi đã được kiểm tra tĩnh và thêm test trong source; cần chạy `cargo test` và build Windows ở môi trường có Rust toolchain trước khi phát hành bản chạy.
