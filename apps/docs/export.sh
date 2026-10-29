#!/usr/bin/env bash
# Pre-render the docs site for static hosting: build it, run the server, follow every
# internal link from `/`, and save each page as HTML next to the wasm bundle.
# Output: target/site (upload as-is to Cloudflare Pages or any static host).
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo leptos build --release -p hydroxy-docs
site=target/site
port=8091

LEPTOS_SITE_ROOT=$site LEPTOS_SITE_ADDR=127.0.0.1:$port LEPTOS_HASH_FILES=true LEPTOS_ENV=PROD \
  target/server-release/hydroxy-docs >/dev/null &
server=$!
trap 'kill $server 2>/dev/null' EXIT
up=
for _ in $(seq 60); do curl -sf "http://127.0.0.1:$port/" >/dev/null && up=1 && break; sleep 1; done
[[ -n $up ]] || { echo "server did not start on port $port" >&2; exit 1; }

# Breadth-first over same-site links; /pkg and /fonts are build output, already in place.
queue=("/")
seen=" / "
pages=0
while ((${#queue[@]})); do
  path=${queue[0]}
  queue=("${queue[@]:1}")
  html=$(curl -sf "http://127.0.0.1:$port$path") || { echo "failed to fetch $path" >&2; exit 1; }
  file=$site${path%/}
  [[ $path == / ]] && file=$site/index
  mkdir -p "$(dirname "$file")"
  printf '%s' "$html" >"$file.html"
  pages=$((pages + 1))
  for link in $(grep -oE 'href="/[^"#?]*' <<<"$html" | cut -c7- | sort -u); do
    [[ $link == /pkg/* || $link == /fonts/* || $link == *.* ]] && continue
    [[ $seen == *" $link "* ]] && continue
    seen+="$link "
    queue+=("$link")
  done
done

curl -s "http://127.0.0.1:$port/__not-found" >"$site/404.html"
((pages >= 200)) || { echo "only $pages pages exported, expected at least 200" >&2; exit 1; }
echo "exported $pages pages to $site"
