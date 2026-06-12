/* MAD interactive HTML report — self-contained, no dependencies */
(function () {
  "use strict";

  const dataEl = document.getElementById("mad-report-data");
  const payload = dataEl ? JSON.parse(dataEl.textContent || "{}") : {};
  const ui = payload.ui || {};
  const embed = new URLSearchParams(location.search).get("embed") === "1";
  if (embed) document.body.classList.add("mad-embed");

  function qs(sel, root) {
    return (root || document).querySelector(sel);
  }
  function qsa(sel, root) {
    return Array.from((root || document).querySelectorAll(sel));
  }

  /* ── Sticky nav + scroll spy ─────────────────────────────────────── */
  function initNav() {
    const nav = qs("#mad-report-nav");
    if (!nav) return;
    const links = qsa("a[href^='#']", nav);
    const sections = links
      .map((a) => {
        const id = a.getAttribute("href").slice(1);
        const el = document.getElementById(id);
        return el ? { a, el } : null;
      })
      .filter(Boolean);

    function onScroll() {
      const y = window.scrollY + 120;
      let current = sections[0];
      for (const s of sections) {
        if (s.el.offsetTop <= y) current = s;
      }
      links.forEach((a) => a.classList.remove("active"));
      if (current) current.a.classList.add("active");
    }
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
  }

  /* ── Dashboard score cards → vendor filter ───────────────────────── */
  function initDashboardClicks() {
    qsa(".mad-score-card").forEach((card) => {
      card.addEventListener("click", () => {
        const id = card.dataset.vendorId;
        if (!id) return;
        const btn = qs(`#mad-vendor-filter button[data-vendor="${id}"]`);
        if (btn) btn.click();
        document.getElementById("section-results")?.scrollIntoView({ behavior: "smooth" });
      });
    });
  }

  /* ── Vendor filter ───────────────────────────────────────────────── */
  function initVendorFilter() {
    const bar = qs("#mad-vendor-filter");
    if (!bar) return;
    const cards = qsa("[data-vendor-id]");
    const vsmCards = qsa("[data-vendor-vsm]");
    const docCards = qsa("[data-vendor-docs]");

    function apply(vendorId) {
      const showAll = !vendorId || vendorId === "all";
      qsa("button", bar).forEach((btn) => {
        btn.classList.toggle("active", btn.dataset.vendor === vendorId);
      });
      const match = (el) =>
        showAll || el.dataset.vendorId === vendorId || el.dataset.vendorVsm === vendorId || el.dataset.vendorDocs === vendorId;
      cards.forEach((el) => el.classList.toggle("mad-hidden", !match(el)));
      vsmCards.forEach((el) => el.classList.toggle("mad-hidden", !showAll && el.dataset.vendorVsm !== vendorId));
      docCards.forEach((el) => el.classList.toggle("mad-hidden", !showAll && el.dataset.vendorDocs !== vendorId));
    }

    qsa("button", bar).forEach((btn) => {
      btn.addEventListener("click", () => apply(btn.dataset.vendor));
    });
    apply("all");
  }

  /* ── VSM interactive viewer ──────────────────────────────────────── */
  function initVsmViewers() {
    qsa(".mad-vsm-viewer").forEach((viewer) => {
      const payloadEl = qs(".mad-vsm-payload", viewer);
      if (!payloadEl) return;
      let mapData;
      try {
        mapData = JSON.parse(payloadEl.textContent || "{}");
      } catch {
        return;
      }
      const nodesById = Object.fromEntries((mapData.nodes || []).map((n) => [n.id, n]));
      const segmentsByEdgeId = Object.fromEntries(
        (mapData.segments || []).map((s) => [s.edge_id, s]),
      );
      const inspector = qs(".mad-vsm-inspector", viewer);
      const stage = qs(".mad-vsm-stage", viewer);
      const world = qs(".mad-vsm-world", viewer);
      if (!stage || !world) return;

      let scale = 1;
      let tx = 0;
      let ty = 0;
      let dragging = false;
      let lx = 0;
      let ly = 0;

      function applyTransform() {
        world.setAttribute(
          "transform",
          `translate(${tx} ${ty}) scale(${scale})`,
        );
      }

      function bindInspectorClose() {
        qs(".mad-inspector-close", inspector)?.addEventListener("click", () => {
          inspector.classList.add("mad-hidden");
          clearSelection();
        });
      }

      function clearSelection() {
        qsa(".mad-vsm-node.selected", viewer).forEach((n) => n.classList.remove("selected"));
        qsa(".mad-vsm-node.edge-source", viewer).forEach((n) => n.classList.remove("edge-source"));
        qsa(".mad-vsm-node.edge-target", viewer).forEach((n) => n.classList.remove("edge-target"));
        qsa(".mad-vsm-edge.selected", viewer).forEach((n) => n.classList.remove("selected"));
        qsa(".mad-vsm-timeline-bar.selected", viewer).forEach((n) => n.classList.remove("selected"));
        qsa(".mad-vsm-timeline-milestone.selected", viewer).forEach((n) =>
          n.classList.remove("selected"),
        );
        qsa(".mad-vsm-timeline-row.selected", viewer).forEach((n) => n.classList.remove("selected"));
      }

      function switchToDiagram() {
        const tab = qs('[data-vsm-tab="diagram"]', viewer);
        if (tab) tab.click();
      }

      function showNode(nodeId) {
        if (!inspector) return;
        const node = nodesById[nodeId];
        if (!node) {
          inspector.classList.add("mad-hidden");
          return;
        }
        inspector.classList.remove("mad-hidden");
        const lead = node.lead_time_minutes > 0 ? formatDur(node.lead_time_minutes) : "—";
        const cycle = node.cycle_time_minutes > 0 ? formatDur(node.cycle_time_minutes) : "—";
        inspector.innerHTML = `
          <h5>${esc(node.label)}</h5>
          <dl>
            <dt>${esc(ui.type || "Type")}</dt><dd>${esc(node.node_type || "process")}</dd>
            <dt>${esc(ui.author || "Author")}</dt><dd>${esc(node.author || "—")}</dd>
            <dt>${esc(ui.role || "Role")}</dt><dd>${esc(node.role || "—")}</dd>
            <dt>${esc(ui.leadTime || "Lead time")}</dt><dd>${lead}</dd>
            <dt>${esc(ui.cycleTime || "Cycle time")}</dt><dd>${cycle}</dd>
          </dl>
          ${node.notes ? `<p class="mad-vsm-notes">${esc(node.notes)}</p>` : ""}
          <button type="button" class="mad-btn-sm mad-inspector-close">${esc(ui.close || "Close")}</button>`;
        bindInspectorClose();
      }

      function showEdge(segment) {
        if (!inspector) return;
        inspector.classList.remove("mad-hidden");
        const dur =
          segment.duration_minutes > 0 ? formatDur(segment.duration_minutes) : "—";
        const edgeLabel = segment.edge_label
          ? `<dt>${esc(ui.label || "Label")}</dt><dd>${esc(segment.edge_label)}</dd>`
          : "";
        inspector.innerHTML = `
          <h5>${esc(segment.from_label)} → ${esc(segment.to_label)}</h5>
          <dl>
            <dt>${esc(ui.flowType || "Flow type")}</dt><dd>${esc(segment.flow_type_label || segment.edge_type)}</dd>
            <dt>${esc(ui.duration || "Duration")}</dt><dd>${dur}</dd>
            ${edgeLabel}
          </dl>
          <button type="button" class="mad-btn-sm mad-inspector-close">${esc(ui.close || "Close")}</button>`;
        bindInspectorClose();
      }

      function selectEdge(edgeId, opts) {
        const segment = segmentsByEdgeId[edgeId];
        if (!segment) return;
        clearSelection();
        if (!opts || opts.switchTab !== false) switchToDiagram();
        qsa(`.mad-vsm-timeline-bar[data-edge-id="${edgeId}"]`, viewer).forEach((b) =>
          b.classList.add("selected"),
        );
        qsa(`.mad-vsm-timeline-row[data-edge-id="${edgeId}"]`, viewer).forEach((r) =>
          r.classList.add("selected"),
        );
        qsa(`.mad-vsm-edge[data-edge-id="${edgeId}"]`, viewer).forEach((e) =>
          e.classList.add("selected"),
        );
        qsa(`.mad-vsm-node[data-node-id="${segment.from_id}"]`, viewer).forEach((n) =>
          n.classList.add("edge-source"),
        );
        qsa(`.mad-vsm-node[data-node-id="${segment.to_id}"]`, viewer).forEach((n) =>
          n.classList.add("edge-target"),
        );
        showEdge(segment);
        const bar = qs(`.mad-vsm-timeline-bar[data-edge-id="${edgeId}"]`, viewer);
        bar?.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "center" });
      }

      function selectNode(nodeId, opts) {
        clearSelection();
        if ((!opts || opts.switchTab !== false) && !(opts && opts.fromDiagram)) {
          switchToDiagram();
        }
        qsa(`.mad-vsm-node[data-node-id="${nodeId}"]`, viewer).forEach((n) =>
          n.classList.add("selected"),
        );
        qsa(`.mad-vsm-timeline-milestone[data-node-id="${nodeId}"]`, viewer).forEach((m) =>
          m.classList.add("selected"),
        );
        showNode(nodeId);
        const milestone = qs(`.mad-vsm-timeline-milestone[data-node-id="${nodeId}"]`, viewer);
        milestone?.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "center" });
      }

      qsa(".mad-vsm-node", viewer).forEach((g) => {
        g.addEventListener("click", (e) => {
          e.stopPropagation();
          selectNode(g.dataset.nodeId, { fromDiagram: true, switchTab: false });
        });
      });

      qsa(".mad-vsm-edge", viewer).forEach((g) => {
        g.addEventListener("click", (e) => {
          e.stopPropagation();
          selectEdge(g.dataset.edgeId, { switchTab: false });
        });
      });

      qsa(".mad-vsm-timeline-bar[data-edge-id]", viewer).forEach((bar) => {
        bar.addEventListener("click", (e) => {
          e.preventDefault();
          selectEdge(bar.dataset.edgeId);
        });
      });

      qsa(".mad-vsm-timeline-milestone[data-node-id]", viewer).forEach((btn) => {
        btn.addEventListener("click", (e) => {
          e.preventDefault();
          selectNode(btn.dataset.nodeId);
        });
      });

      qsa(".mad-vsm-timeline-row[data-edge-id]", viewer).forEach((row) => {
        row.addEventListener("click", () => selectEdge(row.dataset.edgeId));
        row.addEventListener("keydown", (e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            selectEdge(row.dataset.edgeId);
          }
        });
      });

      stage.addEventListener("wheel", (e) => {
        e.preventDefault();
        const delta = e.deltaY > 0 ? 0.9 : 1.1;
        scale = Math.min(3, Math.max(0.35, scale * delta));
        applyTransform();
      }, { passive: false });

      stage.addEventListener("pointerdown", (e) => {
        if (e.target.closest(".mad-vsm-node")) return;
        dragging = true;
        lx = e.clientX;
        ly = e.clientY;
        stage.setPointerCapture(e.pointerId);
      });
      stage.addEventListener("pointermove", (e) => {
        if (!dragging) return;
        tx += e.clientX - lx;
        ty += e.clientY - ly;
        lx = e.clientX;
        ly = e.clientY;
        applyTransform();
      });
      stage.addEventListener("pointerup", () => {
        dragging = false;
      });

      qsa("[data-vsm-action]", viewer).forEach((btn) => {
        btn.addEventListener("click", () => {
          const action = btn.dataset.vsmAction;
          if (action === "zoom-in") scale = Math.min(3, scale * 1.15);
          if (action === "zoom-out") scale = Math.max(0.35, scale / 1.15);
          if (action === "reset") {
            scale = 1;
            tx = 0;
            ty = 0;
          }
          applyTransform();
        });
      });

      qsa("[data-vsm-tab]", viewer).forEach((tab) => {
        tab.addEventListener("click", () => {
          const name = tab.dataset.vsmTab;
          qsa("[data-vsm-tab]", viewer).forEach((t) => t.classList.toggle("active", t === tab));
          qsa("[data-vsm-panel]", viewer).forEach((p) =>
            p.classList.toggle("mad-hidden", p.dataset.vsmPanel !== name),
          );
        });
      });
    });
  }

  /* ── Vendor documentation filters ────────────────────────────────── */
  function initDocFilters() {
    qsa(".mad-doc-filter").forEach((bar) => {
      const card = bar.closest(".vendor-doc-report-card");
      if (!card) return;
      const items = qsa(".vendor-doc-item", card);
      qsa("button", bar).forEach((btn) => {
        btn.addEventListener("click", () => {
          const color = btn.dataset.docColor;
          qsa("button", bar).forEach((b) => b.classList.toggle("active", b === btn));
          items.forEach((item) => {
            const c = item.dataset.docColor || "";
            const show = color === "all" || c === color;
            item.classList.toggle("mad-hidden", !show);
          });
        });
      });
    });
  }

  /* ── Collapsible vendor results ────────────────────────────────── */
  function initVendorCollapse() {
    qsa(".mad-vendor-toggle").forEach((btn) => {
      btn.addEventListener("click", () => {
        const card = btn.closest(".vendor-card");
        if (!card) return;
        card.classList.toggle("mad-collapsed");
        const expandLabel = btn.dataset.expand || ui.expand || "Expand";
        const collapseLabel = btn.dataset.collapse || ui.collapse || "Collapse";
        btn.textContent = card.classList.contains("mad-collapsed") ? expandLabel : collapseLabel;
      });
    });
  }

  function formatDur(minutes) {
    const m = Math.round(minutes);
    if (m < 60) return m + "m";
    if (m < 1440) return Math.floor(m / 60) + "h " + (m % 60 ? (m % 60) + "m" : "").trim();
    return Math.floor(m / 1440) + "d";
  }

  function esc(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  /* ── iframe embed: notify parent of height ─────────────────────── */
  function notifyEmbedHeight() {
    if (!embed || !window.parent || window.parent === window) return;
    const h = document.documentElement.scrollHeight;
    window.parent.postMessage({ type: "mad-report-resize", height: h }, "*");
  }

  function boot() {
    initNav();
    initDashboardClicks();
    initVendorFilter();
    initVsmViewers();
    initDocFilters();
    initVendorCollapse();
    notifyEmbedHeight();
    window.addEventListener("resize", notifyEmbedHeight);
    if (embed) {
      const obs = new MutationObserver(notifyEmbedHeight);
      obs.observe(document.body, { childList: true, subtree: true });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
