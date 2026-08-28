const progressBar = document.getElementById("loading-bar");
const progressLabel = document.getElementById("loading-label");
const progressHint = document.getElementById("loading-hint");
const overlay = document.getElementById("loading-overlay");

const gameTitle = document.body.dataset.gameTitle ?? "Aether Isles";
const modulePath = document.body.dataset.module;
const assetManifest = JSON.parse(document.body.dataset.assets ?? "[]");
const helpText = document.body.dataset.help ?? "";
const lobbyLink = document.getElementById("lobby-link");
const hintSlot = document.getElementById("game-hint");

if (lobbyLink) {
  lobbyLink.textContent = "<- Lobby";
}
if (hintSlot) {
  hintSlot.textContent = helpText;
}
document.title = gameTitle;

function setProgress(value, text) {
  const clamped = Math.max(0, Math.min(1, value));
  progressBar.style.transform = `scaleX(${clamped})`;
  progressLabel.textContent = text;
}

async function preloadOne(url, index, total) {
  const response = await fetch(url, { cache: "force-cache" });
  if (!response.ok) {
    throw new Error(`Failed to load ${url}: ${response.status}`);
  }

  const lengthHeader = response.headers.get("content-length");
  const totalBytes = lengthHeader ? Number.parseInt(lengthHeader, 10) : 0;
  const reader = response.body?.getReader();
  if (!reader) {
    await response.arrayBuffer();
    setProgress(index / total, `Loaded ${index}/${total} resources`);
    return null;
  }

  const chunks = [];
  let loadedBytes = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    if (value) {
      chunks.push(value);
      loadedBytes += value.byteLength;
      const fraction = totalBytes > 0 ? loadedBytes / totalBytes : 0;
      const stepProgress = ((index - 1) + fraction) / total;
      setProgress(stepProgress, `Loading ${index}/${total} resources`);
    }
  }

  const merged = new Uint8Array(loadedBytes);
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.byteLength;
  }
  setProgress(index / total, `Loaded ${index}/${total} resources`);
  return merged;
}

async function boot() {
  if (!modulePath) {
    throw new Error("Missing data-module on game page");
  }
  const total = assetManifest.length;
  const cache = new Map();

  if (total > 0) {
    progressHint.textContent = "Preparing game resources";
    for (let i = 0; i < assetManifest.length; i += 1) {
      const url = assetManifest[i];
      const bytes = await preloadOne(url, i + 1, total);
      if (bytes) {
        cache.set(url, bytes);
      }
    }
  }

  progressHint.textContent = "Starting engine";
  setProgress(1, "Starting game");
  const init = (await import(modulePath)).default;
  const wasmPath = assetManifest.find((asset) => asset.endsWith(".wasm"));
  const wasmBytes = wasmPath ? cache.get(wasmPath) : null;
  if (wasmBytes) {
    await init(wasmBytes);
  } else {
    await init();
  }
  overlay.classList.add("is-hidden");
}

boot().catch((error) => {
  console.error(error);
  progressHint.textContent = "Failed to load game resources";
  progressLabel.textContent = error instanceof Error ? error.message : "Unknown error";
});