// SPDX-License-Identifier: Apache-2.0
// Static SEO/consistency validation for the generated site/. Zero dependencies.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const OUT = join(root, "site");
const BASE = "https://mhalikhani.github.io/rahn";
const GH = "https://github.com/MHAlikhani/rahn";
const errors = [];
const warnings = [];

// gather pages
const pages = [];
(function walk(d) {
  for (const f of readdirSync(d, { withFileTypes: true })) {
    if (f.isDirectory()) walk(join(d, f.name));
    else if (f.name === "index.html") pages.push(join(d, f.name));
  }
})(OUT);

// repo files for external GitHub-link verification
const repoFiles = new Set(
  execSync("git ls-files", { cwd: root }).toString().trim().split("\n"),
);

let titles = new Map(), descriptions = new Map(), canonicals = new Map();
for (const p of pages) {
  const html = readFileSync(p, "utf8");
  const rel = p.slice(OUT.length + 1, -"/index.html".length).replace(/\\/g, "/");
  const url = rel === "404" ? `${BASE}/404.html` : `${BASE}/${rel}${rel === "" ? "" : "/"}`;

  const title = html.match(/<title>(.*?)<\/title>/)?.[1];
  const desc = html.match(/<meta name="description" content="([^"]*)"/)?.[1];
  const canon = html.match(/<link rel="canonical" href="([^"]*)"/)?.[1];

  if (rel !== "404") {
    if (!title) errors.push(`${rel}: no title`);
    else if (titles.has(title)) errors.push(`${rel}: duplicate title with ${titles.get(title)}`);
    else titles.set(title, rel);
    if (!desc) errors.push(`${rel}: no meta description`);
    else if (descriptions.has(desc)) errors.push(`${rel}: duplicate description with ${descriptions.get(desc)}`);
    else descriptions.set(desc, rel);
    if (!desc || desc.length < 70 || desc.length > 165)
      warnings.push(`${rel}: description length ${desc?.length} (aim 70–165)`);
  }
  if (canon !== url) errors.push(`${rel}: canonical '${canon}' != expected '${url}'`);
  if (!html.includes('lang="en"')) errors.push(`${rel}: missing lang`);
  if (!html.includes("og:title") || !html.includes("og:image") || !html.includes("twitter:card"))
    errors.push(`${rel}: incomplete OG/Twitter metadata`);
  if (!html.includes("<h1")) errors.push(`${rel}: no H1`);
  if (/ (here|click here|read more|learn more)\.?<\/a>/i.test(html))
    warnings.push(`${rel}: generic anchor text found`);

  // JSON-LD must parse
  for (const m of html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)) {
    try { JSON.parse(m[1]); } catch (e) { errors.push(`${rel}: invalid JSON-LD (${e.message})`); }
  }
  if (rel !== "404") {
    const ld = [...html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)].map((m) => JSON.parse(m[1]));
    if (!ld.some((s) => s["@type"] === "BreadcrumbList")) errors.push(`${rel}: no BreadcrumbList`);
    if (rel === "faq/" && !ld.some((s) => s["@type"] === "FAQPage")) errors.push(`${rel}: no FAQPage schema`);
    if ((rel === "" || rel === "why-rust/" || rel === "rahn-vs-git/" || rel === "research/") && !ld.some((s) => s["@type"] === "TechArticle" || s["@type"] === "SoftwareSourceCode"))
      errors.push(`${rel}: missing content schema`);
  }

  // link checks
  for (const m of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
    const link = m[1];
    if (link.startsWith("#") || link.startsWith("mailto:")) continue;
    if (link.startsWith(BASE)) {
      const path = link.slice(BASE.length).split("#")[0];
      if (path === "" || path === "/") continue;
      const target = join(OUT, path);
      try { statSync(target); } catch { errors.push(`${rel}: broken internal ${link}`); }
    } else if (link.startsWith("https://github.com/MHAlikhani/rahn/")) {
      const path = link.slice(`${GH}/`).split("#")[0].replace(/\/$/, "");
      if (!path.startsWith("blob/") && !path.startsWith("tree/")) continue;
      const repoPath = path.replace(/^(blob|tree)\//, "").split("/").slice(1).join("/"); // strip branch
      if (!repoFiles.has(repoPath)) errors.push(`${rel}: GH link to nonexistent repo file: ${link}`);
    }
  }
}

// sitemap vs pages
const sm = readFileSync(join(OUT, "sitemap.xml"), "utf8");
const smUrls = [...sm.matchAll(/<loc>(.*?)<\/loc>/g)].map((m) => m[1]);
const expected = pages
  .map((p) => {
    const rel = p.slice(OUT.length + 1, -"index.html".length).replace(/\\/g, "/");
    return rel === "404" ? null : rel === "" ? `${BASE}/` : `${BASE}/${rel}`;
  })
  .filter(Boolean);
for (const u of expected) if (!smUrls.includes(u)) errors.push(`sitemap missing ${u}`);
for (const u of smUrls) if (!expected.includes(u)) errors.push(`sitemap has extra ${u}`);
if (!smUrls.includes(`${BASE}/`)) errors.push("sitemap missing homepage");

// robots
const robots = readFileSync(join(OUT, "robots.txt"), "utf8");
if (!robots.includes("Sitemap: https://mhalikhani.github.io/rahn/sitemap.xml")) errors.push("robots.txt missing sitemap");

console.log(`pages: ${pages.length}, sitemap urls: ${smUrls.length}`);
for (const w of warnings) console.log("WARN: " + w);
for (const e of errors) console.log("ERROR: " + e);
console.log(errors.length ? `FAILED with ${errors.length} error(s)` : "ALL CHECKS PASSED");
process.exit(errors.length ? 1 : 0);
