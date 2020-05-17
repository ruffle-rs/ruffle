let ruffle_shadow_tmpl = document.createElement("template");
ruffle_shadow_tmpl.innerHTML = `
    <style>
        :host {
            display: inline-block;
            /* Default width/height; this will get overridden by user styles/attributes */
            width: 550px;
            height: 400px;
        }

        #container {
            position: relative;
            width: 100%;
            height: 100%;
            overflow: hidden;
        }

        #container canvas {
            width: 100%;
            height: 100%;
        }
        
        #play_button {
            position: absolute;
            width: 100%;
            height: 100%;
            cursor: pointer;
            display: none;
        }

        #play_button .icon {
            position: relative;
            top: 50%;
            left: 50%;
            width: 90%;
            height: 90%;
            max-width: 500px;
            max-height: 500px;
            transform: translate(-50%, -50%);
        }

        #play_button:hover .icon {
            filter: brightness(1.3);
        }
    </style>
    <style id="dynamic_styles"></style>

    <div id="container">
        <div id="play_button"><div class="icon"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid" viewBox="0 0 250 250" style="width:100%;height:100%;"><defs><linearGradient id="a" gradientUnits="userSpaceOnUse" x1="125" y1="0" x2="125" y2="250" spreadMethod="pad"><stop offset="0%" stop-color="#FDA138"/><stop offset="100%" stop-color="#FD3A40"/></linearGradient><g id="b"><path fill="url(#a)" d="M250 125q0-52-37-88-36-37-88-37T37 37Q0 73 0 125t37 88q36 37 88 37t88-37q37-36 37-88M87 195V55l100 70-100 70z"/><path fill="#FFF" d="M87 55v140l100-70L87 55z"/></g></defs><use xlink:href="#b"/></svg></div></div>
    </div>
`;

module.exports = ruffle_shadow_tmpl;
