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

document.querySelectorAll('[type="application/x-shockwave-flash"]')
  .forEach((object) => {
    const source = object.getAttribute('data');
    if (!source) {
      return;
    }
    log('Replacing flash content from', source);
    fetch(new URL(source, window.location))
      .then((r) => r.arrayBuffer())
      .then((ab) => {
        const canvas = document.createElement('canvas');
        object.parentElement.appendChild(canvas);
        object.remove();
        return Ruffle.then((Player) => {
          const play = () => {
            log('Running', source);
            const r = Player.new(canvas, new Uint8Array(ab));
          };
          canvas.addEventListener('click', play);
          canvas.getContext('2d').fillText('Click to run flash content', 10, 10);
        });
      }).catch(error);
  });
`));
script.type = 'module';
document.body.appendChild(script);
