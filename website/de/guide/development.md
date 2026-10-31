# Entwicklung und Releases

Die Laufzeit ist ein eigenständiges Rust-Projekt. CLI, MCP stdio und Streamable HTTP verwenden dieselben Werkzeugimplementierungen. Die Dokumentationswebsite ist getrenntes Autoren-/Build-Werkzeug; Endnutzer benötigen nur das kompilierte Programm.

## Bauen und prüfen

Klonen Sie das [Repository](https://github.com/fcenedes/mcp-gmap-rs) und verwenden Sie die Lockdatei:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

Rust 1.88 ist die erklärte Mindestversion; Release-Binärdateien verwenden Rust 1.94.1. Standardtests nutzen lokale Google-/Token-Fixtures und benötigen weder Zugangsdaten noch Google-Netzwerkzugriff. CI prüft außerdem Linux/macOS, Mindest-Rust, stabiles/nightly rustdoc, Quellpaketierung, Docker-/MCP-/TLS-Smoke-Szenarien und Kubernetes-Manifeste.

Nach lokaler Prüfung gibt `cargo clean` erzeugte Build-Ausgaben frei; spätere Builds kompilieren sie erneut.

## Referenz vollständig halten

Die maßgebliche Werkzeuganleitung kombiniert tatsächliche Programmschemas mit geprüften Standardwerten, Ergebnisbeschreibungen und Verhaltenshinweisen:

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

Die Prüfung entfernt Zugangsdaten, deaktiviert Dotenv-Laden und validiert sämtliche Beispiele an der Fehlender-Schlüssel-Grenze ohne Google-Kontakt. Aktualisieren Sie die geprüften JSON-Hinweise, wenn sich Parameter oder Verhalten ändern.

Live-Tests sind freiwillig und möglicherweise kostenpflichtig. Mit ausdrücklich eingerichtetem Testschlüssel verwenden Sie `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1`. Die sechs OAuth-Tests nutzen den Filter `live_oauth_` und entfernen den Schlüssel aus Kindprozessen. Checken Sie niemals Zugangsdaten ein und aktivieren Sie Live-Tests nicht in gewöhnlicher CI.

## Version veröffentlichen

Aktualisieren Sie Cargo-Version, Lockdatei, Änderungsprotokoll, Installationsbeispiele und Release-Notizen gemeinsam. Pushen Sie die Änderungen, warten Sie auf CI und erstellen Sie einen passenden annotierten Tag `vX.Y.Z`. Der Release-Workflow prüft genau diesen Tag, baut drei native Ziele, verifiziert Archive und Prüfsummen und veröffentlicht den vorbereiteten Entwurf.

Der separate Workflow **Publish crate** benötigt dieses veröffentlichte GitHub-Release. Starten Sie ihn für denselben Tag. Speichern Sie sein eingeschränktes `CARGO_REGISTRY_TOKEN` in der GitHub-Umgebung `crates-io`, niemals im Repository. docs.rs baut nach crates.io-Veröffentlichung automatisch; ein GitHub-Release allein aktualisiert die gehostete Rust-Dokumentation nicht. Prüfen Sie nach dem Upload den tatsächlichen gehosteten Build.

Siehe [Release-Verfahren](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md), [Validierungsprotokoll](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md) und [öffentlichen Rust-Einstiegspunkt](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html).
