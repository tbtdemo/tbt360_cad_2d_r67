# BIM Collaboration Data Model — CAD 360 Việt Nam - TBT

Mục tiêu: cho phép nhiều người thiết kế song song trong cùng dự án/file logic, theo tầng/cao độ, bộ môn và quyền truy cập; hỗ trợ online cộng tác và offline cá nhân.

## Nguyên tắc dữ liệu
- Mọi phần tử thiết kế phải gắn ProjectId, FloorId, Discipline, OwnerAccountId.
- Tọa độ lưu theo Project Coordinates + Floor Elevation + Local Coordinates.
- Dữ liệu local/offline có ChangeId và SyncState để đồng bộ khi online.
- Quyền xem/sửa tách theo dự án, tầng, bộ môn và owner.
- Overlay của người khác là read-only nếu không có quyền sửa.
- Không dùng một file nhị phân duy nhất làm nguồn sự thật khi online; file local là cache/workspace.

## Bảng đề xuất

### Account
- Id UUID
- Email
- DisplayName
- Status
- CreatedAt

### Project
- Id UUID
- Name
- CoordinateOriginX/Y/Z
- RotationDeg
- OwnerAccountId

### ProjectMember
- ProjectId
- AccountId
- Role: Owner / Coordinator / Designer / Viewer
- CanEdit
- CanViewOtherDisciplines
- CanViewOtherFloors

### DisciplineAccess
- ProjectId
- AccountId
- Discipline
- CanView
- CanEdit

### Floor
- Id UUID
- ProjectId
- Name
- SortOrder
- ElevationM
- HeightM

### DesignObject
- Id UUID
- ProjectId
- FloorId
- Discipline
- OwnerAccountId
- EntityType
- LocalX/Y/Z
- GeometryPayload
- Revision
- Deleted
- UpdatedAt

### DesignObjectPermission
- DesignObjectId
- AccountId or Role
- CanView
- CanEdit

### ChangeSet
- Id UUID
- ProjectId
- AccountId
- DeviceId
- BaseRevision
- CreatedAt
- SyncState: Local / Pending / Synced / Conflict

### ChangeItem
- ChangeSetId
- DesignObjectId
- Operation: Create / Update / Delete
- BeforeRevision
- AfterRevision
- Payload

### ViewPreset
- Id UUID
- ProjectId
- AccountId
- Layout: 1 / 2 / 3 / 4
- SlotsJson

### Issue / CoordinationIssue
- Id UUID
- ProjectId
- FloorId
- CreatedBy
- AssignedTo
- DisciplineA
- DisciplineB
- PositionX/Y/Z
- Description
- Status

## Online / Offline
Offline:
1. Đăng nhập lần gần nhất hoặc Local Personal mode.
2. Làm việc trên local cache.
3. Mỗi thay đổi tạo ChangeSet.
4. Không sửa dữ liệu thuộc người khác nếu không có lease/quyền đã cache.

Online:
1. Đồng bộ metadata Project/Floor/Member/Permission.
2. Pull revisions mới.
3. Push ChangeSet local.
4. Nếu Revision khác BaseRevision -> Conflict.
5. Conflict không tự ghi đè; hiển thị compare/accept mine/accept theirs/merge.

## Viewport
Mỗi viewport giữ:
- FloorId
- Discipline filter
- Owner filter
- Overlay opacity
- Current/Reference mode
- Camera/view state

Viewport chính cho phép edit theo scope hiện hành.
Viewport phụ mặc định read-only để xem người khác/tầng khác.

## Lộ trình nối renderer
1. Scope filter trên entity theo FloorId/Discipline/Owner.
2. Overlay renderer với alpha độc lập.
3. Multi viewport dùng chung model cache, khác camera/filter.
4. Presence online: cursor, selection, user color.
5. Object lease/soft lock khi edit.
6. Clash/coordination issue giữa discipline/floor.
