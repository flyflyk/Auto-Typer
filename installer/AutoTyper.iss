; Inno Setup script for AutoTyper. Built by .github/workflows/release.yml:
;   ISCC.exe /DAppVersion=1.2.3 installer\AutoTyper.iss
; Per-user install (no admin prompt) into %LOCALAPPDATA%\Programs\AutoTyper.
;
; The app runs this installer silently to update itself:
;   /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /RELAUNCH=1 [/FROM=<old portable exe>]
; /RELAUNCH=1 starts the installed app afterwards; /FROM is passed on to it as
; --migrated-from so it can offer to delete the portable copy.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
; Never change AppId: Windows uses it to recognise later versions as the same app.
AppId={{09450435-EDD5-4BD4-A4E9-F83080E611D1}
AppName=AutoTyper
AppVersion={#AppVersion}
AppVerName=AutoTyper {#AppVersion}
AppPublisher=flyflyk
AppPublisherURL=https://github.com/flyflyk/Auto-Typer
AppSupportURL=https://github.com/flyflyk/Auto-Typer/issues
AppUpdatesURL=https://github.com/flyflyk/Auto-Typer/releases
VersionInfoVersion={#AppVersion}
PrivilegesRequired=lowest
DefaultDirName={localappdata}\Programs\AutoTyper
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
UsePreviousTasks=yes
CloseApplications=force
RestartApplications=no
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\AutoTyper.exe
UninstallDisplayName=AutoTyper
OutputDir=..\dist
OutputBaseFilename=AutoTyper-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\target\release\auto-typer.exe"; DestDir: "{app}"; DestName: "AutoTyper.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\AutoTyper"; Filename: "{app}\AutoTyper.exe"
Name: "{autodesktop}\AutoTyper"; Filename: "{app}\AutoTyper.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\AutoTyper.exe"; Description: "{cm:LaunchProgram,AutoTyper}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\AutoTyper.exe"; Parameters: "{code:RelaunchArgs}"; Flags: nowait; Check: ShouldRelaunch

[UninstallDelete]
; The app's own small state (the dismissed install offer).
Type: filesandordirs; Name: "{localappdata}\AutoTyper"

[Code]
function ShouldRelaunch: Boolean;
begin
  Result := WizardSilent and (ExpandConstant('{param:RELAUNCH|0}') = '1');
end;

function RelaunchArgs(Param: String): String;
var
  From: String;
begin
  From := ExpandConstant('{param:FROM|}');
  if From <> '' then
    Result := '--migrated-from "' + From + '"'
  else
    Result := '';
end;
