const games = document.getElementById("games");
const canvas = document.getElementById("aether-canvas");
const engineStatus = document.getElementById("engine-status");
const engineProgress = document.getElementById("engine-progress");
const gameLoading = document.getElementById("game-loading");
const gameLoadingLabel = document.getElementById("game-loading-label");
const gameLoadingProgress = document.getElementById("game-loading-progress");
const gameTitle = document.getElementById("game-title");
const gameHelp = document.getElementById("game-help");
const basePath = "__AETHER_PREFIX__";
const sitePath = path => `${basePath}${path}`;
const engineUrl = sitePath("/lobby_web_bg.wasm");

let host;
let manifest = [];
let activeGame;
let engineReady;

function setBar(element, value) {
  element.style.transform = `scaleX(${Math.max(0, Math.min(1, value))})`;
}

async function readResource(url, onProgress) {
  const response = await fetch(url, { cache: "force-cache" });
  if (!response.ok) {
    throw new Error(`Failed to load ${url}: ${response.status}`);
  }
  const length = Number.parseInt(response.headers.get("content-length") ?? "0", 10);
  const reader = response.body?.getReader();
  if (!reader) {
    const bytes = new Uint8Array(await response.arrayBuffer());
    onProgress(1);
    return bytes;
  }
  const chunks = [];
  let loaded = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    loaded += value.byteLength;
    onProgress(length > 0 ? loaded / length : 0);
  }
  const bytes = new Uint8Array(loaded);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  onProgress(1);
  return bytes;
}

async function preloadEngine() {
  engineStatus.textContent = "Downloading shared engine";
  const bytes = await readResource(engineUrl, value => setBar(engineProgress, value));
  engineStatus.textContent = "Compiling shared engine";
  const module = await import(sitePath("/lobby_web.js"));
  await module.default(bytes);
  host = module;
  engineStatus.textContent = "Engine ready";
  setBar(engineProgress, 1);
}

function renderManifest() {
  games.innerHTML = manifest.map(game => `
    <a class="game" href="${sitePath(`/game/${game.id}/`)}" data-game="${game.id}">
      <span class="game-title">${game.name}</span>
      <span>${game.description}</span>
      <strong>Launch -&gt;</strong>
    </a>
  `).join("");
  games.querySelectorAll("[data-game]").forEach(link => {
    link.addEventListener("click", event => {
      event.preventDefault();
      selectGame(link.dataset.game, true);
    });
  });
}

async function preloadGame(game) {
  gameLoading.classList.add("is-visible");
  gameLoadingLabel.textContent = `Loading ${game.name}`;
  setBar(gameLoadingProgress, 0);
  for (let index = 0; index < game.assets.length; index += 1) {
    await readResource(sitePath(`/${game.assets[index]}`), value => {
      setBar(gameLoadingProgress, (index + value) / game.assets.length);
    });
  }
  setBar(gameLoadingProgress, 1);
}

async function selectGame(id, updateUrl) {
  const game = manifest.find(item => item.id === id);
  if (!game || activeGame) return;
  activeGame = game;
  if (updateUrl) history.pushState({ game: id }, "", sitePath(`/game/${id}/`));
  await engineReady;
  await preloadGame(game);
  gameTitle.textContent = game.name;
  gameHelp.textContent = game.id === "runner"
    ? "Arrow keys or A/D steer · Space jump and glide · Hold F or E to swing"
    : game.id === "shipwright"
      ? "Click faces to build · Shift-click to remove · Drag to orbit"
      : "WASD move · Space/Shift vertical · Hold left mouse to look";
  canvas.classList.add("is-visible");
  document.body.classList.add("game-active");
  gameLoadingLabel.textContent = "Starting game";
  host.start_game(game.id);
  gameLoading.classList.remove("is-visible");
}

async function boot() {
  manifest = await fetch(sitePath("/games.json"), { cache: "force-cache" }).then(response => response.json());
  renderManifest();
  engineReady = preloadEngine().catch(error => {
    engineStatus.textContent = "Engine failed to load";
    console.error(error);
    throw error;
  });
  const route = location.pathname.slice(basePath.length).match(/^\/game\/([^/]+)\/?$/);
  if (route) selectGame(route[1], false);
}

window.addEventListener("popstate", () => {
  if (!activeGame) {
    const route = location.pathname.slice(basePath.length).match(/^\/game\/([^/]+)\/?$/);
    if (route) selectGame(route[1], false);
  }
});

boot().catch(error => {
  engineStatus.textContent = "Unable to load games";
  console.error(error);
});