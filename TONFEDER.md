# tonfeder-desktop

Repo zur Desktop-App von Tonfeder: Taste halten, sprechen, loslassen, der Text steht am Cursor. Alles läuft lokal auf dem eigenen Rechner.

Planung, Design und Logo liegen im Schwester-Repo [tonfeder](https://github.com/1MauriceWolfrum-tech/tonfeder).

## Grundlage

Dieser Code basiert auf [Handy](https://github.com/cjpais/Handy) von CJ Pais, MIT-Lizenz (siehe `LICENSE`). Ausgangsstand: Commit `5a9451e` vom 21.10.2026 (nach v0.9.8).

- Der Lizenzhinweis von Handy bleibt erhalten.
- Name und Logo „Handy“ sind nicht frei verwendbar und werden durch Tonfeder ersetzt.
- `README.md` ist die unveränderte README von Handy. Eigene Hinweise stehen in dieser Datei.

Updates aus dem Original holen:

```bash
git fetch upstream
git merge upstream/main
```

(`upstream` = https://github.com/cjpais/Handy, `origin` = dieses Repo)

## Unter Windows bauen

Voraussetzungen: Visual Studio Build Tools 2022 (C++), Rust, Bun, CMake, Vulkan SDK (siehe `BUILD.md`).

```powershell
$env:CARGO_TARGET_DIR = 'C:\tf'   # kurzer Pfad gegen die 260-Zeichen-Grenze von Windows
bun install
bun run tauri dev                  # Entwicklung
bun run tauri build --no-bundle    # eigenständige exe: C:\tf\release\tonfeder.exe
bun run tauri build --bundles nsis # Installer: C:\tf\release\bundle\nsis\Tonfeder_<version>_x64-setup.exe
```

**Nur beim Bauen aus der Claude-App heraus** zusätzlich:

```powershell
$env:LOCALAPPDATA = 'C:\tf\appdata'
```

Grund: Die Claude-App läuft als Windows-App-Paket (MSIX). Windows leitet Schreibzugriffe ihrer Kindprozesse auf `%LOCALAPPDATA%` in einen versteckten Paketordner um. `transcribe-cpp-sys` legt dort eine kurze Junction an (`%LOCALAPPDATA%\tcs`), die MSBuild dann nicht findet. Fehlerbild: `MSB1009: Die Projektdatei ist nicht vorhanden` beim CMake-Compilertest. In einem normalen Terminal tritt das nicht auf.

Die fertige App nicht aus der Claude-App heraus starten, sondern per Doppelklick: Sonst landen Einstellungen und Modelle im umgeleiteten Paketordner.

## Mac-Version (experimentell)

Die Mac-Version baut GitHub Actions mit `.github/workflows/tonfeder-mac.yml`, für Apple Silicon (`macos-26`) und Intel (`macos-latest`, cross-kompiliert).

- **Release veröffentlicht:** Der Workflow baut und hängt `Tonfeder_<version>_aarch64.dmg` und `Tonfeder_<version>_x64.dmg` an das Release.
- **Push auf einen `tonfeder/*`-Branch** oder **manuell** (Actions → Tonfeder macOS → Run workflow, nur wenn die Datei auf `main` liegt): baut nur, die DMGs liegen 30 Tage als Artefakt beim Lauf. So lässt sich der Mac-Build vor einem Release prüfen.

Es gibt kein Apple-Entwicklerzertifikat. Die App ist nur ad-hoc signiert (`signingIdentity: "-"`), macOS verlangt deshalb beim ersten Start „Dennoch öffnen“ unter Datenschutz & Sicherheit. Weil sich die Ad-hoc-Signatur mit jedem Build ändert, kann macOS die Bedienungshilfen-Freigabe nach einem Update erneut abfragen.

Die Handy-Workflows (`main-build.yml`, `release.yml` usw.) erwarten Handys Signatur-Secrets und schlagen hier fehl. Für Tonfeder zählt nur `tonfeder-mac.yml`.

## Sicherheitsprüfung (23.10.2026, Stand `5a9451e`)

| Prüfung | Ergebnis |
|---|---|
| Windows Defender auf `C:\tf\release` | keine Bedrohungen |
| Telemetrie und Tracking im Code | keine gefunden |
| Netzverbindungen | Modell-Downloads (`blob.handy.computer`, Hugging Face), Update-Prüfung bei cjpais/Handy, optionale KI-Dienste (standardmäßig aus) |
| Tastatur (`handy-keys` 0.3.4 und `src-tauri/src/shortcut/handy_keys.rs`) | unter Windows keine Netz-, Datei- oder Prozesszugriffe; es werden nur registrierte Kürzel gemeldet und protokolliert |
| `bun audit` | 53 Meldungen, alle in Build-Werkzeugen (Vite, Rollup, ESLint), nicht in der App |
| `cargo audit` | 20 Lücken: `zip` nur über den Updater; `rustls`, `h2`, `webpki` im HTTPS-Stack (per `cargo update` behebbar); `tar`, `rkyv` nicht im Windows-Build; `quick-xml` nur beim Bauen |

**Nach Schritt 3a (25.10.2026):** `bun audit` 0 Meldungen (nach `bun audit fix`). `cargo audit` 3 Lücken: `quick-xml` (2, nur beim Bauen über `tauri-utils`) und `rkyv` (nicht im Windows-Build). Updater samt `zip`, `tar` und `minisign-verify` entfernt; `rustls` 0.23.46, `rustls-webpki` 0.103.16, `h2` 0.4.20, `webpki-roots` 1.0.10.

## Änderungen gegenüber Handy

**3a · Name, Logo, Sicherheit** (Branch `tonfeder/3a-name-logo-sicherheit`)
- `tauri.conf.json`: Produktname Tonfeder, Programmdatei `tonfeder.exe`, Kennung `de.tonfeder.app` (eigener Datenordner, getrennt von Handy). Updater-Konfiguration und der Signier-Befehl des Handy-Autors entfernt.
- Updater-Plugin ausgebaut (`Cargo.toml`, `lib.rs`, Capabilities); `update_checks_forced_disabled()` liefert immer `true`.
- App-Symbole aus `src-tauri/icons/tonfeder-1024.png`, Tray-Bilder in `src-tauri/resources/`, beides erzeugt mit `scripts/tonfeder-symbole.ps1` und `bun tauri icon`.
- Farben in `src/styles/theme.css` auf Lime umgestellt; Text-Logo und Seitenleisten-Symbol ersetzt (Komponentennamen bleiben gleich, damit Upstream-Updates weniger kollidieren).
- „Handy“ in allen Übersetzungen durch „Tonfeder“ ersetzt; die Über-Seite nennt Handy als Grundlage (Knopf „Handy unterstützen“), Quellcode-Link auf dieses Repo.
- Interne Bezeichner (Fensterklassen, Portable-Marker, `HANDY_*`-Variablen) bleiben unverändert.

## Geplante Änderungen

1. ~~Name und Logo Tonfeder, Updater ausbauen, HTTPS-Bibliotheken aktualisieren~~ (3a)
2. Deutsch als Standard, Modellwahl für die RTX 2060 (Whisper Turbo oder Parakeet)
3. Rechte Strg-Taste allein als Auslöser
4. Wörterbuch nach Klang mit Kölner Phonetik (statt nur Soundex)
5. Sprachbefehle: „neuer Absatz“, „Doppelpunkt“, „Spiegelstrich“
6. Aufnahme-Anzeige im Tonfeder-Stil
7. Optional: Nachbearbeitung mit lokalem Sprachmodell (Ollama)
