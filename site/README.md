<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# site/ — public landing page

Static, dependency-free landing page for RAHN, deployed to GitHub Pages by
[`.github/workflows/pages.yml`](../.github/workflows/pages.yml) (source: this folder,
deployed on every push that touches `site/**`).

- `index.html` — the page: semantic HTML, Open Graph/Twitter metadata, JSON-LD
  (`SoftwareSourceCode` + `FAQPage`), light mode only
- `assets/css/style.css` — single stylesheet, custom properties, responsive breakpoints
  at 960px/720px, honors `prefers-reduced-motion`
- `assets/js/main.js` — optional progressive enhancement (mobile nav, FAQ accordion,
  header shadow); the page is fully functional without it
- `assets/img/` — brand assets (derived from the project logo) and favicons
- `robots.txt`, `sitemap.xml`, `404.html` — crawler and error-surface basics

Canonical URL: `https://mhalikhani.github.io/rahn/`. When renaming the repository or
Pages target, update the canonical URL, Open Graph/Twitter image URLs, and `sitemap.xml`
to match. Code in this folder is licensed Apache-2.0 like the rest of the source.
