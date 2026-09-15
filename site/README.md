<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# site/ — public website

Static website for RAHN, deployed to GitHub Pages at
`https://mhalikhani.github.io/rahn/` by
[`.github/workflows/pages.yml`](../.github/workflows/pages.yml)
(build → validate → deploy on every push touching `site/**` or `site-src/**`).

**Do not edit `site/` by hand** — it is generated output.

- **`../site-src/pages/*.html`** — page sources: JSON front matter (title,
  description, slug, breadcrumb group, schema type) + HTML body.
  The build enforces unique titles and unique meta descriptions.
- **`../site-src/assets/`** — stylesheets, scripts, and brand images.
- **`../site-src/static/`** — copied verbatim (`robots.txt`, `404.html`).
- **`../scripts/build-site.mjs`** — the zero-dependency generator (Node ≥ 18):
  shared layout with full metadata, Open Graph/Twitter tags, canonical URLs,
  BreadcrumbList/TechArticle/FAQPage JSON-LD, and a generated `sitemap.xml`.
- **`../scripts/validate-site.mjs`** — static SEO validation: uniqueness,
  canonical URLs, JSON-LD parsing, OG/Twitter completeness, internal-link and
  GitHub-file-link resolution, sitemap/robots consistency.

When renaming the repository or Pages target, update `BASE` in both scripts.
Code in this folder is licensed Apache-2.0 like the rest of the source.
