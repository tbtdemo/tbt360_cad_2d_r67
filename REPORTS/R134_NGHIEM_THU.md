# R134 — kiểm thử thay đổi tỷ lệ đo bóc, giữ cả hai nhánh Forma/Bluebeam

Baseline: R133, giữ nguyên source Rust và bố cục chính.

## Đã sửa thực
- Khi hiệu chỉnh lại bằng hai điểm chuẩn trên trang đã có phép đo, hệ thống tính lại **mọi** phép đo PDF/ảnh trên đúng trang trước khi commit.
- Mọi giá trị được kiểm tra trước khi ghi; không đổi dữ liệu 3D/modelRows hoặc trang khác.
- Dòng đã xác nhận trở về cần kiểm tra; audit ghi lại giá trị/trạng thái trước và sau.
- Cấm xoá calibration khi trang còn phép đo, tránh khối lượng không có tỷ lệ.
- Snapshot Undo/Redo và JSON Save/Reopen giữ số lượng, tỷ lệ, trạng thái.

## Test đã chạy
- `python TAKEOFF_WORKSPACE/tools/test_r134_recalibrate.py`: PASS Chromium click thật, 10 m -> 20 m; Undo/Redo, Save/Reopen, audit.
- `test_r132_select_delete.py`: PASS.
- `test_r130_drag_evidence.py`: PASS.
- `test_r131_area_grip.py`: PASS.
- `test_multipage.py`: PASS.
- `node --check compiled_app.js`: PASS.
- Ảnh màn hình `TAKEOFF_WORKSPACE/EVIDENCE_R134/01_TRUOC_DOI_TY_LE_10M.png` và `02_SAU_DOI_TY_LE_20M.png` chụp từ Chromium click canvas.

## Giới hạn phát hành
- **Chưa đạt toàn bộ nghiệm thu sản phẩm**: không có cargo/rustc, không build/kiểm thử TBT360 Rust EXE; QTO IFC runtime và AI phân lớp cấu kiện kỹ thuật chưa đủ evidence.
- PASS chỉ xác nhận workspace trình duyệt trên mẫu kiểm thử, không tương đương Autodesk Forma Takeoff hay Bluebeam Revu.
- Không có EXE Windows mới; BAT chỉ mở workspace trình duyệt, không phải trình cài phần mềm native.
