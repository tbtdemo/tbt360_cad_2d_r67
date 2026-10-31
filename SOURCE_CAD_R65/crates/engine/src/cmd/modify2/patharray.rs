use super::*;

use cadcraft_doc::{ArrayKind, ArrayParams, PathArray};

// =====================================================================================
// ARRAYPATH (non-associative copies along a curve)
// =====================================================================================

pub(crate) fn arraypath(
    s: &mut Session,
    hs: &[Handle],
    path: Handle,
    count: Option<usize>,
    spacing: Option<f64>,
    align: bool,
) -> Result<Vec<Handle>> {
    let pk = curves::entity(s, path)?.kind;
    let c = Chain::of(&pk).ok_or_else(|| other("Select a line, arc, circle, polyline, ellipse or spline as the path."))?;
    let l = c.len();
    if l <= 1e-12 {
        return Err(other("The path has no length."));
    }
    let objs: Vec<Handle> = hs.iter().copied().filter(|h| *h != path).collect();
    if objs.is_empty() {
        return Err(other("Select objects to array."));
    }
    let (n, step) = match (count, spacing) {
        (_, Some(sp)) if sp.is_finite() && sp > 0.0 => (((l / sp).floor() as usize).saturating_add(1), sp),
        (Some(n), _) => {
            let n = n.max(1);
            (n, if c.closed { l / n as f64 } else { l / (n.max(2) - 1) as f64 })
        }
        _ => (6, if c.closed { l / 6.0 } else { l / 5.0 }),
    };
    crate::cmd::array::check_size(s, n as u64, objs.len())?;
    let Some((p0, t0)) = c.at_length(0.0) else { return Ok(Vec::new()) };
    let mut out = Vec::new();
    for i in 1..n {
        let Some((p, t)) = c.at_length(step * i as f64) else { continue };
        let mut m = Mat3::translate(p - p0);
        if align {
            m = Mat3::rotate_about(p, t.angle() - t0.angle()).then_before(m);
        }
        out.extend(crate::cmd::modify::transform_entities(s, &objs, &m, true)?);
    }
    Ok(out)
}

/// Create an associative path array. The path handle and sampled local placements are stored in
/// the array block, so the array remains a real associative model object rather than a set of
/// loose copies. `ARRAYPATH` refresh can later resample the same path handle.
pub(crate) fn arraypath_associative(
    s: &mut Session,
    hs: &[Handle],
    path: Handle,
    count: Option<usize>,
    spacing: Option<f64>,
    align: bool,
) -> Result<Value> {
    let pk = curves::entity(s, path)?.kind;
    let c = Chain::of(&pk).ok_or_else(|| other("Select a line, arc, circle, polyline, ellipse or spline as the path."))?;
    let l = c.len();
    if l <= 1e-12 {
        return Err(other("The path has no length."));
    }
    let objs: Vec<Handle> = hs.iter().copied().filter(|h| *h != path).collect();
    if objs.is_empty() {
        return Err(other("Select objects to array."));
    }
    let (n, step) = match (count, spacing) {
        (_, Some(sp)) if sp.is_finite() && sp > 0.0 => (((l / sp).floor() as usize).saturating_add(1), sp),
        (Some(n), _) => {
            let n = n.max(1);
            (n, if c.closed { l / n as f64 } else { l / (n.max(2) - 1) as f64 })
        }
        _ => (6, if c.closed { l / 6.0 } else { l / 5.0 }),
    };
    crate::cmd::array::check_size(s, n as u64, objs.len())?;
    let Some((p0, t0)) = c.at_length(0.0) else { return Err(other("The path has no start point.")) };
    let mut points = Vec::with_capacity(n);
    let mut rotations = Vec::with_capacity(n);
    for i in 0..n {
        let Some((p, t)) = c.at_length(step * i as f64) else { continue };
        points.push(p - p0);
        rotations.push(if align { t.angle() - t0.angle() } else { 0.0 });
    }
    if points.is_empty() {
        return Err(other("The path produced no array items."));
    }
    let base = curves::entity(s, objs[0])?.kind.grips().first().copied().unwrap_or(Vec2::ZERO);
    let params = ArrayParams {
        source: "*".into(),
        kind: ArrayKind::Path(PathArray { path, points, rotations, align }),
        levels: 1,
        level_spacing: 0.0,
        replaced: Vec::new(),
        suppressed: Vec::new(),
        editing: None,
    };
    let h = crate::cmd::array::make_assoc(s, &objs, base, params)?;
    Ok(json!({ "handle": h.hex(), "path": path.hex(), "created": n.saturating_sub(1).saturating_mul(objs.len()), "associative": true }))
}

fn sampled_path(c: &Chain, count: usize, align: bool) -> Result<(Vec<Vec2>, Vec<f64>)> {
    let n = count.max(1);
    let step = if c.closed { c.len() / n as f64 } else { c.len() / (n.max(2) - 1) as f64 };
    let Some((p0, t0)) = c.at_length(0.0) else { return Err(other("The path has no start point.")) };
    let mut points = Vec::with_capacity(n);
    let mut rotations = Vec::with_capacity(n);
    for i in 0..n {
        let Some((p, t)) = c.at_length(step * i as f64) else { continue };
        points.push(p - p0);
        rotations.push(if align { t.angle() - t0.angle() } else { 0.0 });
    }
    if points.is_empty() {
        return Err(other("The path produced no array items."));
    }
    Ok((points, rotations))
}

pub(crate) fn run_arraypath_refresh(s: &mut Session, p: &Value) -> Result<Value> {
    let h = curves::handle_param(p, "handle").ok_or_else(|| bad("arraypathrefresh", "`handle` is required"))?;
    let d = s.doc()?;
    let e = d.entity(h).ok_or_else(|| bad("arraypathrefresh", "no such object"))?;
    let params = d.array_params(e).ok_or_else(|| bad("arraypathrefresh", "the object is not an associative array"))?;
    let ArrayKind::Path(path_array) = params.kind.clone() else {
        return Err(bad("arraypathrefresh", "the object is not an associative path array"));
    };
    let path_entity = d.entity(path_array.path).ok_or_else(|| bad("arraypathrefresh", "the source path no longer exists"))?;
    let chain = Chain::of(&path_entity.kind).ok_or_else(|| bad("arraypathrefresh", "the source path is no longer a supported curve"))?;
    if chain.len() <= 1e-12 {
        return Err(other("The source path has no length."));
    }
    let (points, rotations) = sampled_path(&chain, path_array.points.len(), path_array.align)?;
    let mut refreshed = params;
    refreshed.kind = ArrayKind::Path(PathArray { path: path_array.path, points, rotations, align: path_array.align });
    crate::cmd::array::rebuild(s,h,refreshed,None)?;
    s.doc_mut()?.modify_entity(h,|e|crate::cmd::modify::bump_tbt360_bim_revision(&mut e.common,h))?;
    Ok(json!({ "handle": h.hex(), "path": path_array.path.hex(), "itemCount": path_array.points.len(), "refreshed": true }))
}


pub(crate) fn run_arraypath_auto(s: &mut Session, p: &Value) -> Result<Value> {
    let current = s.doc()?.header.i64("ARRAYPATHAUTO", 0) != 0;
    let on = p.get("on").and_then(Value::as_bool).unwrap_or(!current);
    s.doc_mut()?.header.set_i64("ARRAYPATHAUTO", if on { 1 } else { 0 });
    s.touch();
    Ok(json!({"enabled": on, "sysvar": "ARRAYPATHAUTO", "dependencyMode": "post-command associative path refresh"}))
}

pub(crate) fn run_arraypath_refresh_all(s: &mut Session, _p: &Value) -> Result<Value> {
    let handles: Vec<Handle> = {
        let d=s.doc()?;
        d.space(&s.state()?.edit_space()).map(|st| st.iter().filter_map(|e| d.array_params(e).and_then(|a| matches!(a.kind,ArrayKind::Path(_)).then_some(e.handle))).collect()).unwrap_or_default()
    };
    let mut refreshed=Vec::new(); let mut failed=Vec::new();
    for h in handles {
        match run_arraypath_refresh(s,&json!({"handle":h.hex()})) {Ok(_)=>refreshed.push(h.hex()),Err(e)=>failed.push(json!({"handle":h.hex(),"error":e.to_string()}))}
    }
    Ok(json!({"refreshed":refreshed.len(),"handles":refreshed,"failed":failed,"dependencyMode":"batch refresh of all associative path arrays in current space"}))
}

pub(super) fn run_arraypath(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let path = curves::handle_param(p, "path").ok_or_else(|| bad("arraypath", "`path` (handle) is required"))?;
    let count = p.get("count").and_then(Value::as_u64).map(|n| usize::try_from(n).unwrap_or(usize::MAX));
    let spacing = p.get("spacing").and_then(Value::as_f64);
    if bool_or(p, "associative", false) {
        return arraypath_associative(s, &hs, path, count, spacing, bool_or(p, "align", true));
    }
    let r = arraypath(s, &hs, path, count, spacing, bool_or(p, "align", true))?;
    Ok(json!({ "handles": r.iter().map(|h| h.hex()).collect::<Vec<_>>() }))
}

#[derive(Default)]
pub(in crate::cmd) struct PathArrayM {
    sel: SelectPhase,
    objs: Vec<Handle>,
    path: Option<Handle>,
}

impl PathArrayM {
    /// Start at the path prompt with the objects already selected (ARRAY > PAth).
    pub(in crate::cmd) fn with_objects(objs: Vec<Handle>) -> Self {
        PathArrayM { sel: SelectPhase { picked: objs.clone(), done: true, ..SelectPhase::default() }, objs, path: None }
    }
}

impl Interactive for PathArrayM {
    fn name(&self) -> &'static str {
        "ARRAYPATH"
    }
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        self.sel = SelectPhase::begin(s);
        if self.sel.done {
            self.objs = self.sel.picked.clone();
        }
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if !self.sel.done {
            return self.sel.prompt();
        }
        match self.path {
            None => Prompt::new("Select path curve", Accept::POINT),
            Some(_) => Prompt::new("Enter number of items along path", Accept::NUMBER).default("6"),
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        if !self.sel.done {
            return match self.sel.feed(s, &i)? {
                SelOutcome::More => Ok(Step::Continue),
                SelOutcome::Empty => Ok(Step::Cancel),
                SelOutcome::Done(hs) => {
                    self.objs = hs;
                    Ok(Step::Continue)
                }
            };
        }
        match (self.path, i) {
            (None, Input::Point(p)) => {
                let h = curves::pick_at(s, p).ok_or_else(|| other("*Invalid selection*"))?;
                Chain::of(&curves::entity(s, h)?.kind).ok_or_else(|| other("Select a curve as the path."))?;
                self.path = Some(h);
                Ok(Step::Continue)
            }
            (Some(path), inp @ (Input::Text(_) | Input::Enter)) => {
                let n = match &inp {
                    Input::Text(t) => number(t).filter(|n| *n >= 1.0).ok_or_else(|| other("Requires a count of 1 or more."))? as usize,
                    _ => 6,
                };
                arraypath(s, &self.objs, path, Some(n), None, true)?;
                s.set_selection(Vec::new());
                s.echo(format!("Type = Path  Associative = No ({n} items)"));
                Ok(Step::Done)
            }
            (None, Input::Enter) => Ok(Step::Cancel),
            _ => Ok(Step::Continue),
        }
    }
}
