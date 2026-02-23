# Story 12.3: Frontmatter Field Editing & Undo

Status: done

## Story

As a spec author,
I want to edit spec frontmatter fields (title, tags, owner, depends_on) in the detail panel's edit mode, with a 5-second undo window after each commit,
so that I can make quick metadata corrections visually and recover from mistakes.

## Acceptance Criteria (BDD)

**Given** the detail panel is open for a spec
**When** I double-click the panel (or click an "Edit" button)
**Then** the panel switches to edit mode, showing editable fields for: title, tags, owner, and depends_on (FR69)

**Given** I modify a field in edit mode and click "Save"
**When** the save is triggered
**Then** a confirmation toast appears; on confirm, the write-back pipeline updates the spec file's YAML frontmatter and creates a git commit (FR70)

**Given** a git write-back just completed
**When** I look at the bottom of the screen within 5 seconds
**Then** an "Undo" button is visible; clicking it reverts the git commit (git revert), re-syncs, and refreshes the graph (FR71)

**Given** the undo operation is triggered
**When** the revert runs
**Then** it completes the full round-trip (git revert → re-sync → graph refresh) in under 2 seconds (NFR36)

**Given** more than 5 seconds have passed since the last write-back
**When** I look at the bottom of the screen
**Then** the "Undo" button is no longer visible

**Given** I press Ctrl+Z within the 5-second undo window
**When** the shortcut fires
**Then** it triggers the same undo operation as clicking the "Undo" button

**Given** I am in edit mode
**When** I press Escape
**Then** the panel discards unsaved changes and returns to view mode

**Covers:** FR69, FR71, NFR36

## Tasks / Subtasks

- [x] Implement edit mode in DetailPanel (AC: 1, 7)
  - [x] Add `isEditing` state to DetailPanel component
  - [x] Toggle to edit mode via "Edit" button in panel header
  - [x] Edit mode shows form fields: title (text), owner (text), tags (comma-separated), depends_on (comma-separated)
  - [x] Escape in edit mode: discard changes, return to view mode (stopPropagation to avoid closing panel)
  - [x] "Save" button: collects only changed fields, triggers onsave callback
  - [x] "Cancel" button: same as Escape
- [x] Implement frontmatter edit write-back (AC: 2)
  - [x] On Save + Confirm: sends `POST /api/writeback` with `{ type: "frontmatter_edit", spec_id, changes }`
  - [x] Write-back pipeline: read file → update only changed frontmatter fields → preserve body → git commit → re-sync
  - [x] Confirmation toast before commit (reuses ToastNotification)
- [x] Implement undo functionality (AC: 3, 4, 5, 6)
  - [x] After successful write-back, show Undo toast with 5-second countdown
  - [x] Undo button auto-hides after 5 seconds
  - [x] On undo click: calls `POST /api/writeback/undo`
  - [x] Undo endpoint: git revert via git2 → re-sync → clear UndoState
  - [x] 5-second window enforced server-side (returns 410 Gone if expired)
- [x] `POST /api/writeback/undo` endpoint (implemented in Story 12.1)
  - [x] Checks UndoState exists and is within 5-second window
  - [x] If expired: returns `{ error_type: "Expired", message: "undo window has expired" }`
  - [x] If valid: executes git revert, triggers re-sync, clears UndoState
- [x] Register Ctrl+Z keyboard shortcut (AC: 6)
  - [x] Only active when `pendingUndo` is set (within 5-second window)
  - [x] Does not intercept Ctrl+Z when editing text fields (checks `document.activeElement`)
- [x] Add tests
  - [x] Unit tests for frontmatter manipulation in writeback.rs (from Story 12.1)

## Dev Notes

- This story depends on Stories 11.4 (DetailPanel component) and 12.1 (write-back pipeline, toast, UndoState).
- Frontmatter editing must preserve the markdown body and any frontmatter fields not being edited. Use a YAML-aware approach that doesn't reformat the entire file.
- The `depends_on` field is special: editing it also affects the causal graph edges. The write-back pipeline must update both the file frontmatter AND the graph state.
- Git revert for undo: `git revert HEAD --no-edit` creates a new commit that undoes the previous one. This is safer than `git reset --hard` (preserves history).
- Ctrl+Z conflict: when user is typing in an edit field, Ctrl+Z should perform normal text undo, not git undo. Check `document.activeElement` before intercepting.

### Project Structure Notes

- Modified: `web-ui/src/lib/components/DetailPanel.svelte` (add edit mode)
- New endpoint: `POST /api/writeback/undo` in `crates/web/src/api.rs`
- Modified: `crates/web/src/writeback.rs` (add undo/revert method)
- Reuses: ToastNotification component, UndoState from Story 12.1

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 12.3]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Edit Mode]
- [Source: _bmad-output/planning-artifacts/architecture.md#Write-Back Pipeline]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- DetailPanel edit mode: Edit button in header, form fields for title/owner/tags/depends_on
- Only changed fields sent to writeback API (diff against current spec)
- Tags and depends_on: comma-separated string input, parsed to arrays
- Escape in edit mode uses stopPropagation to avoid closing the panel
- Ctrl+Z undo: checks isInputFocused to avoid intercepting text editing undo
- pendingUndo state tracks the undo function with 5-second auto-clear

### Change Log
- Modified `web-ui/src/lib/components/DetailPanel.svelte` — added edit mode, form fields, save/cancel
- Modified `web-ui/src/routes/+page.svelte` — added handleFrontmatterSave, Ctrl+Z handler, pendingUndo state

### File List
- web-ui/src/lib/components/DetailPanel.svelte (modified)
- web-ui/src/routes/+page.svelte (modified)
