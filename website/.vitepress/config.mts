import { defineConfig } from "vitepress";
import { readFileSync } from "node:fs";

const release = readFileSync(
  new URL("../../Cargo.toml", import.meta.url),
  "utf8",
).match(/^version = "([^"]+)"/m)![1];
const origin = "https://fcenedes.github.io";
const base = "/mcp-gmap-rs/";
const titles = {
  en: [
    "Start here",
    "Run it your way",
    "Go deeper",
    "Reference",
    "Quick start",
    "Google Cloud setup",
    "Authentication",
    "Configuration",
    "Command line",
    "MCP transports",
    "Docker & Kubernetes",
    "Production & TLS",
    "Recipes",
    "Troubleshooting",
    "Agent skill",
    "Development & releases",
    "All 23 tools",
    "All configuration settings",
  ],
  fr: [
    "Bien démarrer",
    "Exécuter et déployer",
    "Aller plus loin",
    "Référence",
    "Démarrage rapide",
    "Configurer Google Cloud",
    "Authentification",
    "Configuration",
    "Ligne de commande",
    "Transports MCP",
    "Docker et Kubernetes",
    "Production et TLS",
    "Recettes",
    "Dépannage",
    "Skill pour agents",
    "Développement et versions",
    "Les 23 outils",
    "Tous les paramètres",
  ],
  it: [
    "Per iniziare",
    "Eseguire e distribuire",
    "Approfondimenti",
    "Riferimento",
    "Avvio rapido",
    "Configurare Google Cloud",
    "Autenticazione",
    "Configurazione",
    "Riga di comando",
    "Trasporti MCP",
    "Docker e Kubernetes",
    "Produzione e TLS",
    "Ricette",
    "Risoluzione dei problemi",
    "Skill per agenti",
    "Sviluppo e release",
    "Tutti i 23 strumenti",
    "Tutte le impostazioni",
  ],
};

function locale(lang: keyof typeof titles) {
  const t = titles[lang];
  const prefix = lang === "en" ? "" : `/${lang}`;
  const item = (index: number, slug: string) => ({
    text: t[index],
    link: `${prefix}/guide/${slug}`,
  });
  return {
    label: { en: "English", fr: "Français", it: "Italiano" }[lang],
    lang,
    description: {
      en: "The complete manual for the standalone Rust Google Maps CLI and MCP server. Setup, OAuth, 23 tools, TLS and Kubernetes.",
      fr: "Le manuel complet du CLI et serveur MCP Google Maps autonome en Rust. Configuration, OAuth, 23 outils, TLS et Kubernetes.",
      it: "Il manuale completo del CLI e server MCP Google Maps autonomo in Rust. Configurazione, OAuth, 23 strumenti, TLS e Kubernetes.",
    }[lang],
    themeConfig: {
      nav: [
        {
          text: { en: "Manual", fr: "Manuel", it: "Manuale" }[lang],
          link: `${prefix}/guide/getting-started`,
          activeMatch: `${prefix}/guide/`,
        },
        {
          text: { en: "Tools", fr: "Outils", it: "Strumenti" }[lang],
          link: `${prefix}/reference/tools`,
          activeMatch: `${prefix}/reference/`,
        },
        {
          text: `v${release}`,
          link: `https://github.com/fcenedes/mcp-gmap-rs/releases/tag/v${release}`,
        },
      ],
      sidebar: [
        {
          text: t[0],
          items: [
            item(4, "getting-started"),
            item(5, "google-cloud"),
            item(6, "authentication"),
            item(7, "configuration"),
          ],
        },
        {
          text: t[1],
          items: [
            item(8, "cli"),
            item(9, "mcp"),
            item(10, "deployment"),
            item(11, "production"),
          ],
        },
        {
          text: t[2],
          items: [
            item(12, "recipes"),
            item(13, "troubleshooting"),
            item(14, "agent-skill"),
            item(15, "development"),
          ],
        },
        {
          text: t[3],
          items: [
            { text: t[16], link: `${prefix}/reference/tools` },
            { text: t[17], link: `${prefix}/reference/configuration` },
          ],
        },
      ],
      outline: {
        level: 2 as const,
        label: {
          en: "On this page",
          fr: "Sur cette page",
          it: "In questa pagina",
        }[lang],
      },
      docFooter: {
        prev: { en: "Previous", fr: "Précédent", it: "Precedente" }[lang],
        next: { en: "Next", fr: "Suivant", it: "Successivo" }[lang],
      },
      returnToTopLabel: {
        en: "Back to top",
        fr: "Retour en haut",
        it: "Torna in alto",
      }[lang],
      sidebarMenuLabel: { en: "Contents", fr: "Sommaire", it: "Indice" }[lang],
      lightModeSwitchTitle: {
        en: "Switch to light theme",
        fr: "Passer au thème clair",
        it: "Passa al tema chiaro",
      }[lang],
      darkModeSwitchTitle: {
        en: "Switch to dark theme",
        fr: "Passer au thème sombre",
        it: "Passa al tema scuro",
      }[lang],
      darkModeSwitchLabel: { en: "Appearance", fr: "Apparence", it: "Aspetto" }[
        lang
      ],
      skipToContentLabel: {
        en: "Skip to content",
        fr: "Aller au contenu",
        it: "Vai al contenuto",
      }[lang],
      langMenuLabel: {
        en: "Change language",
        fr: "Changer de langue",
        it: "Cambia lingua",
      }[lang],
      editLink: {
        pattern:
          "https://github.com/fcenedes/mcp-gmap-rs/edit/main/website/:path",
        text: {
          en: "Improve this page",
          fr: "Améliorer cette page",
          it: "Migliora questa pagina",
        }[lang],
      },
    },
  };
}

export default defineConfig({
  title: "mcp-gmap-rs",
  titleTemplate: ":title · mcp-gmap-rs",
  base,
  cleanUrls: true,
  srcExclude: ["README.md", "node_modules/**"],
  appearance: { initialValue: "dark" },
  lastUpdated: false,
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: `${base}mark.svg` }],
    ["meta", { name: "theme-color", content: "#151a18" }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:site_name", content: "mcp-gmap-rs — The manual" }],
    ["meta", { property: "og:image", content: `${origin}${base}social.png` }],
  ],
  transformHead({ pageData }) {
    if (pageData.relativePath === "404.md")
      return [["meta", { name: "robots", content: "noindex" }]];
    const path = pageData.relativePath
      .replace(/index\.md$/, "")
      .replace(/\.md$/, "");
    const url = `${origin}${base}${path}`;
    const localPath = path.replace(/^(fr|it)\//, "");
    return [
      ["link", { rel: "canonical", href: url }],
      ...(["en", "fr", "it"] as const).map(
        (lang) =>
          [
            "link",
            {
              rel: "alternate",
              hreflang: lang,
              href: `${origin}${base}${lang === "en" ? "" : lang + "/"}${localPath}`,
            },
          ] as [string, Record<string, string>],
      ),
      ["meta", { property: "og:url", content: url }],
      [
        "meta",
        { property: "og:title", content: `${pageData.title} · mcp-gmap-rs` },
      ],
      ["meta", { property: "og:description", content: pageData.description }],
    ];
  },
  sitemap: { hostname: `${origin}${base}` },
  locales: { root: locale("en"), fr: locale("fr"), it: locale("it") },
  themeConfig: {
    logo: "/mark.svg",
    siteTitle: "mcp-gmap-rs",
    socialLinks: [
      { icon: "github", link: "https://github.com/fcenedes/mcp-gmap-rs" },
    ],
    search: {
      provider: "local",
      options: {
        locales: {
          fr: {
            translations: {
              button: {
                buttonText: "Rechercher",
                buttonAriaLabel: "Rechercher dans le manuel",
              },
              modal: {
                displayDetails: "Afficher les détails",
                resetButtonTitle: "Effacer",
                backButtonTitle: "Fermer",
                noResultsText: "Aucun résultat pour",
                footer: {
                  selectText: "sélectionner",
                  selectKeyAriaLabel: "entrée",
                  navigateText: "parcourir",
                  navigateUpKeyAriaLabel: "flèche haut",
                  navigateDownKeyAriaLabel: "flèche bas",
                  closeText: "fermer",
                  closeKeyAriaLabel: "échap",
                },
              },
            },
          },
          it: {
            translations: {
              button: {
                buttonText: "Cerca",
                buttonAriaLabel: "Cerca nel manuale",
              },
              modal: {
                displayDetails: "Mostra dettagli",
                resetButtonTitle: "Cancella",
                backButtonTitle: "Chiudi",
                noResultsText: "Nessun risultato per",
                footer: {
                  selectText: "seleziona",
                  selectKeyAriaLabel: "invio",
                  navigateText: "naviga",
                  navigateUpKeyAriaLabel: "freccia su",
                  navigateDownKeyAriaLabel: "freccia giù",
                  closeText: "chiudi",
                  closeKeyAriaLabel: "esc",
                },
              },
            },
          },
        },
      },
    },
  },
});
