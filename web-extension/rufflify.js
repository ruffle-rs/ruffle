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
  const canvas = document.createElement('canvas');
  object.parentElement.insertBefore(canvas, object);
  object.remove();
  fetch(new URL(source, window.location))
    .then((r) => r.arrayBuffer())
    .then((ab) => {
      return Ruffle.then((Player) => {
        const play = () => {
          log('Running', source);
          const r = Player.new(canvas, new Uint8Array(ab));
        };
        canvas.addEventListener('click', play, { once: true });
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = '#FFAD33';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = 'black';
        ctx.fillText('Click to run flash content in Ruffle', 10, 10);
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
