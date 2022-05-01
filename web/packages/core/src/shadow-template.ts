/**
 * The shadow template which is used to fill the actual Ruffle player element
 * on the page.
 */
export const ruffleShadowTemplate = document.createElement("template");
ruffleShadowTemplate.innerHTML = `
    <style>
        :host {
            --ruffle-blue: #37528c;
            --ruffle-orange: #ffad33;

            display: inline-block;
            position: relative;
            /* Default width/height; this will get overridden by user styles/attributes. */
            width: 550px;
            height: 400px;
            font-family: Arial, sans-serif;
            letter-spacing: 0.4px;
            touch-action: none;
            user-select: none;
            -webkit-user-select: none;
            -webkit-tap-highlight-color: transparent;
        }

        /* Ruffle's width/height CSS interferes Safari fullscreen CSS. */
        /* Ensure that Safari's fullscreen mode actually fills the screen. */
        :host(:-webkit-full-screen) {
            display: block;
            width: 100% !important;
            height: 100% !important;
        }

        /* All of these use the dimensions specified by the embed. */
        #container,
        #play_button,
        #unmute_overlay,
        #unmute_overlay .background,
        #panic,
        #message_overlay {
            position: absolute;
            top: 0;
            bottom: 0;
            left: 0;
            right: 0;
        }

        #container {
            overflow: hidden;
        }

        #container canvas {
            width: 100%;
            height: 100%;
        }

        #play_button,
        #unmute_overlay {
            cursor: pointer;
            display: none;
        }

        #unmute_overlay .background {
            background: black;
            opacity: 0.7;
        }

        #play_button .icon,
        #unmute_overlay .icon {
            position: absolute;
            top: 50%;
            left: 50%;
            width: 50%;
            height: 50%;
            max-width: 384px;
            max-height: 384px;
            transform: translate(-50%, -50%);
            opacity: 0.8;
        }

        #play_button:hover .icon,
        #unmute_overlay:hover .icon {
            opacity: 1;
        }

        #panic {
            font-size: 20px;
            text-align: center;
            /* Inverted colors from play button! */
            background: linear-gradient(180deg, #fd3a40 0%, #fda138 100%);
            color: white;
            display: flex;
            flex-flow: column;
            justify-content: space-around;
        }

        #panic a {
            color: var(--ruffle-blue);
            font-weight: bold;
        }

        #panic-title {
            font-size: xxx-large;
            font-weight: bold;
        }

        #panic-body.details {
            flex: 0.9;
            margin: 0 10px;
        }

        #panic-body textarea {
            width: 100%;
            height: 100%;
            resize: none;
        }

        #panic ul {
            padding: 0;
            display: flex;
            list-style-type: none;
            justify-content: space-evenly;
        }

        #message_overlay {
            position: absolute;
            background: var(--ruffle-blue);
            color: var(--ruffle-orange);
            opacity: 1;
            z-index: 2;
            display: flex;
            align-items: center;
            justify-content: center;
            overflow: auto;
        }

        #message_overlay .message {
            text-align: center;
            max-height: 100%;
            max-width: 100%;
            padding: 5%;
        }

        #message_overlay p {
            margin: 0.5em 0;
        }

        #message_overlay .message div {
            display: flex;
            justify-content: center;
            flex-wrap: wrap;
            column-gap: 1em;
        }

        #message_overlay a, #message_overlay button {
            cursor: pointer;
            background: var(--ruffle-blue);
            color: var(--ruffle-orange);
            border: 2px solid var(--ruffle-orange);
            font-weight: bold;
            font-size: 1.25em;
            border-radius: 0.6em;
            padding: 10px;
            text-decoration: none;
            margin: 2% 0;
        }

        #message_overlay a:hover, #message_overlay button:hover {
            background: #ffffff4c;
        }

        #continue-btn {
             cursor: pointer;
             background: var(--ruffle-blue);
             color: var(--ruffle-orange);
             border: 2px solid var(--ruffle-orange);
             font-weight: bold;
             font-size: 20px;
             border-radius: 20px;
             padding: 10px;
        }

        #continue-btn:hover {
            background: #ffffff4c;
        }

        #context-menu {
            display: none;
            color: black;
            background: #fafafa;
            border: 1px solid gray;
            box-shadow: 0px 5px 10px -5px black;
            position: absolute;
            font-size: 14px;
            text-align: left;
            list-style: none;
            padding: 0;
            margin: 0;
        }

        #context-menu .menu_item {
            padding: 5px 10px;
            cursor: pointer;
            color: black;
        }

        #context-menu .menu_item.disabled {
            cursor: default;
            color: gray;
        }

        #context-menu .menu_item:not(.disabled):hover {
            background: lightgray;
        }

        #context-menu .menu_separator hr {
            border: none;
            border-bottom: 1px solid lightgray;
            margin: 2px;
        }
    </style>
    <style id="dynamic_styles"></style>

    <div id="container">
        <div id="play_button"><div class="icon"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid" viewBox="0 0 250 250" width="100%" height="100%"><defs><linearGradient id="a" gradientUnits="userSpaceOnUse" x1="125" y1="0" x2="125" y2="250" spreadMethod="pad"><stop offset="0%" stop-color="#FDA138"/><stop offset="100%" stop-color="#FD3A40"/></linearGradient><g id="b"><path fill="url(#a)" d="M250 125q0-52-37-88-36-37-88-37T37 37Q0 73 0 125t37 88q36 37 88 37t88-37q37-36 37-88M87 195V55l100 70-100 70z"/><path fill="#FFF" d="M87 55v140l100-70L87 55z"/></g></defs><use xlink:href="#b"/></svg></div></div>
        <div id="unmute_overlay"><div class="background"></div><div class="icon"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid" viewBox="0 0 512 584"  width="100%" height="100%" scale="0.8"><path fill="#FFF" stroke="#FFF" d="m457.941 256 47.029-47.029c9.372-9.373 9.372-24.568 0-33.941-9.373-9.373-24.568-9.373-33.941 0l-47.029 47.029-47.029-47.029c-9.373-9.373-24.568-9.373-33.941 0-9.372 9.373-9.372 24.568 0 33.941l47.029 47.029-47.029 47.029c-9.372 9.373-9.372 24.568 0 33.941 4.686 4.687 10.827 7.03 16.97 7.03s12.284-2.343 16.971-7.029l47.029-47.03 47.029 47.029c4.687 4.687 10.828 7.03 16.971 7.03s12.284-2.343 16.971-7.029c9.372-9.373 9.372-24.568 0-33.941z"/><path fill="#FFF" stroke="#FFF" d="m99 160h-55c-24.301 0-44 19.699-44 44v104c0 24.301 19.699 44 44 44h55c2.761 0 5-2.239 5-5v-182c0-2.761-2.239-5-5-5z"/><path fill="#FFF" stroke="#FFF" d="m280 56h-24c-5.269 0-10.392 1.734-14.578 4.935l-103.459 79.116c-1.237.946-1.963 2.414-1.963 3.972v223.955c0 1.557.726 3.026 1.963 3.972l103.459 79.115c4.186 3.201 9.309 4.936 14.579 4.936h23.999c13.255 0 24-10.745 24-24v-352.001c0-13.255-10.745-24-24-24z"/><text x="256" y="560" text-anchor="middle" font-size="60px" fill="#FFF" stroke="#FFF">Click to unmute</text></svg></div></div>
    </div>

    <ul id="context-menu"></ul>
`;
