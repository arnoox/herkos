# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "herkos"
copyright = "2026, Arnaud Riess"
author = "Arnaud Riess"
release = "0.1.0"

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "myst_parser",
    "sphinx_needs",
]

myst_enable_extensions = [
    "colon_fence",
]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# -- Sphinx-Needs configuration ------------------------------------
# Load configuration from external TOML file
# This is the key line that connects sphinx-needs to ubproject.toml
needs_from_toml = "ubproject.toml"

# -- Options for HTML output ---------------------------------------
html_theme = "alabaster"
html_static_path = ["_static"]
