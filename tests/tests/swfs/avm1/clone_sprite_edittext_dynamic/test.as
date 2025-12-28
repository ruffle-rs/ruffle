function newStyle() {
    var style = new TextField.StyleSheet();

    var classRed:Object = new Object();
    classRed.color = "#FF0000";
    style.setStyle(".classRed", classRed);

    return style;
}

function traceProp(variableName, propName) {
    var text = eval(variableName);
    var text_clone = eval(variableName + "_clone");

    var before = text[propName];
    var after = text_clone[propName];

    if (propName == "_xscale" || propName == "_yscale" || propName == "_rotation") {
        after = Math.round(after);
    }

    trace(variableName + "." + propName + ": " + before + " -> " + after + ";");
}

function traceFormat(variableName) {
    var text = eval(variableName);
    var text_clone = eval(variableName + "_clone");

    var before = text.getNewTextFormat();
    var after = text_clone.getNewTextFormat();

    trace(variableName + ".getNewTextFormat().align: " + before.align + " -> " + after.align + ";");
    trace(variableName + ".getNewTextFormat().bold: " + before.bold + " -> " + after.bold + ";");
    trace(variableName + ".getNewTextFormat().color: " + before.color + " -> " + after.color + ";");
    trace(variableName + ".getNewTextFormat().indent: " + before.indent + " -> " + after.indent + ";");
    trace(variableName + ".getNewTextFormat().leading: " + before.leading + " -> " + after.leading + ";");
    trace(variableName + ".getNewTextFormat().leftMargin: " + before.leftMargin + " -> " + after.leftMargin + ";");
    trace(variableName + ".getNewTextFormat().rightMargin: " + before.rightMargin + " -> " + after.rightMargin + ";");
    trace(variableName + ".getNewTextFormat().size: " + before.size + " -> " + after.size + ";");
}

createTextField("text1", 101, -5, 17, 31, 24);

createTextField("text2", 102, -4, 16, 12, 13);
text2._alpha = 0.5;
text2.antiAliasType = "advanced";
text2.autoSize = "right";
text2.background = true;
text2.backgroundColor = 0x5;
text2.border = true;
text2.borderColor = 0x6;
text2.condenseWhite = true;
text2.embedFonts = true;
text2.filters = [
    new flash.filters.DropShadowFilter()
];
text2.gridFitType = "subpixel";
text2._height = 18;
text2.hscroll = 10;
text2.html = true;
text2.html = true;
text2.htmlText = "<font face=\"X\">asdfghjkl<br>jasdhhjasd<br>ajshdjhasd<br>jashdasjdh</font>";
text2.maxChars = 24;
text2.multiline = true;
text2.scroll = 2;
text2.styleSheet = newStyle();
text2._width = 19;

createTextField("text3", 103, 1, 2, 12, 13);
text3.password = true;
text3.restrict = "asd";
text3.selectable = false;
text3.sharpness = 500;
text3.tabEnabled = true;
text3.tabIndex = 5;
text3.text = "x";
text3.textColor = 7;
text3.thickness = 7;
text3.type = "input";
text3.wordWrap = true;

createTextField("text4", 104, 0, 0, 12, 13);
text4._rotation = 5;
text4.variable = "test";
text4._visible = false;
text4._x = 5;
text4._y = 6;
text4._xscale = 200;
text4._yscale = 150;

duplicateMovieClip(text1, "text1_clone", 201);
duplicateMovieClip(text2, "text2_clone", 202);
duplicateMovieClip(text3, "text3_clone", 203);
duplicateMovieClip(text4, "text4_clone", 204);

traceProp("text1", "_x");
traceProp("text1", "_y");
traceProp("text1", "_width");
traceProp("text1", "_height");
traceFormat("text1");

traceProp("text2", "_x");
traceProp("text2", "_y");
traceProp("text2", "_width");
traceProp("text2", "_height");
traceProp("text2", "_alpha");
traceProp("text2", "antiAliasType");
traceProp("text2", "autoSize");
traceProp("text2", "background");
traceProp("text2", "backgroundColor");
traceProp("text2", "border");
traceProp("text2", "borderColor");
traceProp("text2", "condenseWhite");
traceProp("text2", "embedFonts");
traceProp("text2", "filters");
traceProp("text2", "gridFitType");
traceProp("text2", "_height");
traceProp("text2", "hscroll");
traceProp("text2", "html");
traceProp("text2", "html");
traceProp("text2", "htmlText");
traceProp("text2", "maxChars");
traceProp("text2", "multiline");
traceProp("text2", "_name");
traceProp("text2", "scroll");
traceProp("text2", "styleSheet");
traceProp("text2", "_width");
traceFormat("text2");

traceProp("text3", "_x");
traceProp("text3", "_y");
traceProp("text3", "_width");
traceProp("text3", "_height");
traceProp("text3", "password");
traceProp("text3", "restrict");
traceProp("text3", "selectable");
traceProp("text3", "sharpness");
traceProp("text3", "tabEnabled");
traceProp("text3", "tabIndex");
traceProp("text3", "text");
traceProp("text3", "textColor");
traceProp("text3", "thickness");
traceProp("text3", "type");
traceProp("text3", "wordWrap");
traceFormat("text3");

traceProp("text4", "_x");
traceProp("text4", "_y");
traceProp("text4", "_width");
traceProp("text4", "_height");
traceProp("text4", "_rotation");
traceProp("text4", "variable");
traceProp("text4", "_visible");
traceProp("text4", "_xscale");
traceProp("text4", "_yscale");
traceFormat("text4");
