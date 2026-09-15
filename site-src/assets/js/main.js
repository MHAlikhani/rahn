// SPDX-License-Identifier: Apache-2.0
/* RAHN site — minimal, dependency-free enhancements.
   The page is fully functional without this file. */
(function () {
  "use strict";

  // Mobile navigation toggle
  var toggle = document.getElementById("nav-toggle");
  var nav = document.getElementById("site-nav");
  if (toggle && nav) {
    toggle.addEventListener("click", function () {
      var open = nav.classList.toggle("open");
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
    });
    // Close the menu after navigating
    nav.addEventListener("click", function (e) {
      if (e.target.closest("a")) {
        nav.classList.remove("open");
        toggle.setAttribute("aria-expanded", "false");
      }
    });
    // Close on Escape
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && nav.classList.contains("open")) {
        nav.classList.remove("open");
        toggle.setAttribute("aria-expanded", "false");
        toggle.focus();
      }
    });
  }

  // FAQ: close other open items when one is opened (accordion feel, optional)
  var faq = document.querySelectorAll(".faq details");
  faq.forEach(function (d) {
    d.addEventListener("toggle", function () {
      if (d.open) {
        faq.forEach(function (other) {
          if (other !== d) other.open = false;
        });
      }
    });
  });

  // Header shadow once scrolled
  var header = document.querySelector(".site-header");
  if (header) {
    var onScroll = function () {
      header.style.boxShadow = window.scrollY > 8 ? "0 6px 18px rgba(19,31,38,0.08)" : "none";
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
  }
})();
