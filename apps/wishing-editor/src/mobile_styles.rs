pub(crate) const MOBILE_STYLES: &str = r#"
  .mobile-shell {
    display: none;
    min-width: 0;
  }
  .mobile-screen {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    background: #121212;
    color: #f4f7fb;
  }
  .mobile-page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 18px 16px 14px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: #1c1c1e;
  }
  .mobile-page-header h1 {
    margin: 0;
    font-size: 20px;
    text-align: center;
    font-weight: 700;
    flex: 1;
  }
  .mobile-inline-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 36px;
    padding: 0 10px;
    border-radius: 10px;
    background: rgba(77, 150, 255, 0.12);
    border: 1px solid rgba(77, 150, 255, 0.18);
    color: #70abff;
  }
  .mobile-inline-action.subtle {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.08);
    color: #d2dbeb;
  }
  .mobile-inline-action.ghost {
    background: transparent;
    border-color: transparent;
    color: #73849d;
  }
  .mobile-page-body {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    min-height: 0;
  }
  .mobile-section-label {
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #7e8fa7;
  }
  .mobile-card,
  .mobile-dashboard-hero,
  .mobile-project-card,
  .mobile-layer-card,
  .mobile-placeholder-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.06);
    background: #1c1c1e;
  }
  .mobile-placeholder-card {
    color: #dbe3ef;
  }
  .mobile-placeholder-card.compact {
    border-style: dashed;
    color: #93a3bb;
  }
  .mobile-dashboard-header {
    padding-top: 18px;
  }
  .mobile-dashboard-hero {
    align-items: flex-start;
    text-align: left;
    background:
      radial-gradient(circle at top right, rgba(77, 150, 255, 0.24), transparent 34%),
      linear-gradient(180deg, #24262c 0%, #18191c 100%);
  }
  .mobile-dashboard-hero-title {
    font-size: 20px;
    font-weight: 700;
  }
  .mobile-dashboard-hero-meta,
  .mobile-project-meta,
  .mobile-status-copy {
    color: #a5b1c2;
    line-height: 1.45;
  }
  .mobile-project-card {
    flex-direction: row;
    align-items: center;
  }
  .mobile-project-card.placeholder {
    opacity: 0.88;
  }
  .mobile-project-thumb {
    width: 64px;
    height: 64px;
    border-radius: 18px;
    flex: none;
    background:
      linear-gradient(135deg, rgba(77, 150, 255, 0.36), rgba(0, 120, 212, 0.12)),
      #26303f;
  }
  .mobile-project-thumb.live {
    background:
      linear-gradient(135deg, rgba(111, 187, 94, 0.42), rgba(245, 184, 90, 0.16)),
      #243126;
  }
  .mobile-project-thumb.placeholder {
    background:
      linear-gradient(135deg, rgba(255,255,255,0.08), rgba(255,255,255,0.02)),
      #2a2c31;
  }
  .mobile-project-copy {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .mobile-project-title {
    font-size: 18px;
    font-weight: 700;
    color: #f4f7fb;
  }
  .mobile-card-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .mobile-editor-screen {
    background: #121212;
  }
  .mobile-editor-header {
    padding-top: 14px;
  }
  .mobile-editor-title {
    flex: 1;
    min-width: 0;
    text-align: center;
  }
  .mobile-editor-title h1 {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
  }
  .mobile-editor-title p {
    margin: 4px 0 0;
    font-size: 12px;
    color: #9babbe;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .mobile-editor-canvas-shell {
    position: relative;
    flex: 1;
    min-height: 48vh;
    background: #2a2a2a;
    overflow: hidden;
  }
  .mobile-editor-screen .canvas-host {
    height: 100%;
    padding: 0;
    background:
      linear-gradient(90deg, rgba(255,255,255,0.03) 1px, transparent 1px),
      linear-gradient(180deg, rgba(255,255,255,0.03) 1px, transparent 1px),
      #2a2a2a;
  }
  .mobile-editor-screen .canvas-stage {
    min-height: 100%;
  }
  .mobile-editor-screen .canvas {
    border-radius: 0;
    box-shadow: none;
  }
  .mobile-layer-overlay {
    position: absolute;
    right: 16px;
    bottom: 116px;
    width: min(50vw, 220px);
    padding: 10px;
    border-radius: 18px;
    background: rgba(19, 22, 29, 0.78);
    border: 1px solid rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(16px);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .mobile-overlay-header,
  .mobile-overlay-layer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .mobile-overlay-layer button {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    background: transparent;
    border: none;
    padding: 0;
    color: inherit;
    text-align: left;
  }
  .mobile-overlay-layer.active {
    color: #ffffff;
  }
  .mobile-overlay-layer:not(.active) {
    color: #a7b3c3;
  }
  .mobile-overlay-layer span {
    font-size: 12px;
  }
  .mobile-dpad {
    position: absolute;
    left: 16px;
    bottom: 112px;
    width: 120px;
    height: 120px;
    border-radius: 999px;
    background: rgba(19, 22, 29, 0.78);
    border: 1px solid rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(16px);
  }
  .mobile-dpad button {
    position: absolute;
    width: 42px;
    height: 42px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: #e9eef7;
  }
  .mobile-dpad .up { top: 6px; left: 39px; }
  .mobile-dpad .left { top: 39px; left: 6px; }
  .mobile-dpad .center {
    top: 39px;
    left: 39px;
    font-size: 12px;
    font-weight: 700;
    background: rgba(77, 150, 255, 0.18);
    color: #70abff;
  }
  .mobile-dpad .right { top: 39px; right: 6px; }
  .mobile-dpad .down { bottom: 6px; left: 39px; }
  .mobile-tool-tray {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px 14px 10px;
    background: #1c1c1e;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  .mobile-tool-row {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 8px;
  }
  .mobile-tool-row button,
  .mobile-inline-actions button,
  .mobile-bottom-nav button {
    border-radius: 14px;
    background: #24262b;
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #dfe7f2;
  }
  .mobile-tool-row button {
    min-height: 56px;
    font-size: 12px;
    font-weight: 600;
  }
  .mobile-tool-row button.active {
    background: #2b5e9a;
    border-color: rgba(112, 171, 255, 0.36);
    color: #ffffff;
  }
  .mobile-inline-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .mobile-inline-actions.wide {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
  .mobile-inline-actions.wide button {
    min-height: 42px;
  }
  .mobile-tile-strip {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 2px;
  }
  .mobile-tile-chip,
  .mobile-tileset-cell {
    flex: none;
    width: 56px;
    height: 56px;
    padding: 0;
    border-radius: 14px;
    background-color: #11161f;
    background-repeat: no-repeat;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .mobile-tile-chip.active,
  .mobile-tileset-cell.active {
    border: 2px solid #70abff;
  }
  .mobile-bottom-nav {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 8px;
    padding: 12px 10px calc(12px + env(safe-area-inset-bottom, 0px));
    background: #1c1c1e;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  .mobile-bottom-nav button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-height: 58px;
    font-size: 11px;
    color: #8d99ab;
  }
  .mobile-bottom-nav button.active {
    color: #70abff;
  }
  .mobile-nav-pill {
    width: 22px;
    height: 6px;
    border-radius: 999px;
    background: currentColor;
    opacity: 0.28;
  }
  .mobile-bottom-nav button.active .mobile-nav-pill {
    opacity: 1;
  }

  @media (max-width: 900px) {
    .topbar,
    .workspace {
      display: none;
    }
    .mobile-shell {
      display: flex;
      flex-direction: column;
      min-height: 100dvh;
    }
  }
"#;
