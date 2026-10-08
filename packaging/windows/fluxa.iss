#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceDir
  #error SourceDir is required
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif

[Setup]
AppId={{7A1F6E52-3C2B-4F0E-9B1D-5E8F0A4C2D11}
AppName=Fluxa
AppVersion={#AppVersion}
AppPublisher=Fluxa
DefaultDirName={autopf}\Fluxa
DefaultGroupName=Fluxa
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir={#OutputDir}
OutputBaseFilename=fluxa-{#AppVersion}-windows-x86_64-setup
SetupIconFile=..\icons\fluxa.ico
UninstallDisplayIcon={app}\fluxa-desktop.exe
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
CloseApplications=yes

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Fluxa"; Filename: "{app}\fluxa-desktop.exe"
Name: "{autodesktop}\Fluxa"; Filename: "{app}\fluxa-desktop.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Run]
Filename: "{app}\fluxa-desktop.exe"; Description: "Launch Fluxa"; Flags: nowait postinstall skipifsilent
