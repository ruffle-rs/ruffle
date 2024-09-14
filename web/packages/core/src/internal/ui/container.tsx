export function MainContainer() {
    return (
        <div id="container">
            <div id="play-button">
                <div class="icon">
                    <svg xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid" viewBox="0 0 250 250" width="100%" height="100%">
                        <defs xmlns="http://www.w3.org/2000/svg">
                            <linearGradient xmlns="http://www.w3.org/2000/svg" id="a" gradientUnits="userSpaceOnUse" x1="125" y1="0" x2="125" y2="250" spreadMethod="pad">
                                <stop xmlns="http://www.w3.org/2000/svg" offset="0%" stop-color="#FDA138"></stop>
                                <stop xmlns="http://www.w3.org/2000/svg" offset="100%" stop-color="#FD3A40"></stop>
                            </linearGradient>
                            <g xmlns="http://www.w3.org/2000/svg" id="b">
                                <path xmlns="http://www.w3.org/2000/svg" fill="url(#a)" d="M250 125q0-52-37-88-36-37-88-37T37 37Q0 73 0 125t37 88q36 37 88 37t88-37q37-36 37-88M87 195V55l100 70-100 70z"></path>
                                <path xmlns="http://www.w3.org/2000/svg" fill="#FFF" d="M87 55v140l100-70L87 55z"></path>
                            </g>
                        </defs>
                        <use xmlns="http://www.w3.org/2000/svg" href="#b"></use>
                    </svg>
                </div>
            </div>
            <div id="unmute-overlay">
                <div class="background"></div>
                <div class="icon">
                    <svg id="unmute-overlay-svg" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid" viewBox="0 0 512 584" width="100%" height="100%">
                        <path xmlns="http://www.w3.org/2000/svg" fill="#FFF" stroke="#FFF" d="m457.941 256 47.029-47.029c9.372-9.373 9.372-24.568 0-33.941-9.373-9.373-24.568-9.373-33.941 0l-47.029 47.029-47.029-47.029c-9.373-9.373-24.568-9.373-33.941 0-9.372 9.373-9.372 24.568 0 33.941l47.029 47.029-47.029 47.029c-9.372 9.373-9.372 24.568 0 33.941 4.686 4.687 10.827 7.03 16.97 7.03s12.284-2.343 16.971-7.029l47.029-47.03 47.029 47.029c4.687 4.687 10.828 7.03 16.971 7.03s12.284-2.343 16.971-7.029c9.372-9.373 9.372-24.568 0-33.941z"></path>
                        <path xmlns="http://www.w3.org/2000/svg" fill="#FFF" stroke="#FFF" d="m99 160h-55c-24.301 0-44 19.699-44 44v104c0 24.301 19.699 44 44 44h55c2.761 0 5-2.239 5-5v-182c0-2.761-2.239-5-5-5z"></path>
                        <path xmlns="http://www.w3.org/2000/svg" fill="#FFF" stroke="#FFF" d="m280 56h-24c-5.269 0-10.392 1.734-14.578 4.935l-103.459 79.116c-1.237.946-1.963 2.414-1.963 3.972v223.955c0 1.557.726 3.026 1.963 3.972l103.459 79.115c4.186 3.201 9.309 4.936 14.579 4.936h23.999c13.255 0 24-10.745 24-24v-352.001c0-13.255-10.745-24-24-24z"></path>
                        <text xmlns="http://www.w3.org/2000/svg" id="unmute-text" x="256" y="560" text-anchor="middle" font-size="60px" fill="#FFF" stroke="#FFF">
                            Click to unmute
                        </text>
                    </svg>
                </div>
            </div>
            <input aria-hidden="true" id="virtual-keyboard" type="text" autocomplete="off" autocorrect="off" autocapitalize="none" />
        </div>
    );
}
