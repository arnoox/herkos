# -- Project information -------------------------------------------------------

project = "herkos"
copyright = "2025-2026, herkos contributors"
author = "herkos contributors"

# -- General configuration -----------------------------------------------------

extensions = [
    "myst_parser",
    "sphinx_needs",
    "sphinxcontrib.mermaid",
]

source_suffix = {
    ".md": "markdown",
    ".rst": "restructuredtext",
}

master_doc = "index"
exclude_patterns = ["_build", ".venv", "scripts"]

# -- MyST configuration --------------------------------------------------------

myst_enable_extensions = [
    "colon_fence",
    "fieldlist",
]

myst_heading_anchors = 3

# -- sphinx-needs configuration ------------------------------------------------

needs_from_toml = "ubproject.toml"

# -- HTML output ---------------------------------------------------------------

html_theme = "alabaster"
