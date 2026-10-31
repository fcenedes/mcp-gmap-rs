<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useData, withBase } from "vitepress";
import RouteMap from "./RouteMap.vue";
import catalog from "../../../data/catalog.json";
import locales from "../../../data/locales.json";

const { localeIndex } = useData();
const language = computed(() =>
  localeIndex.value === "root"
    ? "en"
    : (localeIndex.value as keyof typeof locales),
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
const t = computed(() => locales[language.value].landing);
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
