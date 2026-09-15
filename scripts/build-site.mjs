// SPDX-License-Identifier: Apache-2.0
// Static site generator for the RAHN Pages site (site/).
// Zero dependencies: Node >= 18. Source lives in site-src/, output in site/.
// Run: node scripts/build-site.mjs   (also wired into .github/workflows/pages.yml)
import { readFileSync, writeFileSync, readdirSync, mkdirSync, rmSync, cpSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const SRC = join(root, "site-src");
const OUT = join(root, "site");
const BASE = "https://mhalikhani.github.io/rahn";
const GH = "https://github.com/MHAlikhani/rahn";

// ---------- page source: front matter (JSON) + HTML body, split on a lone "---" ----------
function loadPages() {
  const dir = join(SRC, "pages");
  const out = [];
  const walk = (d) => {
    for (const f of readdirSync(d, { withFileTypes: true })) {
      if (f.isDirectory()) walk(join(d, f.name));
      else if (f.name.endsWith(".html")) out.push(join(d, f.name));
    }
  };
  walk(dir);
  return out.map((file) => {
    const raw = readFileSync(file, "utf8");
    const m = raw.match(/^\s*---\n([\s\S]*?)\n---\n([\s\S]*)$/);
    if (!m) throw new Error(`${file}: missing front matter`);
    const meta = JSON.parse(m[1]);
    const rel = file.slice(dir.length + 1).replace(/\\/g, "/").replace(/\.html$/, "");
    meta.slug = meta.slug ?? (rel === "index" ? "" : rel);
    meta.body = m[2].trim();
    return meta;
  });
}

// ---------- shared layout ----------
const NAV = [
  { href: "/", label: "Home" },
  { href: "/concepts/", label: "Concepts" },
  { href: "/architecture/", label: "Architecture" },
  { href: "/research/", label: "Research" },
  { href: "/specifications/", label: "Specs" },
  { href: "/faq/", label: "FAQ" },
];

function breadcrumb(meta) {
  const trail = [{ href: "/", label: "RAHN" }];
  if (meta.group) trail.push({ href: `/${meta.group.slug}/`, label: meta.group.name });
  trail.push({ href: `${BASE}/${meta.slug}`, label: meta.crumb });
  const items = trail
    .map((t, i) =>
      i === trail.length - 1
        ? `<li aria-current="page">${t.label}</li>`
        : `<li><a href="${t.href}">${t.label}</a></li>`,
    )
    .join('<li aria-hidden="true">›</li>');
  return `<nav class="breadcrumb" aria-label="Breadcrumb"><ol>${items}</ol></nav>`;
}

function breadcrumbLD(meta) {
  const trail = meta.group
    ? [`${BASE}/`, `${BASE}/${meta.group.slug}/`, `${BASE}/${meta.slug}`]
    : [`${BASE}/`, `${BASE}/${meta.slug}`];
  const names = meta.group
    ? ["RAHN", meta.group.name, meta.crumb]
    : ["RAHN", meta.crumb];
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: trail.map((url, i) => ({
      "@type": "ListItem",
      position: i + 1,
      name: names[i],
      item: url,
    })),
  };
}

function schemaLD(meta) {
  if (!meta.schema) return null;
  if (meta.schema === "TechArticle") {
    return {
      "@context": "https://schema.org",
      "@type": "TechArticle",
      headline: meta.title,
      description: meta.description,
      url: `${BASE}/${meta.slug}`,
      image: `${BASE}/assets/img/rahn-hero.png`,
      inLanguage: "en",
      isPartOf: { "@type": "WebSite", name: "RAHN", url: `${BASE}/` },
      about: { "@type": "SoftwareSourceCode", name: "RAHN", codeRepository: GH, programmingLanguage: "Rust" },
      license: "https://spdx.org/licenses/Apache-2.0",
    };
  }
  if (meta.schema === "FAQPage") {
    const qa = [...meta.body.matchAll(/<summary>([\s\S]*?)<\/summary>[\s\S]*?<p>([\s\S]*?)<\/p>/g)].map((m) => ({
      "@type": "Question",
      name: m[1].replace(/<[^>]+>/g, "").trim(),
      acceptedAnswer: { "@type": "Answer", text: m[2].replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim() },
    }));
    return qa.length ? { "@context": "https://schema.org", "@type": "FAQPage", mainEntity: qa } : null;
  }
  return null;
}

function head(meta) {
  const url = `${BASE}/${meta.slug}`;
  const ogType = meta.ogType ?? "article";
  const schemas = [breadcrumbLD(meta), schemaLD(meta), meta.extraSchema ?? null].filter(Boolean);
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${meta.title}</title>
  <meta name="description" content="${meta.description}">
  ${meta.robots ? `<meta name="robots" content="${meta.robots}">` : '<meta name="robots" content="index, follow">'}
  <link rel="canonical" href="${url}">
  <meta name="theme-color" content="#264c54">
  <meta property="og:type" content="${ogType}">
  <meta property="og:site_name" content="RAHN">
  <meta property="og:title" content="${meta.title}">
  <meta property="og:description" content="${meta.description}">
  <meta property="og:url" content="${url}">
  <meta property="og:image" content="${BASE}/assets/img/rahn-hero.png">
  <meta property="og:image:width" content="1672">
  <meta property="og:image:height" content="536">
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:title" content="${meta.title}">
  <meta name="twitter:description" content="${meta.description}">
  <meta name="twitter:image" content="${BASE}/assets/img/rahn-hero.png">
  <link rel="icon" type="image/png" sizes="32x32" href="/rahn/assets/img/favicon-32.png">
  <link rel="icon" type="image/png" sizes="180x180" href="/rahn/assets/img/favicon-180.png">
  <link rel="icon" type="image/png" sizes="512x512" href="/rahn/assets/img/favicon-512.png">
  <link rel="apple-touch-icon" href="/rahn/assets/img/favicon-180.png">
  <link rel="stylesheet" href="/rahn/assets/css/style.css">
${schemas.map((s) => `  <script type="application/ld+json">${JSON.stringify(s)}</script>`).join("\n")}
</head>
<body>
  <a class="skip-link" href="#main">Skip to content</a>
  <header class="site-header" id="top">
    <div class="container header-inner">
      <a class="brand" href="/rahn/" aria-label="RAHN home">
        <span class="brand-mark" aria-hidden="true"><img src="/rahn/assets/img/mark-teal.png" alt="" width="24" height="35"></span>
        <span class="brand-name">RAHN</span>
      </a>
      <nav class="site-nav" id="site-nav" aria-label="Primary">
${NAV.map((n) => `        <a href="/rahn${n.href}"${meta.slug === n.href.slice(1) ? ' aria-current="page"' : ""}>${n.label}</a>`).join("\n")}
        <a class="nav-github" href="${GH}" rel="noopener"><svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true" focusable="false"><path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>GitHub</a>
      </nav>
      <button class="nav-toggle" id="nav-toggle" aria-expanded="false" aria-controls="site-nav" aria-label="Toggle navigation menu"><span></span><span></span><span></span></button>
    </div>
  </header>`;
}

function footer() {
  return `  <footer class="site-footer">
    <div class="container footer-grid">
      <div class="footer-brand">
        <span class="brand-name">RAHN</span>
        <p>A stateful execution architecture for evolving networks. Research project — not production software.</p>
      </div>
      <nav class="footer-col" aria-label="Concepts">
        <h3>Concepts</h3>
        <a href="/rahn/concepts/network-state/">Network state</a>
        <a href="/rahn/concepts/deterministic-state/">Deterministic state</a>
        <a href="/rahn/concepts/content-addressing/">Content addressing</a>
        <a href="/rahn/concepts/network-verification/">Verification</a>
        <a href="/rahn/concepts/network-invariants/">Invariants</a>
        <a href="/rahn/concepts/causal-memory/">Causal memory</a>
        <a href="/rahn/concepts/simulation-first-execution/">Simulation-first execution</a>
      </nav>
      <nav class="footer-col" aria-label="Project">
        <h3>Project</h3>
        <a href="/rahn/architecture/">Architecture</a>
        <a href="/rahn/research/">Research</a>
        <a href="/rahn/specifications/">Specifications</a>
        <a href="/rahn/whitepaper/">White paper</a>
        <a href="/rahn/rahn-vs-git/">RAHN vs Git</a>
        <a href="/rahn/why-rust/">Why Rust</a>
      </nav>
      <nav class="footer-col" aria-label="Community">
        <h3>Community</h3>
        <a href="/rahn/contributing/">Contributing</a>
        <a href="/rahn/faq/">FAQ</a>
        <a href="${GH}/discussions" rel="noopener">Discussions</a>
        <a href="${GH}/releases" rel="noopener">Releases</a>
        <a href="${GH}/blob/main/docs/community/good-first-issues.md" rel="noopener">Good first issues</a>
      </nav>
    </div>
    <div class="container footer-legal">
      <p>Code licensed Apache-2.0 · Documents licensed CC-BY-4.0 · RAHN is an architectural research project, not a production network controller.</p>
      <a href="#top" class="to-top">Back to top ↑</a>
    </div>
  </footer>
  <script src="/rahn/assets/js/main.js" defer></script>
</body>
</html>`;
}

// ---------- build ----------
const pages = loadPages();

// uniqueness gates: titles and descriptions must be unique across the site
for (const [key, field] of [["title", "title"], ["description", "description"]]) {
  const seen = new Map();
  for (const p of pages) {
    if (seen.has(p[field])) throw new Error(`duplicate ${key}: "${p[field]}" (${seen.get(p[field])} and ${p.slug})`);
    seen.set(p[field], p.slug);
  }
}

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });
cpSync(join(SRC, "assets"), join(OUT, "assets"), { recursive: true });
cpSync(join(SRC, "static"), join(OUT), { recursive: true });

const slugs = [];
for (const p of pages) {
  const dir = join(OUT, p.slug);
  mkdirSync(dir, { recursive: true });
  const article = p.plain ? p.body : `<article class="doc">${p.group ? breadcrumb(p) : ""}${p.body}</article>`;
  writeFileSync(join(dir, "index.html"), `${head(p)}\n<main id="main">\n${article}\n</main>\n${footer()}`);
  slugs.push(p.slug);
}

// generated sitemap — single source of truth: the page list
const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${slugs
  .filter((s) => s !== "404")
  .map((s) => `  <url>\n    <loc>${BASE}/${s}</loc>\n    <changefreq>monthly</changefreq>\n    <priority>${s === "" ? "1.0" : "0.8"}</priority>\n  </url>`)
  .join("\n")}
</urlset>
`;
writeFileSync(join(OUT, "sitemap.xml"), sitemap);

console.log(`built ${slugs.length} pages → site/`);
console.log(slugs.map((s) => `  /${s}`).join("\n"));
