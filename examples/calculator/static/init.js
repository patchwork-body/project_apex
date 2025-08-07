import init from '/static/client.js';

async function run() {
  try {
    await init();
    console.log('WASM loaded successfully!');
  } catch (error) {
    console.error('Failed to load WASM:', error);
  }
}

run();
