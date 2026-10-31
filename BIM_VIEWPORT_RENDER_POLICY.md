# BIM Viewport Render Policy

- 1..4 model-space windows use the existing CAD native VPORTS engine.
- Exactly one viewport may be Edit.
- Reference viewports are read-only and use configurable opacity.
- Coordination viewport shows all permitted disciplines on one floor.
- Progress viewport is read-only and may color/annotate by completion state later.
- Render classification is metadata-driven; it must not mutate entity visibility.
- Primary objects render normally.
- Reference objects render with alpha.
- Objects outside Floor/Building/Zone scope are hidden from that viewport only.
- Legacy entities without BIM metadata remain visible as reference until migration assigns a scope.
- Selection/edit commands must operate only on the Edit viewport's primary handles.
