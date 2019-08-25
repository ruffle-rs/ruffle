let ruffle_shadow_tmpl = document.createElement("template");
ruffle_shadow_tmpl.innerHTML = `
    <style>
        :host {
            display: block;
        }

        #player {
            width: 100%;
            height: 100%;
        }
    </style>
    <style id="dynamic_styles"></style>
    <canvas id="player"></canvas>
`;

export default ruffle_shadow_tmpl;