# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'rax25kb'
copyright = '2025, Kris Kirby'
author = 'Kris Kirby'
release = '1.6.5'
version = '1.6.5'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.viewcode',
    'sphinx.ext.napoleon',
    'sphinx.ext.intersphinx',
    'sphinx.ext.todo',
]

templates_path = ['_templates']
exclude_patterns = []

language = 'en'

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']
html_logo = None
html_favicon = None

# -- Options for LaTeX output ------------------------------------------------
latex_elements = {
    'papersize': 'letterpaper',
    'pointsize': '10pt',
}

latex_documents = [
    ('index', 'rax25kb.tex', 'rax25kb Documentation',
     'Kris Kirby', 'manual'),
]

# -- Options for manual page output ------------------------------------------
man_pages = [
    ('manpages/rax25kb', 'rax25kb', 'rax25kb Documentation', [author], 1),
    ('manpages/rax25kb-cfg', 'rax25kb.cfg', 'rax25kb Configuration File', [author], 5),
]

# -- Options for Texinfo output ----------------------------------------------
texinfo_documents = [
    ('index', 'rax25kb', 'rax25kb Documentation',
     author, 'rax25kb', 'AX.25 KISS Bridge',
     'Miscellaneous'),
]

# -- Extension configuration -------------------------------------------------
todo_include_todos = True

# Intersphinx mapping
intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
}
