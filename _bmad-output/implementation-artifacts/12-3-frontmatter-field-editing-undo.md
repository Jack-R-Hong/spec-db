# Story 12.3: Frontmatter Field Editing & Undo

Status: ready-for-dev

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

- [ ] Implement edit mode in DetailPanel (AC: 1, 7)
  - [ ] Add `isEditing` state to DetailPanel component
  - [ ] Toggle to edit mode on: double-click panel body OR click "Edit" button
  - [ ] Edit mode shows form fields for: title (text input), tags (tag input with add/remove), owner (text input), depends_on (multi-select or editable list)
  - [ ] Escape in edit mode: discard changes, return to view mode
  - [ ] "Save" button: collect changed fields, trigger write-back flow
  - [ ] "Cancel" button: same as Escape
- [ ] Implement frontmatter edit write-back (AC: 2)
  - [ ] On Save + Confirm: send `POST /api/writeback` with frontmatter change payload
  - [ ] Payload: `{ type: "frontmatter_edit", spec_id, changes: { title?: "...", tags?: [...], owner?: "...", depends_on?: [...] } }`
  - [ ] Write-back pipeline: read file → update only changed frontmatter fields → preserve all other fields and body → write file → git commit → re-sync
  - [ ] Confirmation toast before commit (reuses ToastNotification from Story 12.1)
- [ ] Implement undo functionality (AC: 3, 4, 5, 6)
  - [ ] After successful write-back, show "Undo" button at bottom-center with 5-second countdown
  - [ ] Undo button auto-hides after 5 seconds (use `setTimeout`)
  - [ ] On undo click or Ctrl+Z: call `POST /api/writeback/undo`
  - [ ] Undo endpoint: `git revert HEAD --no-edit` on the last write-back commit → re-sync → return updated graph
  - [ ] Verify round-trip < 2 seconds (NFR36)
  - [ ] Clear `UndoState` in AppState after undo completes or after 5-second window expires
- [ ] Implement `POST /api/writeback/undo` endpoint (AC: 3, 4)
  - [ ] Check `UndoState` exists and is within 5-second window
  - [ ] If expired: return `{ error_type: "expired", message: "Undo window has expired" }`
  - [ ] If valid: execute `git revert` on the stored commit SHA
  - [ ] Trigger re-sync after revert
  - [ ] Clear `UndoState`
  - [ ] Return success with updated graph state
- [ ] Register Ctrl+Z keyboard shortcut (AC: 6)
  - [ ] Only active when undo window is open (UndoState exists and < 5 seconds old)
  - [ ] Do not intercept Ctrl+Z when editing text fields (check active element)
- [ ] Add tests (AC: 1-7)
  - [ ] Component test: double-click panel switches to edit mode
  - [ ] Component test: edit mode shows editable fields
  - [ ] Component test: Escape discards changes
  - [ ] Component test: Save triggers confirmation toast
  - [ ] Component test: Undo button appears after write-back
  - [ ] Component test: Undo button disappears after 5 seconds
  - [ ] Component test: Ctrl+Z triggers undo
  - [ ] Unit test: undo endpoint reverts git commit
  - [ ] Unit test: expired undo returns error
  - [ ] Integration test: edit → save → undo round-trip

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

### Debug Log References

### Completion Notes List

### Change Log

### File List
