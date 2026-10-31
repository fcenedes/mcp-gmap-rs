import { defineConfig } from "vitepress";
import { readFileSync } from "node:fs";
import postcssRTLCSS from "postcss-rtlcss";

const release = readFileSync(
  new URL("../../Cargo.toml", import.meta.url),
  "utf8",
).match(/^version = "([^"]+)"/m)![1];
const origin = "https://fcenedes.github.io";
const base = "/mcp-gmap-rs/";
const languages: string[] = JSON.parse(
  readFileSync(new URL("../i18n/locales.json", import.meta.url), "utf8"),
);
const strings = Object.fromEntries(
  languages.map((key) => [
    key,
    JSON.parse(
      readFileSync(new URL(`../i18n/${key}.json`, import.meta.url), "utf8"),
    ),
  ]),
);
const prefix = (key: string) => (key === "en" ? "" : `/${key}`);

function locale(key: string) {
  const data = strings[key];
  const t = data.titles;
  const item = (index: number, slug: string) => ({
    text: t[index],
    link: `${prefix(key)}/guide/${slug}`,
  });
  return {
    label: data.label,
    lang: data.lang,
    dir: data.dir,
    description: data.description,
    themeConfig: {
      nav: [
        {
          text: data.nav.manual,
          link: `${prefix(key)}/guide/getting-started`,
          activeMatch: `${prefix(key)}/guide/`,
        },
        {
          text: data.nav.tools,
          link: `${prefix(key)}/reference/tools`,
          activeMatch: `${prefix(key)}/reference/`,
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
            { text: t[16], link: `${prefix(key)}/reference/tools` },
            { text: t[17], link: `${prefix(key)}/reference/configuration` },
          ],
        },
      ],
      outline: { level: 2 as const, label: data.ui.outline },
      docFooter: { prev: data.ui.previous, next: data.ui.next },
      returnToTopLabel: data.ui.top,
      sidebarMenuLabel: data.ui.contents,
      lightModeSwitchTitle: data.ui.light,
      darkModeSwitchTitle: data.ui.dark,
      darkModeSwitchLabel: data.ui.appearance,
      skipToContentLabel: data.ui.skip,
      langMenuLabel: data.ui.language,
      editLink: {
        pattern:
          "https://github.com/fcenedes/mcp-gmap-rs/edit/main/website/:path",
        text: data.ui.edit,
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
  // VitePress 1.6 uses physical CSS properties. Mirror its styles for RTL locales.
  // Revisit this plugin when upgrading to a theme that uses logical properties.
  vite: {
    css: { postcss: { plugins: [postcssRTLCSS({ safeBothPrefix: true })] } },
  },
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: `${base}mark.svg` }],
    ["meta", { name: "theme-color", content: "#151a18" }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:site_name", content: "mcp-gmap-rs — The manual" }],
    ["meta", { property: "og:image", content: `${origin}${base}social.png` }],
  ],
  transformPageData(pageData) {
    pageData.title = pageData.title.replace(/\s*\{#[^}]+\}\s*$/, "");
  },
  transformHead({ pageData }) {
    if (pageData.relativePath === "404.md")
      return [["meta", { name: "robots", content: "noindex" }]];
    const path = pageData.relativePath
      .replace(/index\.md$/, "")
      .replace(/\.md$/, "");
    const first = path.split("/")[0];
    const localPath =
      languages.includes(first) && first !== "en"
        ? path.slice(first.length + 1)
        : path;
    const url = `${origin}${base}${path}`;
    return [
      ["link", { rel: "canonical", href: url }],
      ...languages.map(
        (key) =>
          [
            "link",
            {
              rel: "alternate",
              hreflang: strings[key].lang,
              href: `${origin}${base}${key === "en" ? "" : key + "/"}${localPath}`,
            },
          ] as [string, Record<string, string>],
      ),
      ["meta", { property: "og:url", content: url }],
      [
        "meta",
        { property: "og:title", content: `${pageData.title} · mcp-gmap-rs` },
      ],
      [
        "meta",
        {
          property: "og:description",
          content:
            pageData.description ||
            strings[languages.includes(first) ? first : "en"].description,
        },
      ],
    ];
  },
  sitemap: { hostname: `${origin}${base}` },
  locales: Object.fromEntries(
    languages.map((key) => [key === "en" ? "root" : key, locale(key)]),
  ),
  themeConfig: {
    logo: "/mark.svg",
    siteTitle: "mcp-gmap-rs",
    socialLinks: [
      { icon: "github", link: "https://github.com/fcenedes/mcp-gmap-rs" },
    ],
    search: {
      provider: "local",
      options: {
        locales: Object.fromEntries(
          languages.map((key) => [
            key === "en" ? "root" : key,
            { translations: strings[key].search },
          ]),
        ),
        miniSearch: {
          options: {
            // Keep indexing and browser queries identical, including languages without spaces.
            tokenize(text: string) {
              if (typeof Intl.Segmenter === "function") {
                const segmenter = new Intl.Segmenter("en", {
                  granularity: "word",
                });
                return Array.from(segmenter.segment(text))
                  .filter((part) => part.isWordLike)
                  .map((part) => part.segment);
              }
              return (
                text.match(
                  /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]|[\p{L}\p{N}_]+/gu,
                ) || []
              );
            },
            processTerm: (term: string) => term.normalize("NFKC").toLowerCase(),
          },
        },
      },
    },
  },
});
