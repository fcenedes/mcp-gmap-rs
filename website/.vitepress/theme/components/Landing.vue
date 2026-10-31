<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useData, withBase } from "vitepress";
import RouteMap from "./RouteMap.vue";
import catalog from "../../../data/catalog.json";

const { lang } = useData();
const language = computed(() =>
  lang.value === "fr" ? "fr" : lang.value === "it" ? "it" : "en",
);
const prefix = computed(() =>
  language.value === "en" ? "" : `/${language.value}`,
);
const href = (path: string) => withBase(`${prefix.value}${path}`);
const active = ref("all");
const query = ref("");
const copied = ref(false);
const copyFailed = ref(false);
let timer: ReturnType<typeof setTimeout>;
let observer: IntersectionObserver | undefined;
const text = {
  en: {
    edition: "THE FIELD MANUAL",
    tagline: "The world. Within reach.",
    intro:
      "Google Maps for your terminal and your agents. One Rust binary. Every coordinate, a possibility.",
    start: "Open the manual",
    browse: "Explore the tools",
    copy: "Copy install command",
    copied: "Copied",
    failed: "Select the command to copy it",
    map: "ILLUSTRATIVE ROUTE · NO LIVE DATA",
    scroll: "A LITTLE FURTHER",
    section: "01 / ONE BINARY, THREE WAYS IN",
    ways: "Fits the way\nyou work.",
    waysIntro:
      "From a quick lookup to a production agent. The same tools, wherever you run them.",
    cli: "Your terminal.",
    cliText:
      "Find a place, validate an address, plan a route. Structured JSON. Scripts welcome.",
    stdio: "Your agent.",
    stdioText:
      "Launch an MCP stdio server. Connect your host. Give your agent a sense of place.",
    http: "Your infrastructure.",
    httpText:
      "Streamable HTTP, native TLS and mTLS. Ready for containers and Kubernetes.",
    catalog: "02 / THE TOOLKIT",
    toolsTitle: "A world of answers.",
    toolsIntro:
      "23 tools. From street-level details to the conditions overhead. Every parameter, default and example is documented.",
    search: "Filter tools",
    count: "tools",
    all: "All tools",
    places: "Places & addresses",
    routes: "Routes & roads",
    environment: "Local conditions",
    geography: "Geography & maps",
    empty: "No matching tools. Try another search.",
    reference: "Read the full reference",
    journey: "03 / FROM FIRST CALL TO PRODUCTION",
    journeyTitle: "Start small.\nGo anywhere.",
    guide: "YOUR NEXT STEP",
    quick: "Up and running",
    quickText: "Install the binary and make your first call.",
    auth: "Connect to Google",
    authText: "API keys, service accounts and OAuth, explained.",
    deploy: "Take it to production",
    deployText: "Docker, Kubernetes, TLS and the controls that matter.",
    final: "Bring a sense of place\nto your next agent.",
    finalAction: "Start building",
    footer: "Built in Rust. Documented in three languages.",
    source: "Source",
    release: "Releases",
    rustdoc: "Rust API",
    top: "Back to top",
    locale: "EN / FR / IT",
    toolsLink: "Tool reference",
    manualLink: "Manual",
  },
  fr: {
    edition: "LE MANUEL DE TERRAIN",
    tagline: "Le monde. À portée de main.",
    intro:
      "Google Maps pour votre terminal et vos agents. Un binaire Rust. Chaque coordonnée ouvre des possibilités.",
    start: "Ouvrir le manuel",
    browse: "Explorer les outils",
    copy: "Copier la commande",
    copied: "Copié",
    failed: "Sélectionnez la commande pour la copier",
    map: "ITINÉRAIRE ILLUSTRATIF · AUCUNE DONNÉE EN DIRECT",
    scroll: "UN PEU PLUS LOIN",
    section: "01 / UN BINAIRE, TROIS FAÇONS DE L’UTILISER",
    ways: "À votre façon\nde travailler.",
    waysIntro:
      "D’une recherche rapide à un agent en production. Les mêmes outils, partout où vous les exécutez.",
    cli: "Votre terminal.",
    cliText:
      "Trouvez un lieu, validez une adresse, calculez un trajet. Du JSON structuré pour vos scripts.",
    stdio: "Votre agent.",
    stdioText:
      "Lancez un serveur MCP stdio. Connectez votre hôte. Donnez à votre agent le sens du lieu.",
    http: "Votre infrastructure.",
    httpText:
      "HTTP streamable, TLS et mTLS natifs. Pour les conteneurs et Kubernetes.",
    catalog: "02 / LES OUTILS",
    toolsTitle: "Un monde de réponses.",
    toolsIntro:
      "23 outils. Des détails d’une rue aux conditions locales. Chaque paramètre, valeur par défaut et exemple est documenté.",
    search: "Filtrer les outils",
    count: "outils",
    all: "Tous les outils",
    places: "Lieux et adresses",
    routes: "Itinéraires et routes",
    environment: "Conditions locales",
    geography: "Géographie et cartes",
    empty: "Aucun outil trouvé. Essayez une autre recherche.",
    reference: "Consulter toute la référence",
    journey: "03 / DU PREMIER APPEL À LA PRODUCTION",
    journeyTitle: "Commencez petit.\nAllez plus loin.",
    guide: "LA PROCHAINE ÉTAPE",
    quick: "Démarrer rapidement",
    quickText: "Installez le binaire et lancez votre premier appel.",
    auth: "Se connecter à Google",
    authText: "Clés API, comptes de service et OAuth, expliqués.",
    deploy: "Passer en production",
    deployText: "Docker, Kubernetes, TLS et les contrôles utiles.",
    final: "Donnez le sens du lieu\nà votre prochain agent.",
    finalAction: "Commencer",
    footer: "Écrit en Rust. Documenté en trois langues.",
    source: "Code source",
    release: "Versions",
    rustdoc: "API Rust",
    top: "Retour en haut",
    locale: "EN / FR / IT",
    toolsLink: "Référence des outils",
    manualLink: "Manuel",
  },
  it: {
    edition: "IL MANUALE SUL CAMPO",
    tagline: "Il mondo. A portata di mano.",
    intro:
      "Google Maps per il tuo terminale e i tuoi agenti. Un binario Rust. Ogni coordinata, una possibilità.",
    start: "Apri il manuale",
    browse: "Esplora gli strumenti",
    copy: "Copia il comando",
    copied: "Copiato",
    failed: "Seleziona il comando per copiarlo",
    map: "PERCORSO ILLUSTRATIVO · NESSUN DATO IN TEMPO REALE",
    scroll: "UN PO’ PIÙ AVANTI",
    section: "01 / UN BINARIO, TRE MODI DI USARLO",
    ways: "Segue il tuo modo\ndi lavorare.",
    waysIntro:
      "Da una ricerca veloce a un agente in produzione. Gli stessi strumenti, ovunque li esegui.",
    cli: "Il tuo terminale.",
    cliText:
      "Trova un luogo, convalida un indirizzo, pianifica un percorso. JSON strutturato, pronto per gli script.",
    stdio: "Il tuo agente.",
    stdioText:
      "Avvia un server MCP stdio. Collega il tuo host. Dai al tuo agente il senso del luogo.",
    http: "La tua infrastruttura.",
    httpText: "HTTP streamable, TLS e mTLS nativi. Per container e Kubernetes.",
    catalog: "02 / GLI STRUMENTI",
    toolsTitle: "Un mondo di risposte.",
    toolsIntro:
      "23 strumenti. Dai dettagli di una strada alle condizioni locali. Ogni parametro, valore predefinito ed esempio è documentato.",
    search: "Filtra gli strumenti",
    count: "strumenti",
    all: "Tutti gli strumenti",
    places: "Luoghi e indirizzi",
    routes: "Percorsi e strade",
    environment: "Condizioni locali",
    geography: "Geografia e mappe",
    empty: "Nessuno strumento trovato. Prova un’altra ricerca.",
    reference: "Leggi il riferimento completo",
    journey: "03 / DALLA PRIMA CHIAMATA ALLA PRODUZIONE",
    journeyTitle: "Inizia in piccolo.\nVai ovunque.",
    guide: "IL PROSSIMO PASSO",
    quick: "Subito operativo",
    quickText: "Installa il binario e fai la prima chiamata.",
    auth: "Collegati a Google",
    authText: "Chiavi API, account di servizio e OAuth, spiegati.",
    deploy: "Passa alla produzione",
    deployText: "Docker, Kubernetes, TLS e i controlli essenziali.",
    final: "Dai il senso del luogo\nal tuo prossimo agente.",
    finalAction: "Inizia a creare",
    footer: "Scritto in Rust. Documentato in tre lingue.",
    source: "Sorgente",
    release: "Release",
    rustdoc: "API Rust",
    top: "Torna in alto",
    locale: "EN / FR / IT",
    toolsLink: "Riferimento strumenti",
    manualLink: "Manuale",
  },
};
const t = computed(() => text[language.value]);
const categories = [
  "all",
  "places",
  "routes",
  "environment",
  "geography",
] as const;
const normalized = (value: string) =>
  value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase();
const tools = computed(() =>
  catalog.tools.filter(
    (tool) =>
      (active.value === "all" || tool.category === active.value) &&
      normalized(`${tool.name} ${tool.summary[language.value]}`).includes(
        normalized(query.value),
      ),
  ),
);
async function copy() {
  try {
    await navigator.clipboard.writeText("cargo install mcp-gmap-rs --locked");
    copied.value = true;
    copyFailed.value = false;
    clearTimeout(timer);
    timer = setTimeout(() => (copied.value = false), 2200);
  } catch {
    copyFailed.value = true;
  }
}
onMounted(() => {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  observer = new IntersectionObserver(
    (entries) =>
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("is-visible");
          observer?.unobserve(entry.target);
        }
      }),
    { threshold: 0.1 },
  );
  document.querySelectorAll(".reveal").forEach((element) => {
    element.classList.add("will-reveal");
    observer?.observe(element);
  });
});
onUnmounted(() => {
  clearTimeout(timer);
  observer?.disconnect();
});
</script>

<template>
  <main class="atlas-home" id="top">
    <section class="atlas-hero" aria-labelledby="atlas-title">
      <RouteMap />
      <div class="hero-shade"></div>
      <div class="hero-inner">
        <div class="hero-edition">
          <span class="edition-dot"></span>{{ t.edition }}
          <span class="edition-rule"></span> v{{ catalog.version }}
        </div>
        <h1 id="atlas-title">mcp-gmap<span class="hero-rs">/rs</span></h1>
        <p class="hero-tagline">{{ t.tagline }}</p>
        <p class="hero-description">{{ t.intro }}</p>
        <div class="hero-actions">
          <a class="atlas-button" :href="href('/guide/getting-started')"
            >{{ t.start }} <span>↗</span></a
          ><a class="hero-secondary" href="#toolkit"
            >{{ t.browse }} <span>↓</span></a
          >
        </div>
        <div class="install-line">
          <span class="install-prompt">$</span
          ><code>cargo install mcp-gmap-rs --locked</code
          ><button @click="copy" :aria-label="t.copy" :title="t.copy">
            <span v-if="copied">✓</span
            ><svg
              v-else
              width="17"
              height="17"
              viewBox="0 0 20 20"
              fill="none"
              stroke="currentColor"
              stroke-width="1.3"
            >
              <rect x="7" y="7" width="10" height="10" rx="1.5" />
              <path d="M12 4H4v8" />
            </svg></button
          ><span class="copy-feedback" role="status">{{
            copied ? t.copied : copyFailed ? t.failed : ""
          }}</span>
        </div>
      </div>
      <div class="hero-bottom">
        <a href="#ways">{{ t.scroll }} <span>↓</span></a
        ><span>{{ t.map }}</span>
      </div>
    </section>

    <div class="atlas-paper">
      <section
        class="atlas-section ways-section reveal"
        id="ways"
        aria-labelledby="ways-title"
      >
        <p class="section-kicker">{{ t.section }}</p>
        <div class="section-intro">
          <h2 id="ways-title">{{ t.ways }}</h2>
          <p>{{ t.waysIntro }}</p>
        </div>
        <div class="ways-list">
          <a :href="href('/guide/cli')"
            ><span class="way-number">01 / CLI</span>
            <h3>{{ t.cli }}</h3>
            <p>{{ t.cliText }}</p>
            <code>mcp-gmap-rs geocode</code><span class="way-arrow">↗</span></a
          >
          <a :href="href('/guide/mcp')"
            ><span class="way-number">02 / STDIO</span>
            <h3>{{ t.stdio }}</h3>
            <p>{{ t.stdioText }}</p>
            <code>mcp-gmap-rs serve stdio</code
            ><span class="way-arrow">↗</span></a
          >
          <a :href="href('/guide/deployment')"
            ><span class="way-number">03 / HTTP</span>
            <h3>{{ t.http }}</h3>
            <p>{{ t.httpText }}</p>
            <code>mcp-gmap-rs serve http</code
            ><span class="way-arrow">↗</span></a
          >
        </div>
      </section>

      <section
        class="toolkit-section"
        id="toolkit"
        aria-labelledby="tools-title"
      >
        <div class="atlas-section">
          <div class="reveal">
            <p class="section-kicker">{{ t.catalog }}</p>
            <div class="section-intro">
              <h2 id="tools-title">{{ t.toolsTitle }}</h2>
              <p>{{ t.toolsIntro }}</p>
            </div>
          </div>
          <div class="tool-controls">
            <div class="category-tabs" :aria-label="t.browse">
              <button
                v-for="category in categories"
                :key="category"
                :class="{ active: active === category }"
                @click="active = category"
                :aria-pressed="active === category"
              >
                {{ t[category] }}
              </button>
            </div>
            <label class="tool-filter"
              ><svg
                viewBox="0 0 20 20"
                width="18"
                height="18"
                fill="none"
                stroke="currentColor"
                stroke-width="1.4"
              >
                <circle cx="8.5" cy="8.5" r="5.5" />
                <path d="m13 13 4 4" /></svg
              ><input
                v-model="query"
                type="search"
                :placeholder="t.search"
                :aria-label="t.search"
            /></label>
          </div>
          <div class="tool-count" aria-live="polite">
            {{ tools.length.toString().padStart(2, "0") }} / 23 {{ t.count }}
          </div>
          <div class="tool-list">
            <a
              v-for="tool in tools"
              :key="tool.name"
              :href="href(`/reference/tools#${tool.name}`)"
              ><div>
                <h3>{{ tool.name.replace("maps_", "") }}</h3>
                <p>{{ tool.summary[language] }}</p>
              </div>
              <span>↗</span></a
            >
          </div>
          <p v-if="!tools.length" class="no-tools">{{ t.empty }}</p>
          <a class="text-link" :href="href('/reference/tools')"
            >{{ t.reference }} <span>↗</span></a
          >
        </div>
      </section>

      <section
        class="atlas-section journey-section reveal"
        aria-labelledby="journey-title"
      >
        <p class="section-kicker">{{ t.journey }}</p>
        <div class="journey-layout">
          <h2 id="journey-title">{{ t.journeyTitle }}</h2>
          <div class="journey-links">
            <a :href="href('/guide/getting-started')"
              ><span>01</span>
              <div>
                <h3>{{ t.quick }}</h3>
                <p>{{ t.quickText }}</p>
              </div>
              <span>↗</span></a
            ><a :href="href('/guide/authentication')"
              ><span>02</span>
              <div>
                <h3>{{ t.auth }}</h3>
                <p>{{ t.authText }}</p>
              </div>
              <span>↗</span></a
            ><a :href="href('/guide/production')"
              ><span>03</span>
              <div>
                <h3>{{ t.deploy }}</h3>
                <p>{{ t.deployText }}</p>
              </div>
              <span>↗</span></a
            >
          </div>
        </div>
      </section>
    </div>

    <section class="atlas-finale">
      <div class="atlas-section reveal">
        <p class="section-kicker">MCP-GMAP-RS / {{ t.locale }}</p>
        <h2>{{ t.final }}</h2>
        <a class="atlas-button" :href="href('/guide/getting-started')"
          >{{ t.finalAction }} <span>↗</span></a
        >
      </div>
      <svg
        class="finale-orbit"
        viewBox="0 0 600 600"
        fill="none"
        stroke="currentColor"
        aria-hidden="true"
      >
        <circle cx="300" cy="300" r="150" />
        <circle cx="300" cy="300" r="230" />
        <circle cx="300" cy="300" r="299" />
        <path d="M0 300h600M300 0v600" />
        <circle cx="300" cy="150" r="6" fill="currentColor" />
      </svg>
    </section>
    <footer class="atlas-footer">
      <a :href="href('/')" class="footer-brand">mcp-gmap<span>/rs</span></a>
      <p>{{ t.footer }}</p>
      <div>
        <a href="https://github.com/fcenedes/mcp-gmap-rs">{{ t.source }} ↗</a
        ><a href="https://github.com/fcenedes/mcp-gmap-rs/releases"
          >{{ t.release }} ↗</a
        ><a href="https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/"
          >{{ t.rustdoc }} ↗</a
        ><a href="#top" :aria-label="t.top">↑</a>
      </div>
    </footer>
  </main>
</template>
