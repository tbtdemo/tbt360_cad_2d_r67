# CAD 2D TBT – bộ mã nguồn phát triển

Đây là bộ mã nguồn đầy đủ của ứng dụng CAD 2D TBT, giữ kiến trúc CAD desktop/web bằng Rust và các thư viện xử lý bản vẽ trong thư mục `crates/`.

## Quy ước tệp bản vẽ

- Bản vẽ mới mặc định dùng tên `Drawing.tbt360vn.dwg`.
- Người dùng có thể đặt tên như `Mặt bằng.tbt360vn.dwg`.
- Phần `.dwg` vẫn được giữ ở cuối để phần mềm CAD khác nhận diện là DWG.
- CAD 2D TBT vẫn đọc và ghi các tệp `.dwg`, `.dxf` và các định dạng xuất hình ảnh được hỗ trợ.

## Cấu trúc chính

- `apps/cadcraft/`: ứng dụng desktop.
- `apps/cadcraft-web/`: bản chạy trên trình duyệt/WebAssembly.
- `apps/cadcraft-cli/`: công cụ dòng lệnh và tự động hóa.
- `crates/engine/`: lệnh CAD, phiên làm việc, chọn đối tượng, ràng buộc.
- `crates/io/`: đọc/ghi DXF, DWG và xuất PDF/SVG/PNG.
- `crates/ui-egui/`: giao diện, bảng công cụ, hộp thoại, lệnh và ngôn ngữ.
- `crates/doc/`, `crates/geom/`, `crates/render/`: mô hình bản vẽ, hình học và hiển thị.

## Biên dịch bản desktop

```text
cargo run --release -p cadcraft -- --sample
```

Để tạo bản phát hành Windows 64-bit:

```text
cargo build --release -p cadcraft
```

Tệp thực thi sẽ nằm trong `target/release/`. Môi trường build cần Rust theo phiên bản được ghi trong `README.md`.

## Điểm đã tùy chỉnh

- Tên hiển thị cửa sổ: `CAD 2D TBT`.
- Định dạng lưu mặc định: DWG.
- Hậu tố lưu mặc định: `.tbt360vn.dwg`.
- Bộ đọc/ghi vẫn dùng phần mở rộng cuối `.dwg`, nên dữ liệu vẫn là DWG tương thích.

## Dữ liệu lớn và đa luồng

Bộ render đã có đường dựng display-list song song bằng Rayon cho mô hình lớn; canvas có worker `cadcraft-regen` để không khóa luồng giao diện khi tái dựng hình. Các luồng mở file, autosave và điều khiển MCP cũng chạy nền. Không chia sẻ trạng thái giao diện giữa các luồng; kết quả render được ghép lại theo thứ tự bản vẽ.

Tạo bộ dữ liệu kiểm tra:

```text
python scripts/tbt360_stress_test.py --out stress-fixtures
```

Lệnh tạo hai tệp DXF với tọa độ chiếu đại diện châu Âu và châu Á, một khu 20.000 ha và tuyến giao thông 1.000 km. Khi đã build CLI, kiểm tra tiếp bằng:

```text
python scripts/tbt360_stress_test.py --out stress-fixtures --cli target/release/tbt360-cli.exe
```

## Kết nối và trao đổi dữ liệu

- Mở/lưu DWG và DXF; tệp `*.tbt360vn.dwg` vẫn có nội dung DWG.
- Xuất SVG, PNG và PDF.
- CLI dùng cho chuyển đổi, kiểm tra, script và xử lý hàng loạt.
- MCP/control server cho phép phần mềm khác gửi lệnh, đọc trạng thái bản vẽ và lấy ảnh render.
- Tệp thử nghiệm lớn nằm trong `stress-fixtures/` sau khi chạy script; cần kiểm tra thêm trên máy Windows có GPU và bộ nhớ thực tế trước khi đưa vào sản xuất.

## Lệnh tắt và nhật ký lỗi

- Bộ gợi ý lệnh tự hoàn thành khi gõ và nhấn `Tab`, theo kiểu CAD: `L`, `PL`, `C`, `REC`, `M`, `CO`, `RO`, `SC`, `MI`, `O`, `TR`, `EX`, `F`, `CHA`, `X`, `J`, `Z`, `P`, `H`, `B`, `I`, `LA`, `PE`, `DLI`, `DAL`, `DRA`, `DDI` và các tên lệnh đầy đủ.
- Dòng lệnh giữ lịch sử thao tác; nhấn `F2` hoặc bấm vùng lịch sử để mở rộng.
- Nhấp phải vào dòng lệnh → `Copy History` để sao chép log gửi kiểm tra. Lỗi phát sinh từ lệnh sẽ có tiền tố `[ERROR]`.
- Có thể gọi lệnh `TBTLOG` (bí danh `LOG` hoặc `ERRLOG`) để lấy JSON log có giới hạn, phù hợp gửi qua CLI/MCP.
- Có thể tăng số dòng lịch sử trong menu dòng lệnh; dữ liệu log trong phiên được giữ tối đa 5.000 dòng.
