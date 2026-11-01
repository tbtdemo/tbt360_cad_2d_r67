; CAD360 Unified R67 installer: one Windows application containing CAD 2D, BIM, IFC and QTO.
#define MyAppName "CAD 360 Việt Nam - TBT"
#define MyAppVersion "0.68.0"
#define MyAppPublisher "Công ty CP CN TBT Việt Nam"
[Setup]
AppId={{0B643D1A-795E-4C0A-A36A-4AF2C03D1E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\CAD 360 Viet Nam - TBT
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\CAD360.exe
OutputDir=..\dist
OutputBaseFilename=CAD360_VietNam_TBT_Setup
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
DisableProgramGroupPage=yes
[Files]
Source: "..\SOURCE_CAD_R65\target\release\cadcraft.exe"; DestDir: "{app}"; DestName: "CAD360.exe"; Flags: ignoreversion
Source: "..\SOURCE_CAD_R65\target\release\cadcraft-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
[Icons]
Name: "{autoprograms}\CAD 360 Việt Nam - TBT"; Filename: "{app}\CAD360.exe"
Name: "{autodesktop}\CAD 360 Việt Nam - TBT"; Filename: "{app}\CAD360.exe"; Tasks: desktopicon
[Tasks]
Name: "desktopicon"; Description: "Tạo biểu tượng ngoài màn hình"; GroupDescription: "Tùy chọn bổ sung:"
[Run]
Filename: "{app}\CAD360.exe"; Description: "Mở CAD 360 Việt Nam - TBT"; Flags: nowait postinstall skipifsilent
