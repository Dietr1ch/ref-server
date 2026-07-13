;;; Directory Local Variables         -*- no-byte-compile: t; -*-
;;; For more information see (info "(emacs) Directory Variables")

(
 ;; Rust
 (rust-mode . (
							 (rustic-rustfmt-args . "--edition=2024")
							 (lsp-rust-analyzer-rustfmt-extra-args . ["--edition=2024"])
							 ))
 ;; Nix
 (nix-mode . (
							(nix-nixfmt-args . '(
																	 "--strict" ;; strict-mode
																	 "-"  ;; Process stdin
																	 ))
							))
 )
