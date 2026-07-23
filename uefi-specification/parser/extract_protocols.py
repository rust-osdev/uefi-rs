#!/usr/bin/env python3
"""Extract UEFI protocol information from the UEFI Specification RST files.

Downloads both the UEFI and PI spec sources from GitHub, parses every RST
file with docutils (not regex), and writes canonical name / GUID / summary
for every protocol definition found.
"""

import os
import re
import tempfile
import urllib.request
import zipfile

from docutils.core import publish_doctree
from docutils.nodes import section, title, subtitle, Text, paragraph, target

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

UEFI_SPEC_URL = "https://github.com/UEFI/UEFI-Specification-Release/archive/refs/heads/main.zip"
PI_SPEC_URL = "https://github.com/UEFI/PI-Specification-Release/archive/refs/heads/main.zip"
OUTPUT_FILE = "uefi_protocols.txt"

# Global reference map:  :ref:`name` -> section title
REF_MAP: dict[str, str] = {}

# Section titles that happen to contain "PROTOCOL" but are not protocol defs
SKIP_KEYWORDS = (
    "INTRODUCTION", "OVERVIEW", "REVISION HISTORY", "COPYRIGHT",
    "TABLE OF CONTENTS", "ACKNOWLEDGMENTS", "REFERENCES",
    "REVISION", "CHANGE HISTORY", "APPENDIX", "LIST OF",
    "PROTOCOL DESCRIPTIONS", "PROTOCOL HANDLER",
    "BOOT VIA THE",
)

# RST field markers that should never be treated as summary text
FIELD_MARKERS = {
    "GUID", "Summary", "Description", "Protocol Interface Structure",
    "Parameters", "See Also", "Revision Number",
}

# ---------------------------------------------------------------------------
# Spec download / extraction
# ---------------------------------------------------------------------------


def download_and_extract(url: str, temp_dir: str, output_dir_name: str) -> str:
    zip_path = os.path.join(temp_dir, f"{output_dir_name}.zip")
    print(f"Downloading {url} ...")
    urllib.request.urlretrieve(url, zip_path)
    print("Download complete.")
    dest = os.path.join(temp_dir, output_dir_name)
    os.makedirs(dest, exist_ok=True)
    with zipfile.ZipFile(zip_path) as zf:
        zf.extractall(dest)
    entries = [e for e in os.listdir(dest) if os.path.isdir(os.path.join(dest, e))]
    if len(entries) == 1:
        return os.path.join(dest, entries[0])
    raise RuntimeError(f"Unexpected archive structure: {entries}")


# ---------------------------------------------------------------------------
# GUID / name helpers
# ---------------------------------------------------------------------------


def parse_guid_and_name_from_code_block(text: str) -> tuple[str | None, str | None]:
    """Parse GUID and canonical protocol name from a C-style #define code block.

    Returns ``(guid_string, name)`` where *name* is the ``#define`` identifier
    with ``_GUID`` stripped.  The GUID is returned lowercase.
    """
    define_match = re.search(r"#define\s+(\w+)_GUID\b", text)
    name = define_match.group(1) if define_match else None

    hex_vals = []
    for line in text.splitlines():
        hex_vals.extend(re.findall(r"0x\s*([0-9a-fA-F]+)", line))
    if len(hex_vals) < 3:
        return None, name

    first = hex_vals[0].zfill(8)[:8]
    second = hex_vals[1].zfill(4)[:4]
    third = hex_vals[2].zfill(4)[:4]
    rest = "".join(v.zfill(2) for v in hex_vals[3:])

    if len(rest) < 16:
        return None, name
    rest = rest[:16]

    guid = f"{first}-{second}-{third}-{rest[0:4]}-{rest[4:16]}".lower()
    return guid, name


# ---------------------------------------------------------------------------
# RST text extraction (with :ref: resolution)
# ---------------------------------------------------------------------------


def extract_text(node) -> str:
    """Recursively extract all text from a node, resolving ``:ref:`` references."""
    texts = []
    for child in node:
        if isinstance(child, Text):
            texts.append(child.astext())
        elif child.tagname == "problematic":
            raw = child.astext()
            ref_match = re.match(r":ref:\s*`([^`]+)`", raw)
            if ref_match:
                texts.append(REF_MAP.get(ref_match.group(1), ref_match.group(1)))
            else:
                texts.append(raw)
        else:
            texts.append(extract_text(child))
    return "".join(texts)


# ---------------------------------------------------------------------------
# Protocol name detection
# ---------------------------------------------------------------------------


def is_protocol_name(title_text: str) -> bool:
    """Check if a title looks like a protocol definition (not a function/method)."""
    upper = title_text.upper()
    if "PROTOCOL" not in upper:
        return False
    if any(kw in upper for kw in SKIP_KEYWORDS):
        return False
    if "." in title_text or "-" in title_text:
        return False
    return True


# ---------------------------------------------------------------------------
# Body scanning — Summary / GUID extraction
# ---------------------------------------------------------------------------


def _is_bold_text(node, expected_text: str) -> bool:
    """Return True if *node* is a paragraph containing only bold *expected_text*."""
    if not isinstance(node, paragraph):
        return False
    return extract_text(node).strip() == expected_text


def _parse_node_list(nodes_iter) -> dict | None:
    """Scan a flat list of sibling / child nodes for Summary and GUID markers.

    Returns ``{"summary": ..., "guid": ..., "define_name": ...}`` or ``None``.

    - Collects multi-line summaries by gathering all paragraphs between
      ``**Summary**`` and ``**GUID**``.
    - Falls back to paragraphs before the GUID when no ``**Summary**`` marker
      exists (``**Description**`` text, etc.).
    - Recurses into child ``section`` nodes whose title contains ``.``
      (function sub-sections like ``EFI_XXX.FunctionName()``) so that
      protocols whose body lives in the first function sub-section are still
      found.  Sub-sections without ``.`` are not entered — this prevents
      picking up GUIDs from sibling protocol definitions nested under a
      parent ``----`` heading.
    """
    summary_parts: list[str] = []
    pre_summary_parts: list[str] = []
    guid = None
    define_name = None
    in_summary = False
    has_summary_marker = False

    children = list(nodes_iter)
    for i, child in enumerate(children):
        if guid:
            break

        # -- Summary / GUID field markers -----------------------------------
        if _is_bold_text(child, "Summary"):
            in_summary = True
            has_summary_marker = True
            summary_parts = []
            continue

        if _is_bold_text(child, "GUID") or _is_bold_text(child, "Protocol GUID"):
            in_summary = False
            if i + 1 < len(children):
                text = children[i + 1].astext()
                guid, define_name = parse_guid_and_name_from_code_block(text)
            continue

        # -- In summary section ---------------------------------------------
        if in_summary:
            if isinstance(child, paragraph):
                text = extract_text(child).strip()
                # Stop collecting at the next field marker
                if text in ("GUID", "Protocol GUID", "Description",
                            "Protocol Interface Structure", "Parameters", "See Also"):
                    in_summary = False
                else:
                    summary_parts.append(text)
            continue

        # -- Before any Summary marker: collect pre-GUID paragraphs ----------
        if not has_summary_marker and isinstance(child, paragraph):
            text = extract_text(child).strip()
            if text and text not in FIELD_MARKERS and text != "Protocol GUID":
                pre_summary_parts.append(text)

        # -- Function sub-section recursion ----------------------------------
        if isinstance(child, section):
            sec_title = next(
                (c.astext() for c in child.children if isinstance(c, title)), None
            )
            if sec_title and "." in sec_title:
                result = _parse_node_list(list(child))
                if result:
                    guid = result["guid"]
                    define_name = result["define_name"]
                    if not summary_parts:
                        summary_parts = [result["summary"]]
                        has_summary_marker = True

    if not guid:
        return None

    effective_parts = summary_parts if summary_parts else pre_summary_parts
    summary = re.sub(r"\s+", " ", " ".join(effective_parts)).strip()

    return {"summary": summary, "guid": guid, "define_name": define_name}


def parse_node_for_protocol(node) -> dict | None:
    """Convenience wrapper — identical to ``_parse_node_list(list(node))``."""
    return _parse_node_list(list(node))


# ---------------------------------------------------------------------------
# Canonical name derivation
# ---------------------------------------------------------------------------


def _canonical_name(define_name: str | None, heading: str) -> str:
    """Derive a canonical protocol name from the GUID #define and section heading.

    Rules (in order):
      1. If the heading already starts with ``EFI_`` and ends with ``_PROTOCOL``
         the heading is authoritative — some defines use an abbreviated name
         (e.g. define ``EFI_KMS_PROTOCOL_GUID`` for heading
         ``EFI_KEY_MANAGEMENT_SERVICE_PROTOCOL``).
      2. Otherwise use the define name (the GUID ``#define`` gives the canonical
         C identifier).  This handles human-readable headings like
         ``Partition Information Protocol`` → ``EFI_PARTITION_INFO_PROTOCOL``.
      3. Final fallback: the normalized heading as-is.
    """
    heading = heading.removesuffix(" Summary")
    heading_norm = re.sub(r"\s+", "_", heading.upper())

    if heading_norm.startswith("EFI_") and heading_norm.endswith("_PROTOCOL"):
        return heading_norm

    if define_name:
        define_name = define_name.upper()
        if define_name.endswith("_PROTOCOL"):
            return define_name

    return heading_norm


# ---------------------------------------------------------------------------
# RST file parsing
# ---------------------------------------------------------------------------


def _matching_child_section(container, parent_base: str):
    """Yield child ``section`` nodes whose base name matches *parent_base*.

    The base name is the ``_PROTOCOL``-stripped title without ``EFI_`` prefix
    and without trailing version digits.  This allows a parent ``----`` section
    (e.g. ``EFI Driver Diagnostics Protocol``) to borrow body content from a
    child ``####`` section whose name differs only by a version suffix
    (e.g. ``EFI_DRIVER_DIAGNOSTICS2_PROTOCOL``).
    """
    for child in container.children:
        if not isinstance(child, section):
            continue
        ct = next(
            (c.astext().strip() for c in child.children if isinstance(c, title)), None
        )
        if not ct:
            continue
        child_norm = re.sub(r"\s+", "_", ct.upper())
        child_base = child_norm.removeprefix("EFI_") if child_norm.startswith("EFI_") else child_norm
        child_base = child_base.removesuffix("_PROTOCOL")
        child_base = re.sub(r"\d+$", "", child_base)
        if child_base == parent_base:
            yield child


def parse_rst_file(filepath: str) -> list[dict]:
    """Parse a single RST file and extract protocol definitions."""
    try:
        with open(filepath, "r", encoding="utf-8", errors="replace") as f:
            content = f.read()
    except Exception:
        return []

    try:
        doctree = publish_doctree(content)
    except Exception:
        return []

    protocols = []
    for node in doctree.findall():
        # -- Identify candidate headings ------------------------------------
        if isinstance(node, section):
            title_node = next((c for c in node.children if isinstance(c, title)), None)
            title_text = title_node.astext().strip() if title_node else None
            parse_input = node
        elif isinstance(node, subtitle):
            title_text = extract_text(node).strip()
            # For subtitle nodes the body lives in following siblings, not in
            # the subtitle's own children.
            parent = node.parent
            children = list(parent.children)
            idx = children.index(node)
            parse_input = children[idx + 1:]
        else:
            continue

        if not title_text or not is_protocol_name(title_text):
            continue

        # -- Fix common heading imperfections -------------------------------
        title_text = title_text.removesuffix(" Summary")
        title_text = re.sub(r"\s+(PROTOCOL)", r"_PROTOCOL", title_text)

        # -- Extract body content (Summary / GUID) --------------------------
        if isinstance(parse_input, list):
            result = _parse_node_list(parse_input)
        else:
            result = parse_node_for_protocol(parse_input)

        # -- Parent-child fallback ------------------------------------------
        # Some ---- sections (e.g. "EFI Driver Diagnostics Protocol") have
        # no direct body — the Summary / GUID sits in a child #### section.
        # Borrow it when the child's base name matches.
        if not result and isinstance(node, (section, subtitle)):
            container = node.parent if isinstance(node, subtitle) else node
            parent_norm = re.sub(r"\s+", "_", title_text.upper())
            if parent_norm.startswith("EFI_") and parent_norm.endswith("_PROTOCOL"):
                parent_base = parent_norm.removeprefix("EFI_").removesuffix("_PROTOCOL")
                for child in _matching_child_section(container, parent_base):
                    child_result = _parse_node_list(list(child))
                    if child_result is not None:
                        result = child_result
                        break

        # -- Register the protocol if valid ---------------------------------
        if result is not None:
            name = _canonical_name(result["define_name"], title_text)
            if not name.startswith("EFI_"):
                continue
            protocols.append({
                "name": name,
                "guid": result["guid"],
                "summary": result["summary"],
            })

    return protocols


# ---------------------------------------------------------------------------
# RST file collection
# ---------------------------------------------------------------------------


def collect_rst_files(root_dir: str) -> list[str]:
    rst_files = []
    for dirpath, _dirnames, filenames in os.walk(root_dir):
        for fname in filenames:
            if fname.endswith(".rst"):
                rst_files.append(os.path.join(dirpath, fname))
    return rst_files


# ---------------------------------------------------------------------------
# Reference map construction
# ---------------------------------------------------------------------------


def _get_section_title(node):
    title_node = next((c for c in node.children if isinstance(c, title)), None)
    return title_node.astext().strip() if title_node else None


def build_ref_map(rst_files: list[str]):
    """Build a global ``:ref:`` → section-title map.

    In RST a ``.. _my-target:`` directive before a heading causes the heading
    section to carry that target's ID.  The two-pass approach below handles
    this:

    1. Map section ``ids`` to section titles directly.
    2. For ``target`` nodes whose refid is not yet mapped, walk up the parent
       tree to find the enclosing section.
    """
    for filepath in rst_files:
        try:
            with open(filepath, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()
        except Exception:
            continue

        try:
            doctree = publish_doctree(content)
        except Exception:
            continue

        for sec in doctree.findall():
            if not isinstance(sec, section):
                continue
            sec_title = _get_section_title(sec)
            if not sec_title:
                continue
            for sid in sec.get("ids", []):
                if sid not in REF_MAP:
                    REF_MAP[sid] = sec_title

        for tgt in doctree.findall():
            if not isinstance(tgt, target):
                continue
            refid = tgt.get("refid")
            if not refid or refid in REF_MAP:
                continue
            parent = tgt
            while parent.parent is not None:
                parent = parent.parent
                if isinstance(parent, section):
                    sec_title = _get_section_title(parent)
                    if sec_title:
                        REF_MAP[refid] = sec_title
                    break


# ---------------------------------------------------------------------------
# Orphan GUID scanner — protocols defined outside protocol sections
# ---------------------------------------------------------------------------


def scan_orphan_protocol_guids(rst_files: list[str], known_guids: set[str]) -> list[dict]:
    """Scan all RST files for ``#define XXX_PROTOCOL_GUID`` in code blocks
    not already captured by protocol section parsing.

    This picks up protocols like ``EFI_HII_PACKAGE_LIST_PROTOCOL`` whose
    GUID define lives inside a function description rather than a dedicated
    protocol section.
    """
    found: list[dict] = []
    seen_names: set[str] = set()
    seen_guids: set[str] = set(known_guids)

    # Find define lines and extract surrounding context for GUID parsing
    define_line_pat = re.compile(r'#define\s+(\w+_PROTOCOL_GUID)\b')

    for filepath in rst_files:
        try:
            with open(filepath, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()
        except Exception:
            continue

        for m in define_line_pat.finditer(content):
            define_name = m.group(1)
            name = define_name.removesuffix("_GUID")

            if not name.startswith("EFI_"):
                continue
            if name in seen_names:
                continue

            # Extract context from the define line onward for GUID parsing
            start = m.start()
            end = min(start + 2000, len(content))
            context = content[start:end]

            result = parse_guid_and_name_from_code_block(context)
            if result is None or result[0] is None:
                continue
            guid, _ = result

            if guid in seen_guids:
                continue

            seen_guids.add(guid)
            seen_names.add(name)
            found.append({
                "name": name,
                "guid": guid,
                "summary": "",
            })

    return found


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main():
    with tempfile.TemporaryDirectory() as temp_dir:
        print(f"Using temporary directory: {temp_dir}")

        # Download both specs
        all_rst_files = []
        for url, label in [(UEFI_SPEC_URL, "UEFI"), (PI_SPEC_URL, "PI")]:
            spec_dir = download_and_extract(url, temp_dir, label.lower())
            print(f"Extracted {label} spec to: {spec_dir}")
            rst_files = collect_rst_files(spec_dir)
            print(f"  Found {len(rst_files)} RST files.")
            all_rst_files.extend(rst_files)

        print(f"\nTotal RST files: {len(all_rst_files)}")

        print("Building reference map...")
        build_ref_map(all_rst_files)
        print(f"Resolved {len(REF_MAP)} references.")

        all_protocols = []
        for i, rst_file in enumerate(all_rst_files):
            if (i + 1) % 50 == 0:
                print(f"  Processed {i + 1}/{len(all_rst_files)} files...")
            all_protocols.extend(parse_rst_file(rst_file))

        # Deduplicate by name (keep first occurrence)
        seen_names = set()
        unique_protocols = []
        known_guids = set()
        for p in all_protocols:
            if p["name"] not in seen_names:
                seen_names.add(p["name"])
                known_guids.add(p["guid"])
                unique_protocols.append(p)

        # Scan for orphan protocol GUIDs not captured by section parsing
        print(f"\nScanning for orphan protocol GUIDs in code blocks...")
        orphans = scan_orphan_protocol_guids(all_rst_files, known_guids)
        if orphans:
            print(f"  Found {len(orphans)} additional protocol(s).")
            unique_protocols.extend(orphans)

        unique_protocols.sort(key=lambda p: p["name"])

        print(f"\nFound {len(unique_protocols)} unique protocols.")

        output_path = os.path.join(os.getcwd(), OUTPUT_FILE)
        with open(output_path, "w", encoding="utf-8") as f:
            for p in unique_protocols:
                f.write(f'{p["name"]} => "{p["guid"]}" => r#"{p["summary"]}"#,\n')

        print(f"Written to {output_path}")


if __name__ == "__main__":
    main()
