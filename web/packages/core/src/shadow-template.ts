import { text } from "./i18n";

/**
 * Insert all rules from array in the style sheet.
 *
 * @param sheet The style sheet to which to apply the rules.
 * @param rules An array of rules to be applied.
 */
function insertRules(sheet: CSSStyleSheet, rules: Array<string>) {
    for (const rule of rules) {
        try {
            sheet.insertRule(rule);
        } catch (err) {
            // Ignore unsupported rules
        }
    }
}

/**
 * The default styles to apply to the shadow template.
 * This function must be run after the shadow template is added to the page.
 *
 * @param styleElement The static style element to which to add the rules
 */
export function applyStaticStyles(styleElement: HTMLStyleElement) {
    if (!styleElement.sheet) {
        return;
    }

    const rules = [
        `:host {
            all: initial;
            pointer-events: inherit;

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
        }`,

        /* Ruffle's width/height CSS interferes with Safari's fullscreen CSS. */
        /* Ensure that Safari's fullscreen mode actually fills the screen. */
        `:host(:-webkit-full-screen) {
            display: block;
            width: 100% !important;
            height: 100% !important;
        }`,

        `.hidden {
            display: none !important;
        }`,

        /* All of these use the dimensions specified by the embed. */
        `#container,
        #play-button,
        #unmute-overlay,
        #unmute-overlay .background,
        #panic,
        #splash-screen,
        #message-overlay {
            position: absolute;
            top: 0;
            bottom: 0;
            left: 0;
            right: 0;
        }`,

        `#container {
            overflow: hidden;
        }`,

        `#container canvas {
            width: 100%;
            height: 100%;
        }`,

        `#play-button,
        #unmute-overlay {
            cursor: pointer;
            display: none;
        }`,

        `#unmute-overlay .background {
            background: black;
            opacity: 0.7;
        }`,

        `#play-button .icon,
        #unmute-overlay .icon {
            position: absolute;
            top: 50%;
            left: 50%;
            width: 50%;
            height: 50%;
            max-width: 384px;
            max-height: 384px;
            transform: translate(-50%, -50%);
            opacity: 0.8;
        }`,

        `#play-button:hover .icon,
        #unmute-overlay:hover .icon {
            opacity: 1;
        }`,

        /* Includes inverted colors from play button! */
        `#panic {
            font-size: 20px;
            text-align: center;
            background: linear-gradient(180deg, #fd3a40 0%, #fda138 100%);
            color: white;
            display: flex;
            flex-flow: column;
            justify-content: space-around;
        }`,

        `#panic a {
            color: var(--ruffle-blue);
            font-weight: bold;
        }`,

        `#panic-title {
            font-size: xxx-large;
            font-weight: bold;
        }`,

        `#panic-body.details {
            flex: 0.9;
            margin: 0 10px;
        }`,

        `#panic-body textarea {
            width: 100%;
            height: 100%;
            resize: none;
        }`,

        `#panic ul {
            padding: 0;
            display: flex;
            list-style-type: none;
            justify-content: space-evenly;
        }`,

        `#message-overlay {
            position: absolute;
            background: var(--ruffle-blue);
            color: var(--ruffle-orange);
            opacity: 1;
            z-index: 2;
            display: flex;
            align-items: center;
            justify-content: center;
            overflow: auto;
        }`,

        `#message-overlay .message {
            text-align: center;
            max-height: 100%;
            max-width: 100%;
            padding: 5%;
            font-size: 20px;
        }`,

        `#message-overlay p {
            margin: 0.5em 0;
        }`,

        `#message-overlay .message div {
            display: flex;
            justify-content: center;
            flex-wrap: wrap;
            column-gap: 1em;
        }`,

        `#message-overlay a, #message-overlay button {
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
        }`,

        `#message-overlay a:hover, #message-overlay button:hover {
            background: #ffffff4c;
        }`,

        `#continue-btn {
             cursor: pointer;
             background: var(--ruffle-blue);
             color: var(--ruffle-orange);
             border: 2px solid var(--ruffle-orange);
             font-weight: bold;
             font-size: 20px;
             border-radius: 20px;
             padding: 10px;
        }`,

        `#continue-btn:hover {
            background: #ffffff4c;
        }`,

        `#context-menu-overlay, .modal {
            width: 100%;
            height: 100%;
            z-index: 1;
            position: absolute;
        }`,

        `#context-menu {
            color: rgb(var(--modal-foreground-rgb));
            background-color: var(--modal-background);
            border: 1px solid gray;
            box-shadow: 0px 5px 10px -5px black;
            position: absolute;
            font-size: 14px;
            text-align: left;
            list-style: none;
            white-space: nowrap;
            padding: 3px 0;
            margin: 0;
        }`,

        `#context-menu .menu-item {
            padding: 5px 10px;
            color: rgb(var(--modal-foreground-rgb));
        }`,

        `#context-menu .menu-item.disabled {
            cursor: default;
            color: rgba(var(--modal-foreground-rgb), 0.5);
        }`,

        `#context-menu .menu-item:not(.disabled):hover {
            background-color: rgba(var(--modal-foreground-rgb), 0.15);
        }`,

        `#context-menu .menu-separator hr {
            border: none;
            border-bottom: 1px solid rgba(var(--modal-foreground-rgb), 0.2);
            margin: 3px;
        }`,

        `#splash-screen {
            display: flex;
            flex-direction: column;
            background: var(--splash-screen-background, var(--preloader-background, var(--ruffle-blue)));
            align-items: center;
            justify-content: center;
        }`,

        `.loadbar {
            width: 100%;
            max-width: 316px;
            max-height: 10px;
            height: 20%;
            background: #253559;
        }`,

        `.loadbar-inner {
            width: 0px;
            max-width: 100%;
            height: 100%;
            background: var(--ruffle-orange);
        }`,

        `.logo {
            display: var(--logo-display, block);
            max-width: 380px;
            max-height: 150px;
        }`,

        `.loading-animation {
            max-width: 28px;
            max-height: 28px;
            margin-bottom: 2%;
            width: 10%;
            aspect-ratio: 1;
        }`,

        `.spinner {
            stroke-dasharray: 180;
            stroke-dashoffset: 135;
            stroke: var(--ruffle-orange);
            transform-origin: 50% 50%;
            animation: rotate 1.5s linear infinite;
        }`,

        `@keyframes rotate {
            to {
                transform: rotate(360deg);
            }
        }`,

        `#virtual-keyboard {
            position: absolute;
            opacity: 0;
            top: -100px;
            width: 1px;
            height: 1px;
        }`,

        `.modal {
            background-color: #0008;
        }`,

        `.modal-area {
            position: relative;
            left: 50%;
            transform: translateX(-50%);
            background-color: var(--modal-background);
            color: rgb(var(--modal-foreground-rgb));
            width: fit-content;
            padding: 8px 12px;
            border-radius: 12px;
            box-shadow: 0 2px 6px 0px #0008;
        }`,

        `#modal-area {
            width: 450px;
            height: 300px;
        }`,

        `.close-modal {
            width: 16px;
            height: 16px;
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='16px' viewBox='0 -960 960 960' width='16px' fill='black'%3E%3Cpath d='M480-392 300-212q-18 18-44 18t-44-18q-18-18-18-44t18-44l180-180-180-180q-18-18-18-44t18-44q18-18 44-18t44 18l180 180 180-180q18-18 44-18t44 18q18 18 18 44t-18 44L568-480l180 180q18 18 18 44t-18 44q-18 18-44 18t-44-18L480-392Z'/%3E%3C/svg%3E");
            cursor: pointer;
            filter: var(--modal-foreground-filter);
        }`,

        `.modal-button {
            display: inline-block;
            background-color: rgba(var(--modal-foreground-rgb), 0.2);
            color: rgb(var(--modal-foreground-rgb));
            text-decoration: none;
            padding: 4px 8px;
            border-radius: 6px;
            cursor: pointer;
        }`,

        `:not(#volume-controls) > .close-modal {
            position: absolute;
            top: 14px;
            right: 16px;
        }`,

        `.general-save-options {
            text-align: center;
            padding-bottom: 8px;
            border-bottom: 2px solid rgba(var(--modal-foreground-rgb), 0.3);
        }`,

        `#local-saves {
            color: inherit;
            border-collapse: collapse;
            overflow-y: auto;
            display: block;
            height: calc(100% - 45px);
            min-height: 30px;
        }`,

        `#local-saves td {
            border-bottom: 2px solid rgba(var(--modal-foreground-rgb), 0.15);
            height: 30px;
        }`,

        `#local-saves td:nth-child(1) {
            width: 100%;
            word-break: break-all;
        }`,

        `.save-option {
            display: inline-block;
            width: 24px;
            height: 24px;
            cursor: pointer;
            filter: var(--modal-foreground-filter);
            vertical-align: middle;
            opacity: 0.4;
        }`,

        `#local-saves > tr:hover .save-option {
            opacity: 1;
        }`,

        `#download-save {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='0 -960 960 960' width='24px' fill='black'%3E%3Cpath d='M480-337q-8 0-15-2.5t-13-8.5L308-492q-12-12-11.5-28t11.5-28q12-12 28.5-12.5T365-549l75 75v-286q0-17 11.5-28.5T480-800q17 0 28.5 11.5T520-760v286l75-75q12-12 28.5-11.5T652-548q11 12 11.5 28T652-492L508-348q-6 6-13 8.5t-15 2.5ZM240-160q-33 0-56.5-23.5T160-240v-80q0-17 11.5-28.5T200-360q17 0 28.5 11.5T240-320v80h480v-80q0-17 11.5-28.5T760-360q17 0 28.5 11.5T800-320v80q0 33-23.5 56.5T720-160H240Z'/%3E%3C/svg%3E");
        }`,

        `#replace-save {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='0 -1080 960 1200' width='24px' fill='black'%3E%3Cpath d='M440-367v127q0 17 11.5 28.5T480-200q17 0 28.5-11.5T520-240v-127l36 36q6 6 13.5 9t15 2.5q7.5-.5 14.5-3.5t13-9q11-12 11.5-28T612-388L508-492q-6-6-13-8.5t-15-2.5q-8 0-15 2.5t-13 8.5L348-388q-12 12-11.5 28t12.5 28q12 11 28 11.5t28-11.5l35-35ZM240-80q-33 0-56.5-23.5T160-160v-640q0-33 23.5-56.5T240-880h287q16 0 30.5 6t25.5 17l194 194q11 11 17 25.5t6 30.5v447q0 33-23.5 56.5T720-80H240Zm280-560q0 17 11.5 28.5T560-600h160L520-800v160Z'/%3E%3C/svg%3E");
        }`,

        `#delete-save {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='0 -1020 960 1080' width='24px' fill='black'%3E%3Cpath d='M280-120q-33 0-56.5-23.5T200-200v-520q-17 0-28.5-11.5T160-760q0-17 11.5-28.5T200-800h160q0-17 11.5-28.5T400-840h160q17 0 28.5 11.5T600-800h160q17 0 28.5 11.5T800-760q0 17-11.5 28.5T760-720v520q0 33-23.5 56.5T680-120H280Zm120-160q17 0 28.5-11.5T440-320v-280q0-17-11.5-28.5T400-640q-17 0-28.5 11.5T360-600v280q0 17 11.5 28.5T400-280Zm160 0q17 0 28.5-11.5T600-320v-280q0-17-11.5-28.5T560-640q-17 0-28.5 11.5T520-600v280q0 17 11.5 28.5T560-280Z'/%3E%3C/svg%3E");
        }`,

        `.replace-save {
            display: none;
        }`,

        `#video-modal .modal-area {
            width: 95%;
            height: 95%;
            box-sizing: border-box;
        }`,

        `#video-holder {
            height: 100%;
            box-sizing: border-box;
            padding: 36px 4px 6px;
        }`,

        `#video-holder video {
            width: 100%;
            height: 100%;
            background-color: black;
        }`,

        `#volume-controls {
            display: flex;
            align-items: center;
            gap: 6px;
        }`,

        `#mute-checkbox {
            display: none;
        }`,

        `label[for="mute-checkbox"] {
            width: 24px;
            height: 24px;
            line-height: 0;
            cursor: pointer;
            filter: var(--modal-foreground-filter);
        }`,

        `#volume-mute {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='0 -960 960 960' width='24px' fill='black'%3E%3Cpath d='m719.13-419.35-71.67 71.68Q634.78-335 617.13-335t-30.33-12.67q-12.67-12.68-12.67-30.33t12.67-30.33L658.48-480l-71.68-71.67q-12.67-12.68-12.67-30.33t12.67-30.33Q599.48-625 617.13-625t30.33 12.67l71.67 71.68 71.67-71.68Q803.48-625 821.13-625t30.33 12.67q12.67 12.68 12.67 30.33t-12.67 30.33L779.78-480l71.68 71.67q12.67 12.68 12.67 30.33t-12.67 30.33Q838.78-335 821.13-335t-30.33-12.67l-71.67-71.68ZM278-357.87H161.22q-17.66 0-30.33-12.67-12.67-12.68-12.67-30.33v-158.26q0-17.65 12.67-30.33 12.67-12.67 30.33-12.67H278l130.15-129.91q20.63-20.63 46.98-9.45 26.35 11.19 26.35 39.77v443.44q0 28.58-26.35 39.77-26.35 11.18-46.98-9.45L278-357.87Z'/%3E%3C/svg%3E");
        }`,

        `#volume-min {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='161 -960 960 960' width='24px' fill='black'%3E%3Cpath d='M438.65-357.87H321.87q-17.65 0-30.33-12.67-12.67-12.68-12.67-30.33v-158.26q0-17.65 12.67-30.33 12.68-12.67 30.33-12.67h116.78L568.8-732.04q20.63-20.63 46.98-9.45 26.35 11.19 26.35 39.77v443.44q0 28.58-26.35 39.77-26.35 11.18-46.98-9.45L438.65-357.87Z'/%3E%3C/svg%3E");
        }`,

        `#volume-mid {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='80 -960 960 960' width='24px' fill='black'%3E%3Cpath d='M357.98-357.87H241.2q-17.66 0-30.33-12.67-12.67-12.68-12.67-30.33v-158.26q0-17.65 12.67-30.33 12.67-12.67 30.33-12.67h116.78L487.65-731.8q20.63-20.64 47.1-9.57 26.47 11.07 26.47 39.65v443.44q0 28.58-26.47 39.65t-47.1-9.57L357.98-357.87ZM741.8-480q0 42.48-20.47 80.09-20.48 37.61-54.94 60.82-10.22 5.98-20.19.25-9.98-5.73-9.98-17.44v-248.44q0-11.71 9.98-17.32 9.97-5.61 20.19.37 34.46 23.71 54.94 61.45Q741.8-522.48 741.8-480Z'/%3E%3C/svg%3E");
        }`,

        `#volume-max {
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' height='24px' viewBox='9 -960 960 960' width='24px' fill='black'%3E%3Cpath d='M754.22-480.5q0-78.52-41.88-143.9-41.88-65.38-111.91-98.62-14.47-6.74-20.47-20.96-6-14.22-.53-28.93 5.74-15.72 20.34-22.46t29.58 0q92.48 42.46 147.97 127.05 55.48 84.6 55.48 187.82t-55.48 187.82q-55.49 84.59-147.97 127.05-14.98 6.74-29.58 0-14.6-6.74-20.34-22.46-5.47-14.71.53-28.93 6-14.22 20.47-20.96 70.03-33.24 111.91-98.62 41.88-65.38 41.88-143.9ZM286.98-357.87H170.2q-17.66 0-30.33-12.67-12.67-12.68-12.67-30.33v-158.26q0-17.65 12.67-30.33 12.67-12.67 30.33-12.67h116.78L416.65-731.8q20.63-20.64 47.1-9.57 26.47 11.07 26.47 39.65v443.44q0 28.58-26.47 39.65t-47.1-9.57L286.98-357.87ZM670.8-480q0 42.48-20.47 80.09-20.48 37.61-54.94 60.82-10.22 5.98-20.19.25-9.98-5.73-9.98-17.44v-248.44q0-11.71 9.98-17.32 9.97-5.61 20.19.37 34.46 23.71 54.94 61.45Q670.8-522.48 670.8-480Z'/%3E%3C/svg%3E");
        }`,

        `#volume-slider-text {
            width: 4.8ch;
            text-align: center;
            user-select: none;
        }`,

        `#hardware-acceleration-modal .modal-area {
            text-align: center;
            padding: 16px 48px;
            width: 95%;
            box-sizing: border-box;
        }`,

        `#acceleration-text {
            display: block;
            margin-bottom: 8px;
        }`,

        `#clipboard-modal h2 {
            margin-top: 4px;
            margin-right: 36px;
        }`,

        `#clipboard-modal p:last-child {
            margin-bottom: 2px;
        }`,

        /* Handle preferred color scheme. */
        `@media (prefers-color-scheme: light) {
            :host {
                --modal-background: #fafafa;
                --modal-foreground-rgb: 0, 0, 0;
                --modal-foreground-filter: none;
            }
        }`,

        `@media (prefers-color-scheme: dark) {
            :host {
                --modal-background: #282828;
                --modal-foreground-rgb: 221, 221, 221;
                --modal-foreground-filter: invert(90%);
            }
        }`,
    ];
    insertRules(styleElement.sheet, rules);
}

/**
 * Create and return a new HTML Element with the given arguments.
 *
 * @param tag The HTML tag name of the new element.
 * @param id The id of the new element.
 * @param className The class name of the new element.
 * @param attributes A hash of attributes for the new element.
 * @param ns The namespace of the new element.
 *
 * @returns The newly created Element
 */
function createElement(
    tag: string,
    id?: string,
    className?: string,
    attributes?: Record<string, string>,
    ns?: string,
): Element {
    const element = ns
        ? document.createElementNS(ns, tag)
        : document.createElement(tag);
    if (id) {
        element.id = id;
    }
    if (className && ns) {
        element.classList.add(className);
    } else if (className) {
        element.className = className;
    }
    if (attributes) {
        for (const [key, attr] of Object.entries(attributes)) {
            element.setAttribute(key, attr);
        }
    }
    return element;
}

/**
 * Create and return a new HTMLInputElement with the given arguments.
 *
 * @param htmlType The type of input element.
 * @param id The id of the input element.
 * @param min The min of the input element.
 * @param max The max of the input element.
 * @param step The step of the input element.
 *
 * @returns The newly created HTMLInputElement
 */
function createInputElement(
    htmlType: string,
    id: string,
    min?: string,
    max?: string,
    step?: string,
): HTMLInputElement {
    const element = createElement("input", id) as HTMLInputElement;
    element.type = htmlType;
    if (min) {
        element.min = min;
    }
    if (max) {
        element.max = max;
    }
    if (step) {
        element.step = step;
    }
    return element;
}

/**
 * Create and return a new HTMLLabelElement with the given arguments.
 *
 * @param id The id of the label element.
 * @param htmlFor The for of the label element.
 *
 * @returns The newly created HTMLLabelElement
 */
function createLabelElement(id: string, htmlFor: string): HTMLLabelElement {
    const element = createElement("label", id) as HTMLLabelElement;
    element.htmlFor = htmlFor;
    return element;
}

/**
 *
 * @param parentElement The node to which to append a child element.
 * @param childElement The node to be appended to the parent element.
 */
function appendElement(parentElement: Node, childElement: Node) {
    parentElement.appendChild(childElement);
}

/**
 * The shadow template which is used to fill the actual Ruffle player element
 * on the page.
 *
 */
export const ruffleShadowTemplate = document.createElement("template");
const svgns = "http://www.w3.org/2000/svg";
const staticStyles = createElement("style", "static-styles");
const dynamicStyles = createElement("style", "dynamic-styles");
const container = createElement("div", "container");

// Play button elements
const playButton = createElement("div", "play-button");
const playIcon = createElement("div", undefined, "icon");
const playSvg = createElement(
    "svg",
    undefined,
    undefined,
    {
        xmlns: svgns,
        "xmlns:xlink": "http://www.w3.org/1999/xlink",
        preserveAspectRatio: "xMidYMid",
        viewBox: "0 0 250 250",
        width: "100%",
        height: "100%",
    },
    svgns,
);
const playDefs = createElement("defs", undefined, undefined, undefined, svgns);
const playLinearGradient = createElement(
    "linearGradient",
    "a",
    undefined,
    {
        gradientUnits: "userSpaceOnUse",
        x1: "125",
        y1: "0",
        x2: "125",
        y2: "250",
        spreadMethod: "pad",
    },
    svgns,
);
const playStop0 = createElement(
    "stop",
    undefined,
    undefined,
    {
        offset: "0%",
        "stop-color": "#FDA138",
    },
    svgns,
);
const playStop100 = createElement(
    "stop",
    undefined,
    undefined,
    {
        offset: "100%",
        "stop-color": "#FD3A40",
    },
    svgns,
);
const playG = createElement("g", "b", undefined, undefined, svgns);
const playPath1 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "url(#a)",
        d: "M250 125q0-52-37-88-36-37-88-37T37 37Q0 73 0 125t37 88q36 37 88 37t88-37q37-36 37-88M87 195V55l100 70-100 70z",
    },
    svgns,
);
const playPath2 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "#FFF",
        d: "M87 55v140l100-70L87 55z",
    },
    svgns,
);
const playUse = document.createElementNS(svgns, "use");
playUse.href.baseVal = "#b";

// Unmute overlay elements
const unmuteOverlay = createElement("div", "unmute-overlay");
const background = createElement("div", undefined, "background");
const unmuteIcon = createElement("div", undefined, "icon");
const unmuteSvg = createElement(
    "svg",
    "unmute-overlay-svg",
    undefined,
    {
        xmlns: svgns,
        "xmlns:xlink": "http://www.w3.org/1999/xlink",
        preserveAspectRatio: "xMidYMid",
        viewBox: "0 0 512 584",
        width: "100%",
        height: "100%",
        scale: "0.8",
    },
    svgns,
);
const unmutePath1 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "#FFF",
        stroke: "#FFF",
        d: "m457.941 256 47.029-47.029c9.372-9.373 9.372-24.568 0-33.941-9.373-9.373-24.568-9.373-33.941 0l-47.029 47.029-47.029-47.029c-9.373-9.373-24.568-9.373-33.941 0-9.372 9.373-9.372 24.568 0 33.941l47.029 47.029-47.029 47.029c-9.372 9.373-9.372 24.568 0 33.941 4.686 4.687 10.827 7.03 16.97 7.03s12.284-2.343 16.971-7.029l47.029-47.03 47.029 47.029c4.687 4.687 10.828 7.03 16.971 7.03s12.284-2.343 16.971-7.029c9.372-9.373 9.372-24.568 0-33.941z",
    },
    svgns,
);
const unmutePath2 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "#FFF",
        stroke: "#FFF",
        d: "m99 160h-55c-24.301 0-44 19.699-44 44v104c0 24.301 19.699 44 44 44h55c2.761 0 5-2.239 5-5v-182c0-2.761-2.239-5-5-5z",
    },
    svgns,
);
const unmutePath3 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "#FFF",
        stroke: "#FFF",
        d: "m280 56h-24c-5.269 0-10.392 1.734-14.578 4.935l-103.459 79.116c-1.237.946-1.963 2.414-1.963 3.972v223.955c0 1.557.726 3.026 1.963 3.972l103.459 79.115c4.186 3.201 9.309 4.936 14.579 4.936h23.999c13.255 0 24-10.745 24-24v-352.001c0-13.255-10.745-24-24-24z",
    },
    svgns,
);
const unmuteText = createElement(
    "text",
    "unmute-text",
    undefined,
    {
        x: "256",
        y: "560",
        "text-anchor": "middle",
        "font-size": "60px",
        fill: "#FFF",
        stroke: "#FFF",
    },
    svgns,
);

// Virtual keyboard element
const virtualKeyboard = createElement("input", "virtual-keyboard", undefined, {
    type: "text",
    autocapitalize: "off",
    autocomplete: "off",
    autocorrect: "off",
});

// Splash screen elements
const splashScreen = createElement("div", "splash-screen", "hidden");
const splashScreenSvg = createElement(
    "svg",
    undefined,
    "logo",
    {
        xmlns: svgns,
        "xmlns:xlink": "http://www.w3.org/1999/xlink",
        preserveAspectRatio: "xMidYMid",
        viewBox: "0 0 380 150",
    },
    svgns,
);
const splashScreenG = createElement(
    "g",
    undefined,
    undefined,
    undefined,
    svgns,
);
const splashScreenPath1 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "#966214",
        d: "M58.75 85.6q.75-.1 1.5-.35.85-.25 1.65-.75.55-.35 1.05-.8.5-.45.95-1 .5-.5.75-1.2-.05.05-.15.1-.1.15-.25.25l-.1.2q-.15.05-.25.1-.4 0-.8.05-.5-.25-.9-.5-.3-.1-.55-.3l-.6-.6-4.25-6.45-1.5 11.25h3.45m83.15-.2h3.45q.75-.1 1.5-.35.25-.05.45-.15.35-.15.65-.3l.5-.3q.25-.15.5-.35.45-.35.9-.75.45-.35.75-.85l.1-.1q.1-.2.2-.35.2-.3.35-.6l-.3.4-.15.15q-.5.15-1.1.1-.25 0-.4-.05-.5-.15-.8-.4-.15-.1-.25-.25-.3-.3-.55-.6l-.05-.05v-.05l-4.25-6.4-1.5 11.25m-21.15-3.95q-.3-.3-.55-.6l-.05-.05v-.05l-4.25-6.4-1.5 11.25h3.45q.75-.1 1.5-.35.85-.25 1.6-.75.75-.5 1.4-1.1.45-.35.75-.85.35-.5.65-1.05l-.45.55q-.5.15-1.1.1-.9 0-1.45-.7m59.15.3q-.75-.5-1.4-1-3.15-2.55-3.5-6.4l-1.5 11.25h21q-3.1-.25-5.7-.75-5.6-1.05-8.9-3.1m94.2 3.85h3.45q.6-.1 1.2-.3.4-.1.75-.2.35-.15.65-.3.7-.35 1.35-.8.75-.55 1.3-1.25.1-.15.25-.3-2.55-.25-3.25-1.8l-4.2-6.3-1.5 11.25m-45.3-4.85q-.5-.4-.9-.8-2.3-2.35-2.6-5.6l-1.5 11.25h21q-11.25-.95-16-4.85m97.7 4.85q-.3-.05-.6-.05-10.8-1-15.4-4.8-3.15-2.55-3.5-6.35l-1.5 11.2h21Z",
    },
    svgns,
);
const splashScreenPath2 = createElement(
    "path",
    undefined,
    undefined,
    {
        fill: "var(--ruffle-orange)",
        d: "M92.6 54.8q-1.95-1.4-4.5-1.4H60.35q-1.35 0-2.6.45-1.65.55-3.15 1.8-2.75 2.25-3.25 5.25l-1.65 12h.05v.3l5.85 1.15h-9.5q-.5.05-1 .15-.5.15-1 .35-.5.2-.95.45-.5.3-.95.7-.45.35-.85.8-.35.4-.65.85-.3.45-.5.9-.15.45-.3.95l-5.85 41.6H50.3l5-35.5 1.5-11.25 4.25 6.45.6.6q.25.2.55.3.4.25.9.5.4-.05.8-.05.1-.05.25-.1l.1-.2q.15-.1.25-.25.1-.05.15-.1l.3-1.05 1.75-12.3h11.15L75.8 82.6h16.5l2.3-16.25h-.05l.8-5.7q.4-2.45-1-4.2-.35-.4-.75-.8-.25-.25-.55-.5-.2-.2-.45-.35m16.2 18.1h.05l-.05.3 5.85 1.15H105.2q-.5.05-1 .15-.5.15-1 .35-.5.2-.95.45-.5.3-1 .65-.4.4-.8.85-.25.3-.55.65-.05.1-.15.2-.25.45-.4.9-.2.45-.3.95-.1.65-.2 1.25-.2 1.15-.4 2.25l-4.3 30.6q-.25 3 1.75 5.25 1.6 1.8 4 2.15.6.1 1.25.1h27.35q3.25 0 6-2.25.35-.35.7-.55l.3-.2q2-2 2.25-4.5l1.65-11.6q.05-.05.1-.05l1.65-11.35h.05l.7-5.2 1.5-11.25 4.25 6.4v.05l.05.05q.25.3.55.6.1.15.25.25.3.25.8.4.15.05.4.05.6.05 1.1-.1l.15-.15.3-.4.3-1.05 1.3-9.05h-.05l.7-5.05h-.05l.15-1.25h-.05l1.65-11.7h-16.25l-2.65 19.5h.05v.2l-.05.1h.05l5.8 1.15H132.7q-.5.05-1 .15-.5.15-1 .35-.15.05-.3.15-.3.1-.55.25-.05 0-.1.05-.5.3-1 .65-.4.35-.7.7-.55.7-.95 1.45-.35.65-.55 1.4-.15.7-.25 1.4v.05q-.15 1.05-.35 2.05l-1.2 8.75v.1l-2.1 14.7H111.4l2.25-15.55h.05l.7-5.2 1.5-11.25 4.25 6.4v.05l.05.05q.25.3.55.6.55.7 1.45.7.6.05 1.1-.1l.45-.55.3-1.05 1.3-9.05h-.05l.7-5.05h-.05l.15-1.25h-.05l1.65-11.7h-16.25l-2.65 19.5m106.5-41.75q-2.25-2.25-5.5-2.25h-27.75q-3 0-5.75 2.25-1.3.95-2.05 2.1-.45.6-.7 1.2-.2.5-.35 1-.1.45-.15.95l-4.15 29.95h-.05l-.7 5.2h-.05l-.2 1.35h.05l-.05.3 5.85 1.15h-9.45q-2.1.05-3.95 1.6-1.9 1.55-2.25 3.55l-.5 3.5h-.05l-5.3 38.1h16.25l5-35.5 1.5-11.25q.35 3.85 3.5 6.4.65.5 1.4 1 3.3 2.05 8.9 3.1 2.6.5 5.7.75l1.75-11.25h-12.2l.4-2.95h-.05l.7-5.05h-.05q.1-.9.3-1.9.1-.75.2-1.6.85-5.9 2.15-14.9 0-.15.05-.25l.1-.9q.2-1.55.45-3.15h11.25l-3.1 20.8h16.5l4.1-28.05q.15-1.7-.4-3.15-.5-1.1-1.35-2.1m46.65 44.15q-.5.3-1 .65-.4.4-.8.85-.35.4-.7.85-.25.45-.45.9-.15.45-.3.95l-5.85 41.6h16.25l5-35.5 1.5-11.25 4.2 6.3q.7 1.55 3.25 1.8l.05-.1q.25-.4.35-.85l.3-1.05 1.8-14.05v-.05l5.35-37.45h-16.25l-6.15 44.3 5.85 1.15h-9.45q-.5.05-1 .15-.5.15-1 .35-.5.2-.95.45m5.4-38.9q.15-1.7-.4-3.15-.5-1.1-1.35-2.1-2.25-2.25-5.5-2.25h-27.75q-2.3 0-4.45 1.35-.65.35-1.3.9-1.3.95-2.05 2.1-.45.6-.7 1.2-.4.9-.5 1.95l-4.15 29.95h-.05l-.7 5.2h-.05l-.2 1.35h.05l-.05.3 5.85 1.15h-9.45q-2.1.05-3.95 1.6-1.9 1.55-2.25 3.55l-.5 3.5h-.05l-1.2 8.75v.1l-4.1 29.25h16.25l5-35.5 1.5-11.25q.3 3.25 2.6 5.6.4.4.9.8 4.75 3.9 16 4.85l1.75-11.25h-12.2l.4-2.95h-.05l.7-5.05h-.05q.15-.9.3-1.9.1-.75.25-1.6.15-1.25.35-2.65v-.05q.95-6.7 2.35-16.5h11.25l-3.1 20.8h16.5l4.1-28.05M345 66.35h-.05l1.15-8.2q.5-3-1.75-5.25-1.25-1.25-3-1.75-1-.5-2.25-.5h-27.95q-.65 0-1.3.1-2.5.35-4.7 2.15-2.75 2.25-3.25 5.25l-1.95 14.7v.05l-.05.3 5.85 1.15h-9.45q-1.9.05-3.6 1.35-.2.1-.35.25-1.9 1.55-2.25 3.55l-4.85 34.1q-.25 3 1.75 5.25 1.25 1.4 3 1.95 1.05.3 2.25.3H320q3.25 0 6-2.25 2.75-2 3.25-5l2.75-18.5h-16.5l-1.75 11H302.5l2.1-14.75h.05l.85-6 1.5-11.2q.35 3.8 3.5 6.35 4.6 3.8 15.4 4.8.3 0 .6.05h15.75L345 66.35m-16.4-.95-1.25 8.95h-11.3l.4-2.95h-.05l.7-5.05h-.1l.15-.95h11.45Z",
    },
    svgns,
);
const loadingAnimation = createElement(
    "svg",
    undefined,
    "loading-animation",
    {
        xmlns: svgns,
        viewBox: "0 0 66 66",
    },
    svgns,
);
const spinner = createElement(
    "circle",
    undefined,
    "spinner",
    {
        fill: "none",
        "stroke-width": "6",
        "stroke-linecap": "round",
        cx: "33",
        cy: "33",
        r: "30",
    },
    svgns,
);
const loadbar = createElement("div", undefined, "loadbar");
const loadbarInner = createElement("div", undefined, "loadbar-inner");

// Save manager elements
const saveManager = createElement("div", "save-manager", "modal hidden");
const saveModalArea = createElement("div", "modal-area", "modal-area");
const saveModalClose = createElement("span", undefined, "close-modal");
const generalSaveOptions = createElement(
    "div",
    undefined,
    "general-save-options",
);
const backupSaves = createElement("span", undefined, "modal-button");
const localSaves = createElement("table", "local-saves");

// Volume control elements
const volumeControlsModal = createElement(
    "div",
    "volume-controls-modal",
    "modal hidden",
);
const volumeModalArea = createElement("div", undefined, "modal-area");
const volumeControls = createElement("div", "volume-controls");
const volumeMuteCheckbox = createInputElement("checkbox", "mute-checkbox");
const volumeSlider = createInputElement(
    "range",
    "volume-slider",
    "0",
    "100",
    "1",
);
const volumeMuteIcon = createLabelElement("volume-mute", "mute-checkbox");
volumeMuteIcon.title = text("volume-controls-unmute");
const volumeMinIcon = createLabelElement("volume-min", "mute-checkbox");
const volumeMidIcon = createLabelElement("volume-mid", "mute-checkbox");
const volumeMaxIcon = createLabelElement("volume-max", "mute-checkbox");
[volumeMinIcon, volumeMidIcon, volumeMaxIcon].forEach(
    (icon) => (icon.title = text("volume-controls-mute")),
);
const volumeSliderText = createElement("span", "volume-slider-text");
const volumeModalClose = createElement("span", undefined, "close-modal");

// Video modal elements
const videoModal = createElement("div", "video-modal", "modal hidden");
const videoModalArea = createElement("div", undefined, "modal-area");
const videoModalClose = createElement("span", undefined, "close-modal");
const videoHolder = createElement("div", "video-holder");

// Hardware acceleration modal elements
const hardwareModal = createElement(
    "div",
    "hardware-acceleration-modal",
    "modal hidden",
);
const hardwareModalArea = createElement("div", undefined, "modal-area");
const hardwareModalClose = createElement("span", undefined, "close-modal");
const hardwareModalText = createElement("span", "acceleration-text");
hardwareModalText.textContent = text("enable-hardware-acceleration");
const hardwareModalLink = document.createElement("a");
hardwareModalLink.href =
    "https://github.com/ruffle-rs/ruffle/wiki/Frequently-Asked-Questions-For-Users#chrome-hardware-acceleration";
hardwareModalLink.target = "_blank";
hardwareModalLink.className = "modal-button";
hardwareModalLink.textContent = text("enable-hardware-acceleration-link");

// Clipboard message
const clipboardModal = createElement("div", "clipboard-modal", "modal hidden");
const clipboardModalArea = createElement("div", undefined, "modal-area");
const clipboardModalClose = createElement("span", undefined, "close-modal");
const clipboardModalHeading = createElement("h2", undefined);
clipboardModalHeading.textContent = text("clipboard-message-title");
const clipboardModalTextDescription = createElement(
    "p",
    "clipboard-modal-description",
);
const shortcutModifier = navigator.userAgent.includes("Mac OS X")
    ? "Command"
    : "Ctrl";
const clipboardModalTextCopy = createElement("p", undefined);
const clipboardModalTextCopyShortcut = createElement("b", undefined);
clipboardModalTextCopyShortcut.textContent = `${shortcutModifier}+C`;
const clipboardModalTextCopyText = createElement("span", undefined);
clipboardModalTextCopyText.textContent = text("clipboard-message-copy");
const clipboardModalTextCut = createElement("p", undefined);
const clipboardModalTextCutShortcut = createElement("b", undefined);
clipboardModalTextCutShortcut.textContent = `${shortcutModifier}+X`;
const clipboardModalTextCutText = createElement("span", undefined);
clipboardModalTextCutText.textContent = text("clipboard-message-cut");
const clipboardModalTextPaste = createElement("p", undefined);
const clipboardModalTextPasteShortcut = createElement("b", undefined);
clipboardModalTextPasteShortcut.textContent = `${shortcutModifier}+V`;
const clipboardModalTextPasteText = createElement("span", undefined);
clipboardModalTextPasteText.textContent = text("clipboard-message-paste");

// Context menu overlay elements
const contextMenuOverlay = createElement(
    "div",
    "context-menu-overlay",
    "hidden",
);
const contextMenu = createElement("ul", "context-menu");

appendElement(ruffleShadowTemplate.content, staticStyles);
appendElement(ruffleShadowTemplate.content, dynamicStyles);
appendElement(ruffleShadowTemplate.content, container);
// Play button append
appendElement(container, playButton);
appendElement(playButton, playIcon);
appendElement(playIcon, playSvg);
appendElement(playSvg, playDefs);
appendElement(playDefs, playLinearGradient);
appendElement(playLinearGradient, playStop0);
appendElement(playLinearGradient, playStop100);
appendElement(playDefs, playG);
appendElement(playG, playPath1);
appendElement(playG, playPath2);
appendElement(playSvg, playUse);
// Unmute overlay append
appendElement(container, unmuteOverlay);
appendElement(unmuteOverlay, background);
appendElement(unmuteOverlay, unmuteIcon);
appendElement(unmuteIcon, unmuteSvg);
appendElement(unmuteSvg, unmutePath1);
appendElement(unmuteSvg, unmutePath2);
appendElement(unmuteSvg, unmutePath3);
appendElement(unmuteSvg, unmuteText);
// Virtual keyboard append
appendElement(container, virtualKeyboard);
// Splash screen append
appendElement(ruffleShadowTemplate.content, splashScreen);
appendElement(splashScreen, splashScreenSvg);
appendElement(splashScreenSvg, splashScreenG);
appendElement(splashScreenG, splashScreenPath1);
appendElement(splashScreenG, splashScreenPath2);
appendElement(splashScreen, loadingAnimation);
appendElement(loadingAnimation, spinner);
appendElement(splashScreen, loadbar);
appendElement(loadbar, loadbarInner);
// Save manager append
appendElement(ruffleShadowTemplate.content, saveManager);
appendElement(saveManager, saveModalArea);
appendElement(saveModalArea, saveModalClose);
appendElement(saveModalArea, generalSaveOptions);
appendElement(generalSaveOptions, backupSaves);
appendElement(saveModalArea, localSaves);
// Volume control append
appendElement(ruffleShadowTemplate.content, volumeControlsModal);
appendElement(volumeControlsModal, volumeModalArea);
appendElement(volumeModalArea, volumeControls);
appendElement(volumeControls, volumeMuteCheckbox);
appendElement(volumeControls, volumeMuteIcon);
appendElement(volumeControls, volumeMinIcon);
appendElement(volumeControls, volumeMidIcon);
appendElement(volumeControls, volumeMaxIcon);
appendElement(volumeControls, volumeSlider);
appendElement(volumeControls, volumeSliderText);
appendElement(volumeControls, volumeModalClose);
// Video modal append
appendElement(ruffleShadowTemplate.content, videoModal);
appendElement(videoModal, videoModalArea);
appendElement(videoModalArea, videoModalClose);
appendElement(videoModalArea, videoHolder);
// Hardware acceleration modal append
appendElement(ruffleShadowTemplate.content, hardwareModal);
appendElement(hardwareModal, hardwareModalArea);
appendElement(hardwareModalArea, hardwareModalClose);
appendElement(hardwareModalArea, hardwareModalText);
appendElement(hardwareModalArea, hardwareModalLink);
// Clipboard modal append
appendElement(ruffleShadowTemplate.content, clipboardModal);
appendElement(clipboardModal, clipboardModalArea);
appendElement(clipboardModalArea, clipboardModalClose);
appendElement(clipboardModalArea, clipboardModalHeading);
appendElement(clipboardModalArea, clipboardModalTextDescription);
appendElement(clipboardModalArea, clipboardModalTextCopy);
appendElement(clipboardModalTextCopy, clipboardModalTextCopyShortcut);
appendElement(clipboardModalTextCopy, clipboardModalTextCopyText);
appendElement(clipboardModalArea, clipboardModalTextCut);
appendElement(clipboardModalTextCut, clipboardModalTextCutShortcut);
appendElement(clipboardModalTextCut, clipboardModalTextCutText);
appendElement(clipboardModalArea, clipboardModalTextPaste);
appendElement(clipboardModalTextPaste, clipboardModalTextPasteShortcut);
appendElement(clipboardModalTextPaste, clipboardModalTextPasteText);
// Context menu overlay append
appendElement(ruffleShadowTemplate.content, contextMenuOverlay);
appendElement(contextMenuOverlay, contextMenu);
