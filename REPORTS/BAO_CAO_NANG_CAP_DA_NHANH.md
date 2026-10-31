# R65 — kiểm tra đa nhánh CAD / QTO / Windows

## Nhánh CAD/QTO native
- Giữ ứng dụng CAD Rust/egui R65 làm nền source.
- Từ chối diện tích Polyline kín có dây cung tự cắt, cạnh độ dài 0; không dùng diện tích sai để lập QTO.
- Bỏ qua dữ liệu Polyline3D ít hơn hai đỉnh hoặc tọa độ không hữu hạn.
- Thêm hai unit test mới: đường tự cắt và cạnh trùng.
- Chưa chạy unit test Rust: môi trường không có Cargo/Rust. Đường cong bulge giao cắt phức tạp vẫn cần kiểm tra hình học chuyên biệt.

## Nhánh Windows / portable
- Giữ nguyên EXE cadcraft 0.14.0 chính hãng do người dùng đưa vào, có thể chạy CAD hiện có.
- BAT ngoài cùng mở một ứng dụng CAD duy nhất; không tự động chạy QTO HTML thứ hai.
- EXE 0.14.0 KHÔNG chứa code QTO native R65 vừa chỉnh.
- Muốn dùng QTO R65 trong EXE, phải build lại source trên Windows và kiểm tra GUI.

## Kết quả tự kiểm
- ZIP integrity và kiểm kê tệp: chạy trong container.
- Kiểm thử Windows EXE/Rust: CHƯA CHẠY / CHƯA PASS.
- Không tuyên bố đã tích hợp QTO vào binary 0.14.0.
