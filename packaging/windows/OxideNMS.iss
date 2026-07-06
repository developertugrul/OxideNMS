#define AppName "OxideNMS"
#ifndef AppVersion
#define AppVersion "0.0.0"
#endif
#ifndef SourceExe
#define SourceExe "..\..\target\x86_64-pc-windows-msvc\release\oxidenms.exe"
#endif
#ifndef OutputDir
#define OutputDir "..\..\dist"
#endif
#ifndef OutputBaseFilename
#define OutputBaseFilename "OxideNMS-windows-amd64-setup"
#endif

[Setup]
AppId={{9E8D2D50-3491-4AC9-BF93-1A9B982DA75E}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=OxideNMS
AppPublisherURL=https://github.com/developertugrul/OxideNMS
AppSupportURL=https://github.com/developertugrul/OxideNMS/issues
AppUpdatesURL=https://github.com/developertugrul/OxideNMS/releases/latest
DefaultDirName={autopf}\OxideNMS
DefaultGroupName=OxideNMS
DisableProgramGroupPage=yes
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBaseFilename}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
UninstallDisplayIcon={app}\OxideNMS.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "OxideNMS.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\OxideNMS"; Filename: "{app}\OxideNMS.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\OxideNMS"; Filename: "{app}\OxideNMS.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\OxideNMS.exe"; Description: "Launch OxideNMS"; Flags: nowait postinstall skipifsilent
