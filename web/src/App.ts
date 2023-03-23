import init, { get_wasm_version, get_all_image_paths, get_all_resource_names, GameState } from 'tmv';

const ROOT = '/mv/';

var gameState: GameState | null = null;

let frameTimes: number[] = [];
let lastTimestamp: number | null = null;
let debugOpen = false;

function rafLoop(timestamp: number) {
  const powerupState = gameState!.get_char_state();
  const infoLine = gameState!.get_info_line();
  document.getElementById('infoLine')!.innerText = infoLine;
  document.getElementById('hpIndicator')!.innerText = '❤️'.repeat(Math.max(0, powerupState.hp));

  ["wall_jump", "dash", "water", "small", "lava", "double_jump"].map((powerUpName, i) => {
    const havePowerUp = powerupState.power_ups.includes(powerUpName);
    document.getElementById('powerup' + (i + 1))!.style.display = havePowerUp ? 'flex' : 'none';
  });

  // const frameTime = timestamp - this.lastFrameTimestamp;
  // this.fps = 1000 / frameTime;
  // if (this.fpsCounterRef.current !== null) {
  //   this.fpsCounterRef.current.innerText = `FPS: ${this.fps.toFixed(2)} - ${this.conn.syncedGameWorld!.get_debug_string()}`;
  // }
    // this.conn.syncedGameWorld!.step(1e-3 * frameTime);
    // this.conn.syncedGameWorld!.draw_frame();
  //} finally {
  if (lastTimestamp !== null) {
    // Don't step by more than a tenth of a second at a time.
    const dt = Math.min(0.1, 1e-3 * (timestamp - lastTimestamp));
    gameState!.step(dt);
    frameTimes.push(dt);
    if (frameTimes.length > 10) {
      frameTimes.shift();
    }
    const fps = 1 / frameTimes.reduce((a, b) => a + b, 0) * frameTimes.length;
    document.getElementById('fpsCounter')!.innerText = `FPS: ${fps.toFixed(2)}`;
  }
  gameState!.draw_frame();
  window.requestAnimationFrame(rafLoop);
  lastTimestamp = timestamp;
}

function onKeyDown(e: KeyboardEvent) {
  if (e.repeat)
    return;
  if (e.key === 'f') {
    debugOpen = !debugOpen;
    document.getElementById('fpsCounter')!.style.display = debugOpen ? 'block' : 'none';
  }
  if (gameState !== null) {
    gameState.apply_input_event(JSON.stringify({ type: 'KeyDown', key: e.key }));
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.repeat)
    return;
  if (gameState !== null) {
    gameState.apply_input_event(JSON.stringify({ type: 'KeyUp', key: e.key }));
  }
}

let savingInterval: any = null;

(window as any).clearProgress = function() {
  if (window.confirm('Are you sure you want to completely restart the game?')) {
    clearInterval(savingInterval);
    localStorage.removeItem('pmvSaveData');
    window.location.reload();
  }
}

async function main() {
  await init();
  console.log('Hello, world: ' + get_wasm_version());

  // Load all the images
  const allImagePaths = get_all_image_paths();
  console.log('Loading images:', allImagePaths);
  for (const path of allImagePaths) {
    const img = new Image();
    img.src = ROOT + path;
    img.style.display = 'none';
    img.style.imageRendering = 'pixelated';
    img.id = path;
    document.body.appendChild(img);
  }

  // Begin loading all the resources
  const allResourceNames = get_all_resource_names();
  console.log('Loading resources:', allResourceNames);
  const resourcePromises = allResourceNames.map((name: string) => {
    return fetch(ROOT + name).then((res) => res.arrayBuffer()).then((buf) => {
      console.log(`Loaded resource ${name}: ${buf.byteLength} bytes`);
      return { name, buf };
    });
  });
  Promise.all(resourcePromises).then((results) => {
    const resources: { [name: string]: Uint8Array } = {};
    results.forEach((result: any) => {
      resources[result.name] = new Uint8Array(result.buf);
    });
    
    console.log('All resources loaded');
    gameState = new GameState(resources);
    const pmvSaveData = localStorage.getItem('pmvSaveData');
    if (pmvSaveData !== null) {
      gameState.apply_save_data(pmvSaveData);
    }
    // FIXME: There's no need to save so frequently, but also it doesn't matter?
    savingInterval = setInterval(() => {
      const saveData = gameState!.get_save_data();
      localStorage.setItem('pmvSaveData', saveData);
    }, 500);

    window.requestAnimationFrame(rafLoop);
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
  });
}

main();

export {};
