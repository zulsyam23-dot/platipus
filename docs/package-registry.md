# P2LT HTTP registry

P2LT can use either its local directory registry or an HTTP(S) registry. Set
`P2LT_REGISTRY_URL` to the registry base URL (for example,
`https://packages.example.org/v1`). When it is set, the URL is used for
`p2lt add <name>`, `search`, `update`, and `publish`; otherwise P2LT retains the
local registry behavior.

## Endpoints

The protocol is deliberately small and can be implemented by a static file
host with PUT support:

| Method | Path | Request/response |
|---|---|---|
| `GET` | `/index.txt` | UTF-8 package names, one per line; blank lines and `#` comments are ignored |
| `GET` | `/packages/{name}.libplt` | The latest valid P2LT package archive |
| `PUT` | `/packages/{name}.libplt` | Upload/release the archive for `{name}` |

Package names are limited to ASCII letters, digits, `.`, `_`, and `-`. The
downloaded archive must pass the `.libplt` checksum and its manifest package
name must match the requested name. `curl` performs HTTP transfers, follows
redirects, and treats non-success HTTP status codes as errors. `p2lt publish`
uploads a freshly packed archive. The server is responsible for updating
`index.txt` when packages are added.

The protocol does not define authentication. A public registry must restrict
`PUT` through its hosting layer (for example, an authenticated reverse proxy);
do not expose an unauthenticated writable registry to the public internet.
Use HTTPS for non-local registries.

## GitHub dependencies

`p2lt add github.com/owner/repository` clones the default branch. An optional
branch or tag can be selected with `#ref`, for example
`p2lt add github.com/owner/repository#main`. An `https://github.com/...` URL and
the `.git` suffix are also accepted. The repository root must contain
`p2lt.toml` and either `src/lib.plt` or `src/main.plt`.

GitHub packages are installed into the global package store. The manifest
records the package version and the lockfile records the source and resolved
Git commit. `p2lt update` re-fetches the selected ref (or the default branch)
and updates the lockfile revision. `git` must be available on `PATH`.

## Reproducibility and safety

Registry installs are integrity-checked and the lockfile records a checksum of
the installed package tree. GitHub installs record both that checksum and the
resolved Git revision. The current archive checksum is FNV-1a for accidental
corruption detection, not a cryptographic signature. Review package source
before using it; P2LT does not execute package build scripts.
