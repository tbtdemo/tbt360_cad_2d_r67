# Underlays and attached images

CADCraft keeps attachment placement and clipping metadata in the drawing even when the
engine has no decoder for the external binary format. This means `EXPORT` to DXF/DWG can
round-trip the record without silently deleting it; display is reported as `rendered: false`
until a host renderer is connected.

Commands:

- `IMAGEATTACH` (`IMA`) / `IMAGECLIP` (`ICLIP`) for PNG, JPG/JPEG, BMP and TIF/TIFF.
- `PDFATTACH` (`PDFA`) / `PDFCLIP` (`PDFC`) for a PDF page.
- `ATTACH` dispatches by extension and records DWF/DWFx, DGN, WMF and EPS underlays.
- `UNDERLAYCLIP` (`DWFCLIP`, `DGNCLIP`, `WMFCLIP`, `EPSCLIP`, `UCLIP`) applies or removes
  a polygon clip to those vector underlays.
- `UNDERLAYINFO` (`UINFO`) returns the stable path, PDF page, placement and clip metadata.

The current engine intentionally does not decode PDF, DWF/DWFx, DGN, WMF or EPS bytes and
does not claim `PDFIMPORT`. A host can add a renderer/importer later while consuming the
same metadata contract. Clip vertices are stored as parallel DXF X/Y tags, preserving
repeated coordinates in rectangular clips.

## R32 viewport workflow

R32 adds an **Underlay / Raster Attach** dialog in Tool Sets. PDF, DWF/DWFx, DGN, WMF and EPS metadata underlays render a stable placement frame (or clip polygon when present), so placement is visible even though page/vector contents are not decoded. Raster `IMAGE` entities keep their existing frame renderer. This adds no file-picker or decoder dependency; users enter a local path explicitly.
