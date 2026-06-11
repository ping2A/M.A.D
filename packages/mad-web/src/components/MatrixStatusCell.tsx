import { useEffect, useRef, useState } from "react";
import type { ComplianceStatus } from "../types";
import {
  ComplianceStatusBadge,
  complianceStatusClass,
  StatusIcon,
} from "./ComplianceStatusBadge";

const ALL_STATUSES: ComplianceStatus[] = [
  "untested",
  "compliant",
  "partial",
  "non_compliant",
];

interface MatrixStatusCellProps {
  status: ComplianceStatus;
  notes: string | null;
  statusLabel: (s: ComplianceStatus) => string;
  cycleTitle: string;
  pickTitle: string;
  addNotesTitle: string;
  notesTitle: string;
  notesPlaceholder: string;
  saveLabel: string;
  onCycle: () => void;
  onSetStatus: (status: ComplianceStatus, notes: string | null) => void;
}

export function MatrixStatusCell({
  status,
  notes,
  statusLabel,
  cycleTitle,
  pickTitle,
  addNotesTitle,
  notesTitle,
  notesPlaceholder,
  saveLabel,
  onCycle,
  onSetStatus,
}: MatrixStatusCellProps) {
  const [pickerOpen, setPickerOpen] = useState(false);
  const [notesOpen, setNotesOpen] = useState(false);
  const [noteDraft, setNoteDraft] = useState(notes ?? "");
  const cellRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!pickerOpen && !notesOpen) return;
    const onDocClick = (e: MouseEvent) => {
      if (cellRef.current && !cellRef.current.contains(e.target as Node)) {
        setPickerOpen(false);
        setNotesOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [pickerOpen, notesOpen]);

  const openNotes = () => {
    setPickerOpen(false);
    setNoteDraft(notes ?? "");
    setNotesOpen(true);
  };

  const saveNotes = () => {
    onSetStatus(status, noteDraft.trim() || null);
    setNotesOpen(false);
  };

  const pickStatus = (next: ComplianceStatus) => {
    onSetStatus(next, notes);
    setPickerOpen(false);
  };

  return (
    <div className="matrix-status-cell" ref={cellRef}>
      <div className="matrix-status-main">
        <button
          type="button"
          className={complianceStatusClass(status, "matrix")}
          onClick={onCycle}
          title={`${statusLabel(status)} — ${cycleTitle}`}
          aria-label={`${statusLabel(status)} — ${cycleTitle}`}
        >
          <span className="compliance-status-icon">
            <StatusIcon status={status} size={18} />
          </span>
        </button>
        <div className="matrix-status-actions">
          <button
            type="button"
            className={`matrix-action-btn ${pickerOpen ? "active" : ""}`}
            onClick={() => {
              setNotesOpen(false);
              setPickerOpen((v) => !v);
            }}
            title={pickTitle}
            aria-label={pickTitle}
            aria-expanded={pickerOpen}
          >
            ▾
          </button>
          <button
            type="button"
            className={`matrix-action-btn ${notes ? "has-notes" : ""}`}
            onClick={() => (notesOpen ? setNotesOpen(false) : openNotes())}
            title={notes ? notesTitle.replace("{notes}", notes) : addNotesTitle}
            aria-label={notes ? notesTitle.replace("{notes}", notes) : addNotesTitle}
          >
            ✎
          </button>
        </div>
      </div>

      {pickerOpen && (
        <div className="matrix-status-menu" role="menu">
          {ALL_STATUSES.map((s) => (
            <button
              key={s}
              type="button"
              role="menuitemradio"
              aria-checked={status === s}
              className={`matrix-status-menu-item ${status === s ? "selected" : ""}`}
              onClick={() => pickStatus(s)}
            >
              <ComplianceStatusBadge status={s} label={statusLabel(s)} variant="menu" />
            </button>
          ))}
        </div>
      )}

      {notesOpen && (
        <div className="matrix-notes-popover">
          <textarea
            value={noteDraft}
            onChange={(e) => setNoteDraft(e.target.value)}
            rows={3}
            placeholder={notesPlaceholder}
            autoFocus
          />
          <div className="matrix-notes-actions">
            <button type="button" className="btn btn-primary btn-sm" onClick={saveNotes}>
              {saveLabel}
            </button>
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => setNotesOpen(false)}
            >
              ×
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
