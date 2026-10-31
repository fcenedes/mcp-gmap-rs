# Développement et publication

Le programme d’exécution reste un projet Rust autonome. Les ressources de documentation et le site n’ajoutent aucune dépendance d’exécution à la CLI ou au serveur MCP. Le code est sous licence MIT et conserve la licence de l’amont dans `licenses/`.

## Vérifier une modification

Depuis le dépôt :

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo test --doc --all-features --locked
```

Les tests ordinaires utilisent des fixtures locales et n’appellent pas Google. Les tests d’acceptation réels sont ignorés par défaut, requièrent des identifiants et peuvent être facturés. Exécutez-les seulement si la modification et votre autorisation le justifient.

La référence des outils est générée à partir du catalogue de l’exécutable et de notes relues :

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

Le contrôle retire les identifiants et désactive dotenv. Les 23 exemples doivent atteindre l’erreur attendue de clé absente après validation de leurs arguments. La CI refuse les outils ou paramètres non documentés et les sorties générées périmées.

## Livrer une version

La CI vérifie Linux/macOS, Clippy, Rust 1.88, rustdoc stable/nightly, le paquet source, le conteneur et les manifestes Kubernetes. Mettez à jour version Cargo, lockfile, changelog, guides et notes de version, puis créez un tag annoté correspondant après succès des vérifications. Le workflow Release contrôle ce tag, construit les trois binaires, vérifie leurs empreintes puis publie GitHub.

Le workflow manuel **Publish crate** publie séparément le tag déjà livré sur GitHub. Son secret `CARGO_REGISTRY_TOKEN` appartient à l’environnement GitHub `crates-io` ; il ne doit jamais entrer dans le dépôt. La publication crates.io déclenche ensuite docs.rs. Une version publiée et son tag ne sont pas remplacés : une correction de documentation nécessite une nouvelle version du crate pour docs.rs.

Après les vérifications locales, `cargo clean` récupère l’espace disque ; une compilation ultérieure le recréera. Consultez le [guide de livraison source](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md) pour le détail des workflows.
