let ruffle_shadow_tmpl = document.createElement("template");
ruffle_shadow_tmpl.innerHTML = `
    <style>
        :host {
            display: inline-block;
        }

        #container {
            position: relative;
            width: 100%;
            height: 100%;
            overflow: hidden;
        }

        #player {
            width: 100%;
            height: 100%;
        }
        
        #play_button {
            position: relative;
            width: 100%;
            height: 100%;
            cursor: pointer;
            display: none;
        }

        #play_button .icon {
            position: relative;
            background-image: url("data:image/svg+xml;base64,CjxzdmcgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB2ZXJzaW9uPSIxLjEiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiBwcmVzZXJ2ZUFzcGVjdFJhdGlvPSJ4TWlkWU1pZCIgeD0iMHB4IiB5PSIwcHgiIHdpZHRoPSIyNTBweCIgaGVpZ2h0PSIyNTBweCIgdmlld0JveD0iMCAwIDI1MCAyNTAiPgo8ZGVmcz4KPGxpbmVhckdyYWRpZW50IGlkPSJHcmFkaWVudF8xIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgeDE9IjEyNSIgeTE9IjAiIHgyPSIxMjUiIHkyPSIyNTAiIHNwcmVhZE1ldGhvZD0icGFkIj4KPHN0b3AgIG9mZnNldD0iMCUiIHN0b3AtY29sb3I9IiNGREExMzgiLz4KCjxzdG9wICBvZmZzZXQ9IjEwMCUiIHN0b3AtY29sb3I9IiNGRDNBNDAiLz4KPC9saW5lYXJHcmFkaWVudD4KCjxnIGlkPSJMYXllcjBfMF9GSUxMIj4KPHBhdGggZmlsbD0idXJsKCNHcmFkaWVudF8xKSIgc3Ryb2tlPSJub25lIiBkPSIKTSAyNTAgMTI1ClEgMjUwIDczLjIgMjEzLjM1IDM2LjYgMTc2LjggMCAxMjUgMCA3My4yIDAgMzYuNjUgMzYuNiAwIDczLjIgMCAxMjUgMCAxNzYuOCAzNi42NSAyMTMuMzUgNzMuMiAyNTAgMTI1IDI1MCAxNzYuOCAyNTAgMjEzLjM1IDIxMy4zNSAyNTAgMTc2LjggMjUwIDEyNQpNIDg3IDE5NQpMIDg3IDU1IDE4NyAxMjUgODcgMTk1IFoiLz4KCjxwYXRoIGZpbGw9IiNGRkZGRkYiIHN0cm9rZT0ibm9uZSIgZD0iCk0gODcgNTUKTCA4NyAxOTUgMTg3IDEyNSA4NyA1NSBaIi8+CjwvZz4KPC9kZWZzPgoKPGcgdHJhbnNmb3JtPSJtYXRyaXgoIDEsIDAsIDAsIDEsIDAsMCkgIj4KPHVzZSB4bGluazpocmVmPSIjTGF5ZXIwXzBfRklMTCIvPgo8L2c+Cjwvc3ZnPgo=");
            background-size: 100% 100%;
            top: 50%;
            left: 50%;
            width: 90%;
            height: 90%;
            max-width: 500px;
            max-height: 500px;
            transform: translate(-50%, -50%);
        }

        #play_button:hover .icon {
            background-image: url("data:image/svg+xml;base64,CjxzdmcgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB2ZXJzaW9uPSIxLjEiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiBwcmVzZXJ2ZUFzcGVjdFJhdGlvPSJ4TWlkWU1pZCIgeD0iMHB4IiB5PSIwcHgiIHdpZHRoPSIyNTBweCIgaGVpZ2h0PSIyNTBweCIgdmlld0JveD0iMCAwIDI1MCAyNTAiPgo8ZGVmcz4KPGxpbmVhckdyYWRpZW50IGlkPSJHcmFkaWVudF8xIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgeDE9IjEyNSIgeTE9IjAiIHgyPSIxMjUiIHkyPSIyNTAiIHNwcmVhZE1ldGhvZD0icGFkIj4KPHN0b3AgIG9mZnNldD0iMCUiIHN0b3AtY29sb3I9IiNGRkUwNDQiLz4KCjxzdG9wICBvZmZzZXQ9IjEwMCUiIHN0b3AtY29sb3I9IiNGRjQ5NTIiLz4KPC9saW5lYXJHcmFkaWVudD4KCjxnIGlkPSJMYXllcjBfMF9GSUxMIj4KPHBhdGggZmlsbD0idXJsKCNHcmFkaWVudF8xKSIgc3Ryb2tlPSJub25lIiBkPSIKTSAyNTAgMTI1ClEgMjUwIDczLjIgMjEzLjM1IDM2LjYgMTc2LjggMCAxMjUgMCA3My4yIDAgMzYuNjUgMzYuNiAwIDczLjIgMCAxMjUgMCAxNzYuOCAzNi42NSAyMTMuMzUgNzMuMiAyNTAgMTI1IDI1MCAxNzYuOCAyNTAgMjEzLjM1IDIxMy4zNSAyNTAgMTc2LjggMjUwIDEyNQpNIDg3IDE5NQpMIDg3IDU1IDE4NyAxMjUgODcgMTk1IFoiLz4KCjxwYXRoIGZpbGw9IiNGRkZGRkYiIHN0cm9rZT0ibm9uZSIgZD0iCk0gODcgNTUKTCA4NyAxOTUgMTg3IDEyNSA4NyA1NSBaIi8+CjwvZz4KPC9kZWZzPgoKPGcgdHJhbnNmb3JtPSJtYXRyaXgoIDEsIDAsIDAsIDEsIDAsMCkgIj4KPHVzZSB4bGluazpocmVmPSIjTGF5ZXIwXzBfRklMTCIvPgo8L2c+Cjwvc3ZnPgo=");
        }
    </style>
    <style id="dynamic_styles"></style>

    <div id="container">
        <div id="play_button"><div class="icon"></div></div>
        <canvas id="player"></canvas>
    </div>
`;

export default ruffle_shadow_tmpl;