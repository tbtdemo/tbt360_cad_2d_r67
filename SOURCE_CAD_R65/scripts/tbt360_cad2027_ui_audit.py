#!/usr/bin/env python3
"""Static release gate for the TBT360 CAD 2027 UI hardening surface.

This deliberately distinguishes three different claims:
  * registered: the engine has a CommandSpec
  * exposed: the Drafting Tool Sets contains a visible button id
  * interactive: commands that need pointer/text input have an Interactive machine

It cannot replace a Windows GUI acceptance run, but it prevents the old false-positive where a
button and a JSON-only command were counted as a usable mouse workflow.
"""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
engine_files = list((root / 'crates/engine/src').rglob('*.rs'))
engine = '\n'.join(p.read_text(errors='ignore') for p in engine_files)
palette = (root / 'crates/ui-egui/src/palettes.rs').read_text(errors='ignore')

required_ui = [
    'pedit', 'splinedit', 'arrayedit', 'arrayclose', 'lengthen', 'align', 'blend', 'reverse',
    'ncopy', 'mocoro', 'chspace', 'flatten', 'arraypath',
    'select', 'selectsimilar', 'lasso', 'isolateobjects', 'hideobjects', 'unisolateobjects',
    'xattach', '-xref', 'xbind', 'xopen', 'xclip', 'refedit', 'refset', 'refclose',
    'field', 'updatefield', 'spell', 'attedit', '-attedit', 'rename',
    'dimedit', 'tolerance', 'dimreassociate', 'dimdisassociate', 'dimtedit',
    'dimspace', 'dimbreak', 'diminspect', 'dimjogged', 'dimjogline', 'textedit',
    'attach', 'pdfattach', 'imageattach',
]

# These commands require a prompt/pointer workflow when started from a button. Commands omitted
# here are intentionally immediate (for example UNISOLATEOBJECTS) or can safely operate on the
# current selection/all matching objects (UPDATEFIELD/SPELL).
interactive_required = {
    'pedit', 'splinedit', 'arrayedit', 'arrayclose', 'lengthen', 'align', 'blend', 'reverse',
    'ncopy', 'mocoro', 'chspace', 'flatten', 'arraypath', 'select', 'selectsimilar', 'lasso',
    'isolateobjects', 'hideobjects', 'xattach', '-xref', 'xbind', 'xopen', 'xclip',
    'refedit', 'refset', 'refclose', 'field', 'attedit', '-attedit', 'rename', 'dimedit', 'tolerance', 'dimreassociate',
    'dimdisassociate', 'dimtedit', 'dimspace', 'dimbreak', 'diminspect', 'dimjogged',
    'dimjogline', 'textedit',
}

# R32 promotes ATTACH/PDFATTACH/IMAGEATTACH through the Underlay / Raster dialog. They do not
# need an engine Interactive machine because the dialog supplies validated command parameters.
engine_only = []


def spec_body(cmd: str) -> str:
    pat = re.compile(
        rf'CommandSpec::new\("{re.escape(cmd)}".*?(?=\n\s*CommandSpec::new\(|\n\s*\]|\n\s*\.into_iter\(\))',
        re.I | re.S,
    )
    m = pat.search(engine)
    return m.group(0) if m else ''


failed = []
print('TBT360 CAD 2027 UI / interaction static audit — R32')
for cmd in required_ui + engine_only:
    body = spec_body(cmd)
    registered = bool(body)
    exposed = bool(re.search(rf'\([^\n]*"{re.escape(cmd)}"\s*,', palette, re.I))
    need_ui = cmd in required_ui
    need_interactive = cmd in interactive_required
    interactive = '.interactive(' in body
    ok = registered and (exposed or not need_ui) and (interactive or not need_interactive)
    status = 'PASS' if ok else 'FAIL'
    print(
        f'{cmd:18} engine={str(registered):5} ui={str(exposed):5} '
        f'interactive={str(interactive):5} need_interactive={str(need_interactive):5} status={status}'
    )
    if not ok:
        failed.append(cmd)

print('\nBoundary: PASS proves source wiring + presence of an Interactive machine where required.')
print('It does NOT prove Windows rendering, pointer event delivery, Undo/Redo, Save/Reopen, or file-dialog integration.')
print('Dialog-routed attach commands: attach, pdfattach, imageattach')
if failed:
    print('FAILED:', ', '.join(failed))
sys.exit(1 if failed else 0)
