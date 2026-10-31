use super::*;

// =====================================================================================
// TEXTEDIT
// =====================================================================================

fn text_of(k: &EntityKind) -> Option<String> {
    match k {
        EntityKind::Text(t) => Some(t.value.clone()),
        EntityKind::AttDef(a) => Some(a.text.value.clone()),
        EntityKind::MText(t) => Some(t.contents.replace("\\P", "\n")),
        EntityKind::Dimension(d) => Some(if d.text.is_empty() { "<>".into() } else { d.text.clone() }),
        _ => None,
    }
}

pub(crate) fn set_text(s: &mut Session, h: Handle, text: &str) -> Result<()> {
    if curves::is_locked(s, h) {
        return Err(other("The object is on a locked layer."));
    }
    let mut k = curves::entity(s, h)?.kind;
    match &mut k {
        EntityKind::Text(t) => t.value = text.to_string(),
        EntityKind::AttDef(a) => a.text.value = text.to_string(),
        EntityKind::MText(t) => t.contents = text.replace('\n', "\\P"),
        EntityKind::Dimension(d) => {
            d.text = if text == "<>" { String::new() } else { text.to_string() };
            d.block = None;
        }
        _ => return Err(other("Select a text, mtext, attribute definition or dimension.")),
    }
    set_kind(s,h,k)?;
    s.doc_mut()?.modify_entity(h,|e|crate::cmd::modify::bump_tbt360_bim_revision(&mut e.common,h))?;
    Ok(())
}

pub(super) fn run_textedit(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    if hs.is_empty() { return Err(bad("textedit", "`handle` is required")); }
    let text = str_param(p, "text").ok_or_else(|| bad("textedit", "`text` is required"))?;
    if text.len() > 1_000_000 {
        return Err(bad("textedit", "text is too long"));
    }
    for h in &hs { set_text(s, *h, text)?; }
    Ok(json!({ "handle": hs[0].hex(), "handles": hs.iter().map(|h| h.hex()).collect::<Vec<_>>(), "changed": hs.len() }))
}

#[derive(Default)]
pub(super) struct TextEditM {
    h: Option<(Handle, String)>,
    /// Edits made in this run, newest last: each object as it was before (the Undo option).
    edits: Vec<(Handle, EntityKind, cadcraft_doc::Common)>,
    /// Single mode: end after one edit (the Mode option; Multiple by default).
    single: bool,
    /// Asking for the edit mode.
    mode: bool,
}

impl Interactive for TextEditM {
    fn name(&self) -> &'static str {
        "TEXTEDIT"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.mode {
            let current = if self.single { "Single" } else { "Multiple" };
            return Prompt::new("Enter a text edit mode option", curves::KW).kw(&["Single", "Multiple"]).default(current);
        }
        match &self.h {
            None => Prompt::new("Select an annotation object", Accept::POINT).kw(&["Undo", "Mode"]),
            Some((_, old)) => Prompt::new("Enter new text", Accept::TEXT).default(old.chars().take(40).collect::<String>()),
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        if self.mode {
            match i {
                Input::Keyword(k) if k == "Single" => self.single = true,
                Input::Keyword(k) if k == "Multiple" => self.single = false,
                Input::Enter => {}
                _ => return Err(other("Enter Single or Multiple.")),
            }
            self.mode = false;
            return Ok(Step::Continue);
        }
        match (&self.h, i) {
            (None, Input::Point(p)) => {
                let h = curves::pick_at(s, p).ok_or_else(|| other("*Invalid selection*"))?;
                let old = text_of(&curves::entity(s, h)?.kind).ok_or_else(|| other("Select a text, mtext, attribute definition or dimension."))?;
                self.h = Some((h, old));
                Ok(Step::Continue)
            }
            (Some((h, _)), Input::Text(t) | Input::Keyword(t)) => {
                let h = *h;
                let before_entity=curves::entity(s,h)?;
                let before=before_entity.kind.clone();
                let common=before_entity.common.clone();
                set_text(s,h,&t)?;
                self.edits.push((h,before,common));
                self.h = None;
                Ok(if self.single { Step::Done } else { Step::Continue })
            }
            (Some(_), Input::Enter) => {
                self.h = None;
                Ok(if self.single { Step::Done } else { Step::Continue })
            }
            (None, Input::Enter) => Ok(Step::Done),
            (None, Input::Keyword(k)) if k == "Undo" => {
                match self.edits.pop() {
                    Some((h,before,common)) => {
                        set_kind(s,h,before)?;
                        s.doc_mut()?.modify_entity(h,|e|e.common=common)?;
                    },
                    None => s.echo("Everything has been undone"),
                }
                Ok(Step::Continue)
            }
            (None, Input::Keyword(k)) if k == "Mode" => {
                self.mode = true;
                Ok(Step::Continue)
            }
            _ => Ok(Step::Continue),
        }
    }
}
