SPHINXBUILD   = sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

.PHONY: help html generate clean

help:
	@echo "  html      build HTML documentation"
	@echo "  generate  regenerate auto-generated need files"
	@echo "  clean     remove build artifacts"

generate:
	python scripts/generate_all.py

html: generate
	$(SPHINXBUILD) -b html $(SOURCEDIR) $(BUILDDIR)/html

clean:
	rm -rf $(BUILDDIR)
