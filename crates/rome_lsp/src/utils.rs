use std::collections::HashMap;

use crate::line_index::{LineCol, LineIndex};
use lspower::lsp::{self, CodeAction, CodeActionKind, Diagnostic, TextEdit, Url, WorkspaceEdit};
use rome_analyze::{Edit, Indel, TextAction};
use tracing::trace;

use rslint_parser::{TextRange, TextSize};

pub(crate) fn position(line_index: &LineIndex, offset: TextSize) -> lsp::Position {
	let line_col = line_index.line_col(offset);
	lsp::Position::new(line_col.line, line_col.col)
}

pub(crate) fn range(line_index: &LineIndex, range: TextRange) -> lsp::Range {
	let start = position(line_index, range.start());
	let end = position(line_index, range.end());
	lsp::Range::new(start, end)
}

pub(crate) fn offset(line_index: &LineIndex, position: lsp::Position) -> TextSize {
	let line_col = LineCol {
		line: position.line as u32,
		col: position.character as u32,
	};
	line_index.offset(line_col)
}

pub(crate) fn text_range(line_index: &LineIndex, range: lsp::Range) -> TextRange {
	let start = offset(line_index, range.start);
	let end = offset(line_index, range.end);
	TextRange::new(start, end)
}

pub(crate) fn text_edit(line_index: &LineIndex, indel: &Indel) -> TextEdit {
	let text_range = indel.range;
	let range = range(line_index, text_range);
	let new_text = indel.text.clone();
	TextEdit { range, new_text }
}

// Crashing when calling text_trimmed
#[allow(unused)]
pub(crate) fn broken_text_edit(line_index: &LineIndex, edit: &Edit) -> TextEdit {
	match edit {
		Edit::Remove { target } => {
			let range = range(line_index, target.trimmed_range());
			TextEdit {
				range,
				new_text: String::default(),
			}
		}
		Edit::Insert { offset, element } => {
			let range = range(line_index, TextRange::new(*offset, *offset));
			let new_text = match element.syntax_element() {
				rslint_parser::NodeOrToken::Node(it) => it.text_trimmed().into(),
				rslint_parser::NodeOrToken::Token(it) => it.text_trimmed().into(),
			};
			TextEdit { range, new_text }
		}
		Edit::Replace {
			target,
			replacement,
		} => {
			let range = range(line_index, target.trimmed_range());
			let new_text = match replacement.syntax_element() {
				rslint_parser::NodeOrToken::Node(it) => it.text_trimmed().into(),
				rslint_parser::NodeOrToken::Token(it) => it.text_trimmed().into(),
			};

			TextEdit { range, new_text }
		}
	}
}

pub(crate) fn text_action_to_lsp(
	action: &TextAction,
	line_index: &LineIndex,
	uri: Url,
	diagnostics: Option<Vec<Diagnostic>>,
) -> CodeAction {
	trace!("Action to LSP");
	let edits = action
		.edits
		.iter()
		.map(|r| text_edit(&line_index, r))
		.collect();

	let mut text_edits = HashMap::new();
	text_edits.insert(uri.clone(), edits);
	let changes = Some(text_edits);
	let edit = WorkspaceEdit {
		changes,
		document_changes: None,
		change_annotations: None,
	};

	CodeAction {
		title: action.title.clone(),
		kind: Some(CodeActionKind::QUICKFIX),
		diagnostics,
		edit: Some(edit),
		command: None,
		is_preferred: None,
		disabled: None,
		data: None,
	}
}
