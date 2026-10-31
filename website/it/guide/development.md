# Sviluppo, CI e release

Il prodotto distribuito è un eseguibile Rust indipendente. Il sito e gli strumenti di generazione della documentazione sono strumenti di sviluppo separati; non sono richiesti per eseguire CLI o MCP.

## Verifica locale

Da un checkout con Rust 1.88 o successivo:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

I test ordinari usano fixture locali e non richiedono Google. Coprono i 42 strumenti, sottoprocessi CLI, stdio e HTTP. I test live sono esplicitamente ignorati per impostazione predefinita: richiedono credenziali, API abilitate e fatturazione e possono generare costi.

```sh
cargo test --locked --test live live_maps_ -- --ignored --test-threads=1
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

La seconda suite rimuove la chiave dai processi figli per verificare realmente OAuth. Non eseguire queste suite quando bastano i test offline. Dopo le verifiche, `cargo clean` recupera lo spazio generato; la build successiva ricompilerà.

## Mantenere il riferimento strumenti

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

Gli schemi forniscono tipi, obbligatorietà e limiti; le note JSON revisionate descrivono valori predefiniti e comportamento. La verifica rimuove le credenziali, disabilita dotenv e controlla tutti gli esempi senza chiamare Google. Aggiorna insieme manuale, traduzioni e note quando cambia l'interfaccia.

## Pubblicare

La CI verifica Linux/macOS, Clippy, MSRV 1.88, rustdoc stable/nightly, pacchetto Cargo, container e manifest Kubernetes. Il workflow Release ricontrolla il tag esatto, costruisce tre binari nativi e pubblica archivi con checksum solo dopo il successo completo. Il workflow manuale **Publish crate** pubblica quel tag su crates.io; questo caricamento avvia automaticamente docs.rs.

Aggiorna versione Cargo, lockfile, changelog e guide prima di creare un nuovo tag annotato. I tag pubblicati non vengono spostati e le versioni crates.io non si sovrascrivono. Mantieni `CARGO_REGISTRY_TOKEN` nel Secret GitHub dedicato. Consulta la [procedura di release](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md) e la [documentazione Rust](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/) per i dettagli.
