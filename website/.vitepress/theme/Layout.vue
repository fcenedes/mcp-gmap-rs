<script setup lang="ts">
import DefaultTheme from "vitepress/theme";
import { useData } from "vitepress";
import Landing from "./components/Landing.vue";
import { version } from "../../data/catalog.json";
const { frontmatter, lang } = useData();
const labels = { en: "THE MANUAL", fr: "LE MANUEL", it: "IL MANUALE" };
</script>

<template>
  <DefaultTheme.Layout>
    <template #home-hero-before
      ><Landing v-if="frontmatter.layout === 'home'"
    /></template>
    <template #doc-before
      ><div class="manual-eyebrow">
        {{ labels[lang as keyof typeof labels] || labels.en }} / v{{ version }}
      </div></template
    >
    <template #sidebar-nav-before
      ><a
        class="sidebar-intro"
        :href="lang === 'en' ? '/mcp-gmap-rs/' : `/mcp-gmap-rs/${lang}/`"
        >{{
          lang === "fr"
            ? "Votre atlas de possibilités."
            : lang === "it"
              ? "Il tuo atlante di possibilità."
              : "Your atlas of possibilities."
        }}
        <span>↗</span></a
      ></template
    >
  </DefaultTheme.Layout>
</template>
