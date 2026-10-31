# CAD 360 Việt Nam - TBT — Release Control Matrix

Mục tiêu: một ứng dụng CAD native, một cửa sổ, các nghiệp vụ tích hợp bằng tab/menu, một bộ cài Windows duy nhất. Không dùng updater CADCraft ở runtime.

## Luồng 1 — Build/Core
- [x] Giữ dependency graph R65 cố định; không dùng cargo generate-lockfile trong CI.
- [x] Gỡ craft-update khỏi Cargo.toml/Cargo.lock/runtime.
- [x] CI có chốt chặn updater CADCraft quay lại runtime.
- [ ] cargo check --workspace --locked PASS trên commit hiện tại.
- [ ] cargo test --workspace --locked --no-fail-fast PASS.
- [ ] Release build PASS.

## Luồng 2 — Giao diện thống nhất
- [x] CAD là ứng dụng chính duy nhất.
- [x] Takeoff chuyển vào tab chính CAD/ĐO BÓC, dùng cùng CadApp/session.
- [x] Bỏ launcher QTO Web/CAD gốc/EXE R65 bên ngoài khỏi luồng phát hành chính.
- [ ] Chuyển các nghiệp vụ QTO R134 cần thiết sang tab/menu native.
- [ ] Kiểm thử Layers/Properties/Table và thao tác đa lựa chọn.
- [ ] Kiểm thử Save/Reopen, Undo/Redo và DXF/DWG end-to-end.

## Luồng 3 — Việt hóa + TBT hóa
- [x] Tên cửa sổ/sản phẩm chuyển sang CAD 360 Việt Nam - TBT.
- [x] Updater CADCraft bị loại khỏi runtime.
- [x] About được TBT hóa nhưng vẫn giữ ghi nhận/giấy phép mã nguồn mở.
- [x] Có vi.tsv và đã bổ sung nhãn workspace chính.
- [ ] Hoàn thiện 100% chuỗi UI người dùng nhìn thấy: menu, bảng, dialog, tooltip, thông báo.
- [ ] Rà tất cả chuỗi CADCraft còn xuất hiện ở runtime UI.

## Luồng 4 — Test + phát hành
- [x] Workflow kiểm tra menu, i18n, Takeoff.
- [x] Workflow chạy full workspace test trước release.
- [x] Chỉ phát hành một Setup: CAD360_VietNam_TBT_Setup.exe.
- [ ] CI mới nhất PASS toàn bộ gate.
- [ ] Setup cài/gỡ thành công trên Windows sạch.
- [ ] Smoke test mở CAD, thao tác cơ bản, lưu/mở lại, Takeoff.
- [ ] Ký số bộ cài/phần mềm trước phát hành chính thức.

## Quy tắc kiểm soát
1. Không đánh dấu hoàn thành nếu chưa có bằng chứng compile/test/CI hoặc test Windows thực tế.
2. Không đổi dependency lớn khi sửa một lỗi cục bộ.
3. Không xóa attribution/license của mã nguồn mở; chỉ đổi branding runtime phù hợp.
4. Không đóng gói QTO Web/CAD gốc thành sản phẩm riêng cho người dùng.
5. Mỗi lỗi sửa phải có regression test khi có thể.
