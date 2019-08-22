'use strict';

/* eslint-env browser */

// https://bugzilla.mozilla.org/show_bug.cgi?id=1536094
const script = document.createElement('script');
script.appendChild(document.createTextNode(`
// Ruffle Player

import init, { Player } from 'https://ruffle.rs/assets/ruffle.js';

const log = (...args) => console.info('[RUFFLE FLASH EMULATOR]', ...args);
const error = (...args) => console.error('[RUFFLE FLASH EMULATOR]', ...args);

const Ruffle = init().then(() => Player);

function convertObject(object) {
  const source = object.getAttribute('data');
  if (!source) {
    return;
  }
  log('Replacing flash content from', source);
  const div = document.createElement('div');
  {
    div.style = 'background-color: #FFAD33; display: inline-block;';

    const h1 = document.createElement('h1');
    h1.appendChild(document.createTextNode('▶️ Click to play flash content in Ruffle'));
    div.appendChild(h1);
  }
  object.parentElement.insertBefore(div, object);
  object.remove();
  fetch(new URL(source, window.location))
    .then((r) => r.arrayBuffer())
    .then((ab) => {
      return Ruffle.then((Player) => {
        const play = () => {
          log('Running', source);
          const canvas = document.createElement('canvas');
          div.appendChild(canvas);
          const r = Player.new(canvas, new Uint8Array(ab));
        };
        div.addEventListener('click', play, { once: true });
      });
    }).catch(error);
}

function scanForObjects() {
  document.querySelectorAll('[type="application/x-shockwave-flash"]')
    .forEach((object) => convertObject(object));
}

const observer = new MutationObserver((mutations) => {
  let scan = false;
  mutations.forEach((mutation) => {
    if (mutation.addedNodes.length > 0) {
      scan = true;
    }
  });
  if (scan) {
    scanForObjects();
  }
});

scanForObjects();

observer.observe(document.body, { attributes: false, childList: true, subtree: true });

`));
script.type = 'module';
document.body.appendChild(script);
