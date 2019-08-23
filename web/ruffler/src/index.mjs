/* eslint-env browser */
/* global __webpack_require__, browser */

const FLASH_MIME = 'application/x-shockwave-flash';

let Ruffle;

__webpack_require__.p = (typeof browser !== 'undefined' && browser.runtime && browser.runtime.getURL)
  ? browser.runtime.getURL('dist/')
  : 'https://ruffle.rs/ruffler';

const template = document.createElement('template');
template.innerHTML = `
<style>
:host {
  display: inline-block;
  background-color: #FFAD33;
}

#canvas {
  height: 100%;
  width: 100%;
}
</style>
<h1>\u{25B6} Click to play flash content</h1>
<canvas id="#canvas">
`;

class RufflePlayer extends HTMLElement {
  constructor(...args) {
    super(...args);

    this.shadow = this.attachShadow({ mode: 'closed' });
    this.shadow.appendChild(template.content.cloneNode(true));
    this.canvas = this.shadow.querySelector('#canvas');

    this.player = undefined;

    this.interactionPromise = new Promise((resolve) => {
      this.addEventListener('click', resolve, { once: true });
    });
  }

  play(data) {
    return this.interactionPromise
      .then(() => {
        if (this.player !== undefined) {
          this.player.destroy();
        }
        this.player = Ruffle.new(this.canvas, new Uint8Array(data));
      });
  }

  playFromURL(url) {
    return this.interactionPromise
      .then(() => fetch(url))
      .then((r) => r.arrayBuffer())
      .then((ab) => this.play(ab));
  }

  static handle(element, name) {
    if (element.type !== FLASH_MIME) {
      return;
    }
    const replacement = document.createElement(name);
    Array.from(element.attributes)
      .forEach((attribute) => {
        if (attribute.specified) {
          replacement.setAttribute(attribute.name, attribute.value);
        }
      });
    Array.from(element.children)
      .forEach((child) => {
        replacement.appendChild(child);
      });
    element.parentNode.replaceChild(replacement, element);
  }
}

export class RuffleObject extends RufflePlayer {
  constructor(...args) {
    super(...args);

    if (this.hasAttribute('data')) {
      this.play(this.getAttribute('data'));
    }
  }

  static handle(object) {
    return super.handle(object, 'ruffle-object');
  }
}

export class RuffleEmbed extends RufflePlayer {
  static handle(embed) {
    return super.handle(embed, 'ruffle-embed');
  }

  get src() {
    return this.attributes.src;
  }

  set src(v) {
    this.attributes.src = v;
  }

  attributeChangedCallback(name, oldValue, newValue) {
    if (name === 'src') {
      this.play(newValue);
    }
  }
}
RuffleEmbed.observedAttributes = ['src'];

customElements.define('ruffle-object', RuffleObject);
customElements.define('ruffle-embed', RuffleEmbed);

function scanForFlash() {
  // object scanning MUST BE FIRST, because
  // embeds can be subchildren of objects.
  Array.from(document.getElementsByTagName('object'))
    .forEach((object) => {
      RuffleObject.handle(object);
    });

  Array.from(document.getElementsByTagName('embed'))
    .forEach((embed) => {
      RuffleObject.handle(embed);
    });
}

import('../../pkg')
  .then((loaded) => {
    Ruffle = loaded;

    const observer = new MutationObserver((mutations) => {
      let needsScan = false;
      mutations.forEach((mutation) => {
        if (mutation.addedNodes.length > 0) {
          needsScan = true;
        }
      });
      if (needsScan) {
        scanForFlash();
      }
    });

    observer.observe(document, { attributes: false, childList: true, subtree: true });

    scanForFlash();
  });
