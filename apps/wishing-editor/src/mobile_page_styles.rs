pub(crate) const MOBILE_PAGE_STYLES: &str = r#"
  .mobile-tileset-grid,
  .mobile-object-grid {
    display: grid;
    gap: 10px;
  }
  .mobile-tileset-grid {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
  .mobile-object-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  .mobile-selected-preview {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .mobile-selected-preview-art {
    width: 72px;
    height: 72px;
    border-radius: 18px;
    background-color: #11161f;
    background-repeat: no-repeat;
    border: 1px solid rgba(255,255,255,0.08);
    flex: none;
  }
  .mobile-selected-preview-copy {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .mobile-layer-card {
    gap: 12px;
  }
  .mobile-layer-card.active {
    border-color: rgba(112, 171, 255, 0.36);
    box-shadow: inset 0 0 0 1px rgba(112, 171, 255, 0.18);
  }
  .mobile-layer-card-main {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .mobile-layer-name {
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
  .mobile-layer-name span {
    font-size: 12px;
    color: #8e9bb0;
  }
  .mobile-layer-actions {
    display: flex;
    gap: 8px;
  }
  .mobile-layer-actions button.on {
    color: #70abff;
  }
  .mobile-layer-actions button.off {
    color: #8e9bb0;
  }
  .mobile-progress-row,
  .mobile-settings-row {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: space-between;
  }
  .mobile-progress-row {
    color: #a7b3c3;
    font-size: 12px;
  }
  .mobile-progress-track {
    flex: 1;
    height: 6px;
    border-radius: 999px;
    background: #2d3543;
    overflow: hidden;
  }
  .mobile-progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4d96ff, #2f6fc1);
  }
  .mobile-search input,
  .mobile-field input {
    width: 100%;
    box-sizing: border-box;
    border-radius: 14px;
    background: #1c1c1e;
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: #eef4fb;
    padding: 12px 14px;
  }
  .mobile-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
    color: #9dadc2;
  }
  .mobile-object-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 14px 10px;
    border-radius: 18px;
    background: #1c1c1e;
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #f4f7fb;
    text-align: center;
  }
  .mobile-object-card.active {
    border-color: rgba(112, 171, 255, 0.36);
    background: #1d2c3f;
  }
  .mobile-object-icon {
    width: 34px;
    height: 34px;
    background-repeat: no-repeat;
    background-size: contain;
    background-position: center;
  }
  .mobile-object-label {
    font-size: 12px;
    line-height: 1.35;
  }
  .mobile-settings-row.placeholder {
    color: #93a3bb;
  }
"#;
