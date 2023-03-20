import init, { get_wasm_version, get_all_image_paths, get_all_resource_names, GameState } from 'tmv';

var gameState: GameState | null = null;

let frameTimes: number[] = [];
let lastTimestamp: number | null = null;

function rafLoop(timestamp: number) {
  const powerupState = gameState!.get_powerup_state();
  document.getElementById('coins')!.innerText = powerupState.coins.toString();

  // const frameTime = timestamp - this.lastFrameTimestamp;
  // this.fps = 1000 / frameTime;
  // if (this.fpsCounterRef.current !== null) {
  //   this.fpsCounterRef.current.innerText = `FPS: ${this.fps.toFixed(2)} - ${this.conn.syncedGameWorld!.get_debug_string()}`;
  // }
    // this.conn.syncedGameWorld!.step(1e-3 * frameTime);
    // this.conn.syncedGameWorld!.draw_frame();
  //} finally {
  if (lastTimestamp !== null) {
    const dt = timestamp - lastTimestamp;
    gameState!.step(1e-3 * dt);
    frameTimes.push(dt);
    if (frameTimes.length > 10) {
      frameTimes.shift();
    }
    const fps = 1000 / frameTimes.reduce((a, b) => a + b, 0) * frameTimes.length;
    document.getElementById('fpsCounter')!.innerText = `FPS: ${fps.toFixed(2)}`;
  }
  gameState!.draw_frame();
  window.requestAnimationFrame(rafLoop);
  lastTimestamp = timestamp;
}

function onKeyDown(e: KeyboardEvent) {
  if (e.repeat)
    return;
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

async function main() {
  await init();
  console.log('Hello, world: ' + get_wasm_version());

  // Load all the images
  const allImagePaths = get_all_image_paths();
  console.log('Loading images:', allImagePaths);
  for (const path of allImagePaths) {
    const img = new Image();
    img.src = path;
    img.style.display = 'none';
    img.id = path;
    document.body.appendChild(img);
  }

  // Begin loading all the resources
  const allResourceNames = get_all_resource_names();
  console.log('Loading resources:', allResourceNames);
  const resourcePromises = allResourceNames.map((name: string) => {
    return fetch(name).then((res) => res.arrayBuffer()).then((buf) => {
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
    window.requestAnimationFrame(rafLoop);
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
  });
}

main();

export {};
