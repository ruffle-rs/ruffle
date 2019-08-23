/** 
 * This IIFE is *not touched by Webpack* and exists primarily to ensure Webpack
 * is loaded without extension privileges.
 * 
 * Inside the IIFE, we do two things:
 * 
 *  1. Use our fancy extension powers to generate an unprivileged script to set
 *     the webpack public path.
 *  2. Generate another unprivileged script with a link to the Ruffle extension
 *     resource.
 * 
 * This gives webpack the environment it expects, at the expense of breaking
 * literally every site that uses it's own webpack.
 */
(function () {
    // Browser extensions are loaded from a dynamically-generated URL, we have to
    // tell webpack about that.

    var webpack_path_script = document.createElement('script');
    webpack_path_script.appendChild(document.createTextNode("__webpack_public_path__ = \"" + browser.runtime.getURL("dist/0.ruffle.js").replace("0.ruffle.js", "") + "\""));
    webpack_path_script.type = "text/javascript";
    document.body.appendChild(webpack_path_script);

    var script = document.createElement('script');
    script.src = browser.runtime.getURL("dist/ruffle.js");
    script.type = "text/javascript";
    document.body.appendChild(script);
}());