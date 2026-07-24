#define MyAppName "Todo App GPUI"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Your Name"
#define MyAppExeName "todo-app-gpui.exe"

[Setup]
AppId={A8D4A1D4-3E7A-4B6C-9A37-123456789ABC}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}

DefaultDirName={autopf}\Todo App GPUI
DefaultGroupName=Todo App GPUI

OutputDir=installer-output
OutputBaseFilename=TodoApp-GPUI-Setup

Compression=lzma
SolidCompression=yes

ArchitecturesInstallIn64BitMode=x64

[Files]
Source: "target\release\todo-app-gpui.exe"; \
    DestDir: "{app}"; \
    Flags: ignoreversion

[Icons]
Name: "{group}\Todo App GPUI"
Filename: "{app}\todo-app-gpui.exe"

Name: "{commondesktop}\Todo App GPUI"
Filename: "{app}\todo-app-gpui.exe"

[Run]
Filename: "{app}\todo-app-gpui.exe"
Description: "Launch Todo App GPUI"
Flags: nowait postinstall skipifsilent