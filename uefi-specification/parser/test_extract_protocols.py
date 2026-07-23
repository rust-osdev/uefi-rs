#!/usr/bin/env python3
import os
import re
import tempfile
import unittest

import extract_protocols as ep


class TestParseGuidAndNameFromCodeBlock(unittest.TestCase):
    def test_standard_guid(self):
        text = (
            "#define EFI_DEVICE_PATH_PROTOCOL_GUID \\\n"
            "    {0x09576e91,0x6d3f,0x11d2,\\\n"
            "      {0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b}}"
        )
        guid, name = ep.parse_guid_and_name_from_code_block(text)
        self.assertEqual(guid, "09576e91-6d3f-11d2-8e39-00a0c969723b")
        self.assertEqual(name, "EFI_DEVICE_PATH_PROTOCOL")

    def test_no_guid_returns_none(self):
        guid, name = ep.parse_guid_and_name_from_code_block("no hex here")
        self.assertIsNone(guid)
        self.assertIsNone(name)

    def test_incomplete_guid_returns_none(self):
        guid, name = ep.parse_guid_and_name_from_code_block(
            "#define X_GUID \\\n    {0x1234,0x5678}"
        )
        self.assertIsNone(guid)
        self.assertEqual(name, "X")


class TestIsProtocolName(unittest.TestCase):
    def test_valid_underscore(self):
        self.assertTrue(ep.is_protocol_name("EFI_DEVICE_PATH_PROTOCOL"))

    def test_valid_spaces(self):
        self.assertTrue(ep.is_protocol_name("EFI Device Path Protocol"))

    def test_no_protocol_keyword(self):
        self.assertFalse(ep.is_protocol_name("EFI_DEVICE_PATH"))

    def test_skip_keyword(self):
        self.assertFalse(ep.is_protocol_name("INTRODUCTION TO PROTOCOLS"))

    def test_protocol_descriptions_skipped(self):
        self.assertFalse(ep.is_protocol_name("Protocol Descriptions"))

    def test_protocol_handler_skipped(self):
        self.assertFalse(ep.is_protocol_name("Protocol Handler Services"))

    def test_contains_dot(self):
        self.assertFalse(ep.is_protocol_name("EFI.SOME.PROTOCOL"))

    def test_contains_dash(self):
        self.assertFalse(ep.is_protocol_name("EFI-SOME-PROTOCOL"))

    def test_single_word_protocol_ok(self):
        self.assertTrue(ep.is_protocol_name("Font Protocol"))
        self.assertTrue(ep.is_protocol_name("String Protocol"))


def _protocol_rst(title, summary, guid_hex):
    """Build RST resembling the UEFI spec structure (level-2 protocol section)."""
    # guid_hex = [a1, a2, a3, b1, b2, b3, b4, b5, b6, b7, b8]
    # -> #define X_GUID {0xa1,0xa2,0xa3,{0xb1,0xb2,...,0xb8}}
    a = ",".join("0x" + h for h in guid_hex[:3])
    b = ",".join("0x" + h for h in guid_hex[3:])
    underline = "#" * len(title)
    return (
        "Document Title\n"
        "==============\n"
        "\n"
        "Dummy Section\n"
        "-------------\n"
        "\n"
        + title + "\n"
        + underline + "\n"
        "\n"
        "**Summary**\n"
        "\n"
        + summary + "\n"
        "\n"
        "**GUID**\n"
        "\n"
        ".. code-block::\n"
        "\n"
        "   #define " + title + "_GUID \\\n"
        "    {" + a + ",\\\n"
        "      {" + b + "}}\n"
    )


class TestRefResolution(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        ep.REF_MAP.clear()

    def tearDown(self):
        self.temp_dir.cleanup()

    def _write(self, rel: str, content: str):
        path = os.path.join(self.temp_dir.name, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)

    def _abs(self, rel: str) -> str:
        return os.path.join(self.temp_dir.name, rel)

    def test_resolves_ref_to_section_title(self):
        target = """\
Document Title
==============

Dummy Section
-------------

.. _my-protocol:

EFI_MY_PROTOCOL
###############

**Summary**

Test.

**GUID**

.. code-block::

   #define EFI_MY_PROTOCOL_GUID \\
    {0x00000000,0x0000,0x0000,\\
      {0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01}}
"""
        referrer = """\
Document Title
==============

Dummy Section
-------------

EFI_REFERRING_PROTOCOL
######################

**Summary**

See :ref:`my-protocol` for details.

**GUID**

.. code-block::

   #define EFI_REFERRING_PROTOCOL_GUID \\
    {0x00000000,0x0000,0x0000,\\
      {0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x02}}
"""
        self._write("target.rst", target)
        self._write("referrer.rst", referrer)

        ep.build_ref_map([self._abs("target.rst")])
        protocols = ep.parse_rst_file(self._abs("referrer.rst"))
        self.assertEqual(len(protocols), 1)
        s = protocols[0]["summary"]
        self.assertIn("EFI_MY_PROTOCOL", s)
        self.assertNotIn(":ref:", s)

    def test_ref_with_target_in_same_file(self):
        self._write(
            "combined.rst",
            """\
Document Title
==============

Dummy Section
-------------

.. _my-protocol:

EFI_MY_PROTOCOL
###############

**Summary**

See :ref:`my-protocol` for details.

**GUID**

.. code-block::

   #define EFI_MY_PROTOCOL_GUID \\
    {0x00000000,0x0000,0x0000,\\
      {0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01}}
""",
        )
        ep.build_ref_map([self._abs("combined.rst")])
        protocols = ep.parse_rst_file(self._abs("combined.rst"))
        self.assertEqual(len(protocols), 1)
        s = protocols[0]["summary"]
        self.assertIn("EFI_MY_PROTOCOL", s)
        self.assertNotIn(":ref:", s)


class TestProtocolDetection(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.spec_dir = os.path.join(self.temp_dir.name, "spec")
        os.makedirs(self.spec_dir)

    def tearDown(self):
        self.temp_dir.cleanup()

    def _write(self, filename: str, content: str):
        with open(os.path.join(self.spec_dir, filename), "w", encoding="utf-8") as f:
            f.write(content)

    def test_efi_device_path_protocol(self):
        self._write(
            "dp.rst",
            _protocol_rst(
                "EFI_DEVICE_PATH_PROTOCOL",
                "Used to retrieve device paths.",
                ["09576e91", "6d3f", "11d2",
                 "8e", "39", "00", "a0", "c9", "69", "72", "3b"],
            ),
        )
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "dp.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_DEVICE_PATH_PROTOCOL")
        self.assertEqual(p[0]["guid"], "09576e91-6d3f-11d2-8e39-00a0c969723b")
        self.assertIn("retrieve device paths", p[0]["summary"])

    def test_efi_hii_image_decoder_protocol(self):
        self._write(
            "hii.rst",
            _protocol_rst(
                "EFI_HII_IMAGE_DECODER_PROTOCOL",
                "Provides image decoding services.",
                ["9e4caed1", "eb8c", "4f3b",
                 "b3", "1d", "a0", "3b", "0f", "0c", "6b", "3a"],
            ),
        )
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "hii.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_HII_IMAGE_DECODER_PROTOCOL")
        self.assertEqual(p[0]["guid"], "9e4caed1-eb8c-4f3b-b31d-a03b0f0c6b3a")

    def test_efi_kms_protocol(self):
        self._write(
            "kms.rst",
            _protocol_rst(
                "EFI_KMS_PROTOCOL",
                "Manages key management services.",
                ["ec3f0dd8", "6e3f", "4b2c",
                 "a1", "2a", "8e", "9b", "2e", "6d", "1c", "4a"],
            ),
        )
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "kms.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_KMS_PROTOCOL")
        self.assertEqual(p[0]["guid"], "ec3f0dd8-6e3f-4b2c-a12a-8e9b2e6d1c4a")

    def test_heading_fix_missing_underscore(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_MEMORY_ATTRIBUTE PROTOCOL
#############################

**Summary**

Test.

**GUID**

.. code-block::

   #define EFI_MEMORY_ATTRIBUTE_PROTOCOL_GUID \\
    {0x00000000,0x0000,0x0000,\\
      {0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01}}
"""
        self._write("fix.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "fix.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_MEMORY_ATTRIBUTE_PROTOCOL")

    def test_heading_fix_trailing_summary(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_SOME_PROTOCOL Summary
#########################

**Summary**

Test.

**GUID**

.. code-block::

   #define EFI_SOME_PROTOCOL_GUID \\
    {0x00000000,0x0000,0x0000,\\
      {0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01}}
"""
        self._write("fix2.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "fix2.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_SOME_PROTOCOL")

    def test_spaced_heading_resolves_via_define(self):
        """Heading like 'EFI Device Path Protocol' should get canonical name from define."""
        content = """\
Document Title
==============

Dummy Section
-------------

EFI Device Path Protocol
------------------------

**Summary**

Used to retrieve device paths.

**GUID**

.. code-block::

   #define EFI_DEVICE_PATH_PROTOCOL_GUID \\
    {0x09576e91,0x6d3f,0x11d2,\\
      {0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b}}
"""
        self._write("dp.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "dp.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_DEVICE_PATH_PROTOCOL")
        self.assertEqual(p[0]["guid"], "09576e91-6d3f-11d2-8e39-00a0c969723b")

    def test_name_heading_preferred_over_define(self):
        """Heading is EFI_KEY_MANAGEMENT_SERVICE_PROTOCOL, define says EFI_KMS_PROTOCOL_GUID."""
        content = """\
Document Title
==============

Dummy Section
-------------

.. _efi-key-management-service-protocol:

EFI_KEY_MANAGEMENT_SERVICE_PROTOCOL
###################################

**Summary**

Manages key management services.

**GUID**

.. code-block::

   #define EFI_KMS_PROTOCOL_GUID \\
    {0xec3f0dd8,0x6e3f,0x4b2c,\\
      {0xa1,0x2a,0x8e,0x9b,0x2e,0x6d,0x1c,0x4a}}
"""
        self._write("kms.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "kms.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_KEY_MANAGEMENT_SERVICE_PROTOCOL")
        self.assertEqual(p[0]["guid"], "ec3f0dd8-6e3f-4b2c-a12a-8e9b2e6d1c4a")

    def test_summary_guid_in_child_section(self):
        """Protocol with Summary/GUID inside first child function section."""
        content = """\
Document Title
==============

Dummy Section
-------------

EFI HII Image Decoder Protocol
------------------------------

Some intro text.

EFI_HII_IMAGE_DECODER_PROTOCOL.DecodeImage()
#############################################

**Summary**

Provides image decoding services.

**GUID**

.. code-block::

   #define EFI_HII_IMAGE_DECODER_PROTOCOL_GUID \\
    {0x9e4caed1,0xeb8c,0x4f3b,\\
      {0xb3,0x1d,0xa0,0x3b,0x0f,0x0c,0x6b,0x3a}}
"""
        self._write("hii.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "hii.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_HII_IMAGE_DECODER_PROTOCOL")
        self.assertEqual(p[0]["guid"], "9e4caed1-eb8c-4f3b-b31d-a03b0f0c6b3a")

    def test_efi_driver_diagnostics2_protocol(self):
        """Heading EFI_DRIVER_DIAGNOSTICS2_PROTOCOL preserved from #### section."""
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_DRIVER_DIAGNOSTICS2_PROTOCOL
################################

**Summary**

Used to perform diagnostics on a controller.

**GUID**

.. code-block::

   #define EFI_DRIVER_DIAGNOSTICS_PROTOCOL_GUID \\
    {0x4d330321,0x025f,0x4aac,\\
      {0x90,0xd8,0x5e,0xd9,0x00,0x17,0x3b,0x63}}
"""
        self._write("dd2.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "dd2.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_DRIVER_DIAGNOSTICS2_PROTOCOL")
        self.assertEqual(p[0]["guid"], "4d330321-025f-4aac-90d8-5ed900173b63")

    def test_efi_key_management_service_protocol(self):
        """Heading says KEY_MANAGEMENT_SERVICE, define says KMS."""
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_KEY_MANAGEMENT_SERVICE_PROTOCOL
###################################

**Summary**

The Key Management Service protocol manages keys.

**GUID**

.. code-block::

   #define EFI_KMS_PROTOCOL_GUID \\
    {0xec3a978d,0x7c4e,0x48fa,\\
      {0x9a,0xbe,0x6a,0xd9,0x1c,0xc8,0xf8,0x11}}
"""
        self._write("kms.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "kms.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_KEY_MANAGEMENT_SERVICE_PROTOCOL")
        self.assertEqual(p[0]["guid"], "ec3a978d-7c4e-48fa-9abe-6ad91cc8f811")

    def test_efi_http_protocol(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_HTTP_PROTOCOL
#################

**Summary**

Provides HTTP services.

**GUID**

.. code-block::

   #define EFI_HTTP_PROTOCOL_GUID \\
    {0x7a59b29b,0x910b,0x4171,\\
      {0x82,0x42,0xa8,0x5a,0x0d,0xf2,0x5b,0x5b}}
"""
        self._write("http.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "http.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_HTTP_PROTOCOL")
        self.assertEqual(p[0]["guid"], "7a59b29b-910b-4171-8242-a85a0df25b5b")

    def test_efi_http_utilities_protocol(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_HTTP_UTILITIES_PROTOCOL
###########################

**Summary**

Builds and parses HTTP headers.

**GUID**

.. code-block::

   #define EFI_HTTP_UTILITIES_PROTOCOL_GUID \\
    {0x3e35c163,0x4074,0x45dd,\\
      {0x43,0x1e,0x23,0x98,0x9d,0xd8,0x6b,0x32}}
"""
        self._write("httpu.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "httpu.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_HTTP_UTILITIES_PROTOCOL")
        self.assertEqual(p[0]["guid"], "3e35c163-4074-45dd-431e-23989dd86b32")

    def test_efi_regular_expression_protocol(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_REGULAR_EXPRESSION_PROTOCOL
###############################

**Summary**

Matches strings against regular expression patterns.

**GUID**

.. code-block::

   #define EFI_REGULAR_EXPRESSION_PROTOCOL_GUID \\
    {0xb3f79d9a,0x436c,0xdc11,\\
      {0xb0,0x52,0xcd,0x85,0xdf,0x52,0x4c,0xe6}}
"""
        self._write("regex.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "regex.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_REGULAR_EXPRESSION_PROTOCOL")
        self.assertEqual(p[0]["guid"], "b3f79d9a-436c-dc11-b052-cd85df524ce6")

    def test_efi_rest_protocol(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_REST_PROTOCOL
#################

**Summary**

Provides REST services.

**GUID**

.. code-block::

   #define EFI_REST_PROTOCOL_GUID \\
    {0x0db48a36,0x4e54,0xea9c,\\
      {0x9b,0x09,0x1e,0xa5,0xbe,0x3a,0x66,0x0b}}
"""
        self._write("rest.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "rest.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_REST_PROTOCOL")
        self.assertEqual(p[0]["guid"], "0db48a36-4e54-ea9c-9b09-1ea5be3a660b")

    def test_efi_partition_info_protocol(self):
        """Heading with ---- underline, content in siblings (subtitle case)."""
        content = """\
Document Title
==============

Dummy Section
-------------

Partition Information Protocol
------------------------------

**Summary**

Provides cached partition information for MBR and GPT partition types.

**GUID**

.. code-block::

   #define EFI_PARTITION_INFO_PROTOCOL_GUID \\
    {0x8cf2f62c,0xbc9b,0x4821,\\
      {0x80,0x8d,0xec,0x9e,0xc4,0x21,0xa1,0xa0}}
"""
        self._write("pi.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "pi.rst"))
        self.assertEqual(len(p), 1)
        self.assertEqual(p[0]["name"], "EFI_PARTITION_INFO_PROTOCOL")
        self.assertEqual(p[0]["guid"], "8cf2f62c-bc9b-4821-808d-ec9ec421a1a0")

    def test_efi_driver_diagnostics_both_v1_and_v2(self):
        """---- parent produces EFI_DRIVER_DIAGNOSTICS_PROTOCOL, #### child produces v2."""
        content = """\
Document Title
==============

EFI Driver Diagnostics Protocol
-------------------------------

EFI_DRIVER_DIAGNOSTICS2_PROTOCOL
################################

**Summary**

Used to perform diagnostics on a controller.

**GUID**

.. code-block::

   #define EFI_DRIVER_DIAGNOSTICS_PROTOCOL_GUID \\
    {0x4d330321,0x025f,0x4aac,\\
      {0x90,0xd8,0x5e,0xd9,0x00,0x17,0x3b,0x63}}
"""
        self._write("dd.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "dd.rst"))
        names = [entry["name"] for entry in p]
        self.assertIn("EFI_DRIVER_DIAGNOSTICS_PROTOCOL", names)
        self.assertIn("EFI_DRIVER_DIAGNOSTICS2_PROTOCOL", names)
        for entry in p:
            self.assertEqual(entry["guid"], "4d330321-025f-4aac-90d8-5ed900173b63")

    def test_multi_line_summary(self):
        content = """\
Document Title
==============

Dummy Section
-------------

EFI_MULTILINE_PROTOCOL
######################

**Summary**

First paragraph of the summary.

Second paragraph of the summary.

**GUID**

.. code-block::

   #define EFI_MULTILINE_PROTOCOL_GUID \\
    {0x00000000,0x0000,0x0000,\\
      {0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01}}
"""
        self._write("multi.rst", content)
        p = ep.parse_rst_file(os.path.join(self.spec_dir, "multi.rst"))
        self.assertEqual(len(p), 1)
        self.assertIn("First paragraph", p[0]["summary"])
        self.assertIn("Second paragraph", p[0]["summary"])

    def test_all_three_found_together(self):
        self._write(
            "dp.rst",
            _protocol_rst(
                "EFI_DEVICE_PATH_PROTOCOL",
                "Used to retrieve device paths.",
                ["09576e91", "6d3f", "11d2",
                 "8e", "39", "00", "a0", "c9", "69", "72", "3b"],
            ),
        )
        self._write(
            "hii.rst",
            _protocol_rst(
                "EFI_HII_IMAGE_DECODER_PROTOCOL",
                "Provides image decoding services.",
                ["9e4caed1", "eb8c", "4f3b",
                 "b3", "1d", "a0", "3b", "0f", "0c", "6b", "3a"],
            ),
        )
        self._write(
            "kms.rst",
            _protocol_rst(
                "EFI_KMS_PROTOCOL",
                "Manages key management services.",
                ["ec3f0dd8", "6e3f", "4b2c",
                 "a1", "2a", "8e", "9b", "2e", "6d", "1c", "4a"],
            ),
        )
        rst_files = ep.collect_rst_files(self.spec_dir)
        all_p = []
        for f in rst_files:
            all_p.extend(ep.parse_rst_file(f))
        names = {p["name"] for p in all_p}
        for expected in (
            "EFI_DEVICE_PATH_PROTOCOL",
            "EFI_HII_IMAGE_DECODER_PROTOCOL",
            "EFI_KMS_PROTOCOL",
        ):
            self.assertIn(expected, names)


class TestProtocolNamesExistInSpec(unittest.TestCase):
    """Integration test: every canonical protocol name must appear in the spec source files."""

    SPEC_URL = "https://github.com/UEFI/UEFI-Specification-Release/archive/refs/heads/main.zip"

    @classmethod
    def setUpClass(cls):
        cls.spec_text = cls._download_and_index_spec()
        cls.protocols = cls._extract_protocols()

    @classmethod
    def _download_and_index_spec(cls) -> str:
        import tempfile, urllib.request, zipfile
        with tempfile.TemporaryDirectory() as tmp:
            zip_path = os.path.join(tmp, "spec.zip")
            urllib.request.urlretrieve(cls.SPEC_URL, zip_path)
            all_text = []
            with zipfile.ZipFile(zip_path) as zf:
                for name in zf.namelist():
                    if name.endswith(".rst"):
                        all_text.append(zf.read(name).decode("utf-8", errors="replace"))
            return "\n".join(all_text)

    @classmethod
    def _extract_protocols(cls) -> list[dict]:
        import tempfile, urllib.request, zipfile
        with tempfile.TemporaryDirectory() as tmp:
            zip_path = os.path.join(tmp, "spec.zip")
            urllib.request.urlretrieve(cls.SPEC_URL, zip_path)
            with zipfile.ZipFile(zip_path) as zf:
                zf.extractall(tmp)
            root = os.path.join(tmp, os.listdir(tmp)[0])
            ep.REF_MAP.clear()
            rst_files = ep.collect_rst_files(root)
            ep.build_ref_map(rst_files)
            all_p = []
            for f in rst_files:
                all_p.extend(ep.parse_rst_file(f))
            seen = set()
            unique = []
            for p in all_p:
                if p["name"] not in seen:
                    seen.add(p["name"])
                    unique.append(p)
            return unique

    def test_every_protocol_name_appears_in_spec(self):
        missing = []
        for p in self.protocols:
            name = p["name"]
            if name not in self.spec_text:
                missing.append(name)
        self.assertEqual(
            missing, [],
            f"{len(missing)} protocol name(s) not found in spec source:\n"
            + "\n".join(f"  {n}" for n in missing),
        )


if __name__ == "__main__":
    unittest.main()
