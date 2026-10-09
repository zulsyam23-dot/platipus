; Inno Setup script for the Platipus toolchain.
;
; It packages the three binaries this workspace builds -- `platipus`, `plt` and
; `p2lt` -- and nothing else. There is no runtime to install: a Platipus
; program is HTML, CSS and JavaScript in a directory, and the compiler needs
; Node on the machine only for `platipus test` and `platipus dev`.
;
; Build the binaries first:
;
;   cargo build --release -p platipus-cli -p platipus-p2lt
;
; then compile this script:
;
;   iscc installer\platipus.iss
;
; The output lands in installer\Platipus-0.1.0-setup.exe.

#define AppName "Platipus"
#define AppVersion "0.1.0"
#define AppPublisher "Platipus"
#define AppURL "https://github.com/zulsyam23-dot/platipus"
#define AppExeName "platipus.exe"

[Setup]
AppId={{7C4E2B1A-9D3F-4E58-A6B2-0F1D8C5E7A31}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
VersionInfoVersion={#AppVersion}

; Per-user by default. Installing the compiler machine-wide would put three
; `.exe` files named after ordinary English words in a directory on the system
; path, where `plt` in particular is short enough to collide with something.
; A machine-wide install is one page away, in [Tasks].
;
; `lowest` is deliberate and pairs with the `HKCU` write in [Registry]: the
; PATH entry belongs to the user who chose the task, not to the machine.
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
DisableDirPage=no
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=.
OutputBaseFilename=Platipus-{#AppVersion}-setup
SetupIconFile=platipus.ico
WizardStyle=modern
WizardImageFile=wizard.bmp
UninstallDisplayIcon={app}\{#AppExeName}
UninstallDisplayName={#AppName}
Compression=lzma2/max
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "addtopath"; Description: "Add the Platipus commands to my PATH"; \
    GroupDescription: "Integration:"; Flags: checkedonce
Name: "desktopicon"; Description: "Create a desktop shortcut"; \
    GroupDescription: "Shortcuts:"; Flags: unchecked

[Files]
; The binaries. `plt` is the same program under the short name, because
; `p2lt build` and friends are typed often enough that `platipus` is a lot to
; type for the same thing.
Source: "..\target\release\platipus.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\plt.exe";      DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\p2lt.exe";     DestDir: "{app}"; Flags: ignoreversion

; Documentation that is worth having next to the tools. The rest of `docs` is
; design history, which belongs in the repository rather than in Program Files.
Source: "..\README.md";        DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "..\docs\guide.md";    DestDir: "{app}\docs"; Flags: ignoreversion
Source: "..\docs\library.md";  DestDir: "{app}\docs"; Flags: ignoreversion
Source: "..\docs\libplt-format.md"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "..\docs\package-registry.md"; DestDir: "{app}\docs"; Flags: ignoreversion

[Dirs]
; Where `p2lt add` and `p2lt install` keep packages. Created here so a user who
; runs `p2lt` before their first install gets a clear error rather than a
; failure partway through a clone.
Name: "{localappdata}\{#AppName}\store\packages"

[Icons]
Name: "{group}\{#AppName}";        Filename: "{app}\{#AppExeName}"; \
    Parameters: "help"; WorkingDir: "{app}"; IconFilename: "{app}\{#AppExeName}"
Name: "{group}\Package manager";   Filename: "{app}\p2lt.exe"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}";  Filename: "{app}\{#AppExeName}"; \
    Parameters: "help"; Tasks: desktopicon

[Registry]
; PATH for the current user only, and only when the task was chosen. Writing the
; machine PATH would need elevation for something that is a developer tool.
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; \
    ValueData: "{olddata};{app}"; \
    Check: NeedsAddPath(ExpandConstant('{app}')); Tasks: addtopath

[Run]
; Run the freshly installed binary once, so a broken install is caught here
; rather than the first time the user types a command. The window is hidden and
; nothing is shown afterwards: this is a check, not a demo.
Filename: "{app}\{#AppExeName}"; Parameters: "version"; \
    Description: "Check that {#AppName} runs"; \
    Flags: runhidden waituntilterminated; StatusMsg: "Checking the installation..."

[Code]
/// Whether `{app}` is already on the user's PATH, so the task is only offered
/// when it would change something.
///
/// The PATH change is written to `HKCU\Environment` and deliberately not
/// broadcast with `WM_SETTINGCHANGE`. Broadcasting would only help programs
/// that re-read the environment on that message; a terminal the user opens
/// afterwards reads it from the registry either way. Skipping it keeps the
/// install free of a call into user32 whose signature has to be exactly right
/// on both bitnesses for very little gain.
function NeedsAddPath(const Dir: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  // Compared with a leading and trailing separator on both sides so that a
  // directory which merely shares a prefix is not mistaken for a match.
  Result := Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;
