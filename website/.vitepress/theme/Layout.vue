<script setup lang="ts">
import { computed } from "vue";
import DefaultTheme from "vitepress/theme";
import { useData, withBase } from "vitepress";
import Landing from "./components/Landing.vue";
import { version } from "../../data/catalog.json";
import locales from "../../data/locales.json";
const { frontmatter, localeIndex } = useData();
const key = computed(() =>
  localeIndex.value === "root"
    ? "en"
    : (localeIndex.value as keyof typeof locales),
);
const strings = computed(() => locales[key.value]);
const home = computed(() =>
  withBase(key.value === "en" ? "/" : `/${key.value}/`),
);
</script>

<template>
  <DefaultTheme.Layout>
    <template #home-hero-before
      ><Landing v-if="frontmatter.layout === 'home'"
    /></template>
    <template #doc-before
      ><div class="manual-eyebrow">
        {{ strings.eyebrow }} / v{{ version }}
      </div></template
    >
    <template #sidebar-nav-before
      ><a class="sidebar-intro" :href="home"
        >{{ strings.sidebarIntro }} <span>↗</span></a
      ></template
    >
  </DefaultTheme.Layout>
</template>
