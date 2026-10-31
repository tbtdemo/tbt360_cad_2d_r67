# BÀN GIAO TBT360 — CAD R65 + QTO R134 (FULL SOURCE, TỰ BUILD)

**Mục đích:** Một bộ tài liệu và mã nguồn đầy đủ để tự biên dịch bản CAD R65 và tiếp tục tích hợp QTO. Đây **không phải** bộ EXE R65 đã biên dịch.

## Cấu trúc

- `SOURCE_CAD_R65/`: source Rust CAD 2D/3D, QTO native, `Cargo.lock`, các `.github/workflows` và giấy phép, lấy từ gói R65 có kiểm tra hình học mới nhất trong cuộc trao đổi.
- `QTO_R134_WEB_WORKSPACE/`: phần QTO PDF/2D/3D độc lập đến R134: đo bằng chuột, trang PDF, hiệu chỉnh tỷ lệ, dấu đo, Undo/Redo và nhật ký. **Chưa được tích hợp hoàn toàn vào EXE Rust R65.**
- `CAD_ORIGINAL_014_REFERENCE/`: các binary Windows `cadcraft.exe`/`cadcraft-cli.exe` gốc 0.14.0 người dùng đã chạy được. **Để đối chiếu hoặc thử CAD nền, không phải bản R65.**
- `REPORTS/`: báo cáo gốc; `SHA256SUMS.txt`: kiểm tra tệp chính.
- `BUILD_CAD_R65_WINDOWS.bat`: chạy trên máy **dùng để build** có Rust 1.95+, MSVC Build Tools; kiểm tra test trước rồi build release, copy binary vào `BUILD_OUTPUT_R65` nếu thành công.
- `CHAY_EXE_R65_SAU_BUILD.bat`: BAT dùng trong thư mục chứa EXE đã build; script build sẽ copy thành `CHAY_TBT360.bat`.
- `CHAY_CAD_GOC_014.bat`: chạy CAD 0.14.0 tham chiếu (đã có EXE thật).
- `CHAY_QTO_WEB_THAM_CHIEU.bat`: chạy workspace QTO độc lập để so sánh tính năng.

## Xây bản phát hành CAD mới

1. Cài Rust >=1.95, MSVC Build Tools và Windows SDK trên **máy build**, hoặc dùng GitHub Actions workflow trong `SOURCE_CAD_R65/.github/workflows/tbt360-r65-windows-portable.yml` trên repository CAD riêng.
2. Chạy `BUILD_CAD_R65_WINDOWS.bat`. Nếu tests/build thất bại, **không phát hành**. R65 còn phụ thuộc Git crates; CI cần truy cập mạng tương ứng.
3. Sau thành công, kiểm tra `BUILD_OUTPUT_R65/cadcraft.exe`, chạy `CHAY_TBT360.bat`, vẽ LINE/CIRCLE/PLINE/ARC, sử dụng Takeoff trong CAD và chạy Undo/Redo/Save/Reopen.
4. Chụp màn hình **EXE R65 thật** tại các tính năng mới. Kiểm tra format DXF/IFC bằng dữ liệu thực, không lấy workspace Chromium thay thế bằng chứng native.
5. Hoàn thiện chuyển QTO PDF/IFC từ web workspace vào app Rust trước khi gọi là **một phần mềm thống nhất**.
6. Đóng `BUILD_OUTPUT_R65/` và source, tài nguyên cần thiết, mẫu demo và kết quả test thành **một ZIP portable**.

## Trạng thái trung thực

- CAD 0.14.0: có Windows EXE gốc thật, được người dùng xác nhận mở thành công.
- CAD R65: **source mới, chưa có Windows EXE đã build và test** trong môi trường này.
- QTO native R65: có code QTO hình học/tổng hợp trong CAD, unit test nguồn; chưa chạy Rust test.
- QTO R134: web workspace độc lập; không có nghĩa các chức năng web đã xuất hiện trong CAD exe.
- GitHub push: **chưa thực hiện**; repository CAD riêng chưa được xác minh với quyền ghi.
- Không dùng EXE cũ để chứng minh chức năng mới.
