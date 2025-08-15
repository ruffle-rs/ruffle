var nextY = 0;
var nextX = 0;

function newStyle() {
    var style = new TextField.StyleSheet();

    var classSmol:Object = new Object();
    classSmol.fontSize = 10;
    style.setStyle(".classSmol", classSmol);

    var classRed:Object = new Object();
    classRed.color = "#FF0000";
    style.setStyle(".classRed", classRed);

    var classGreen:Object = new Object();
    classGreen.color = "#00FF00";
    style.setStyle(".classGreen", classGreen);

    var classRedSmol:Object = new Object();
    classRedSmol.color = "#FF0000";
    classRedSmol.fontSize = 10;
    style.setStyle(".classRedSmol", classRedSmol);

    var classMagenta:Object = new Object();
    classMagenta.color = "#FF00FF";
    style.setStyle(".class-magenta", classMagenta);

    return style;
}

function newTextField() {
    var text = _root.createTextField(
        "text",
        _root.getNextHighestDepth(),
        nextX, nextY, 70, 40);
    nextY += 40;
    if (nextY >= 600) {
        nextY = 0;
        nextX += 80;
    }
    text.embedFonts = true;
    text.border = true;
    text.setNewTextFormat(new TextFormat("TestFont", 20));
    return text;
}

function printText(text) {
    trace("  Text: " + escapeNewlines(text.text));
    trace("  HTML: " + escapeNewlines(text.htmlText));
}

function escapeNewlines(text) {
    return text
        .split("\r").join("\\r")
        .split("\n").join("\\n");
}

function testSetter(value) {
    var text = newTextField();

    text.styleSheet = value;
    trace("  " + text.styleSheet);
}

function runInitialTests() {
    var text = null;

    trace("Getter & setter values:");
    testSetter(4);
    testSetter("4");
    testSetter("");
    testSetter(0.1);
    testSetter(null);
    testSetter(undefined);
    testSetter(new Object());
    testSetter(true);
    testSetter(new TextField.StyleSheet());
    testSetter(_root);

    trace("HTML flag:");
    text = newTextField();
    trace("  Flag before: " + text.html);
    text.styleSheet = null;
    trace("  Flag after setting CSS to null: " + text.html);
    var s = new TextField.StyleSheet();
    text.styleSheet = s;
    trace("  Flag after setting CSS: " + text.html);
    text.styleSheet = null;
    trace("  Flag after resetting CSS: " + text.html);

    trace("Setting style outside:");
    text = newTextField();
    var s = new TextField.StyleSheet();
    text.styleSheet = s;
    trace("  Style before (1): " + s.getStyleNames());
    trace("  Style before (2): " + text.styleSheet.getStyleNames());
    var classRed:Object = new Object();
    classRed.color = "#FF0000";
    s.setStyle(".classRed", classRed);
    trace("  Style after (1):  " + s.getStyleNames());
    trace("  Style after (2):  " + text.styleSheet.getStyleNames());

    trace("Setting style after HTML:");
    text = newTextField();
    text.htmlText = 'ab<font      face="TestFont"><span class="classRed">a</span></font>';
    trace("  Text before:      " + escapeNewlines(text.text));
    trace("  HTML before:      " + escapeNewlines(text.htmlText));
    text.styleSheet = newStyle();
    trace("  Text after style: " + escapeNewlines(text.text));
    trace("  HTML after style: " + escapeNewlines(text.htmlText));
    text.htmlText = 'ab<font      face="TestFont"><span class="classRed">a</span></font>';
    trace("  Text after set:   " + escapeNewlines(text.text));
    trace("  HTML after set:   " + escapeNewlines(text.htmlText));
    text.styleSheet = null;
    trace("  Text no style:    " + escapeNewlines(text.text));
    trace("  HTML no style:    " + escapeNewlines(text.htmlText));
    text.htmlText = 'ab<font      face="TestFont"><span class="classRed">b</span></font>';
    trace("  Text after set:   " + escapeNewlines(text.text));
    trace("  HTML after set:   " + escapeNewlines(text.htmlText));

    trace("Setting text after stylized HTML:");
    text = newTextField();
    text.styleSheet = newStyle();
    text.htmlText = '<font face="TestFont"><span class="classRed">a</span></font>';
    trace("  HTML after style: " + escapeNewlines(text.htmlText));
    trace("  Text after style: " + escapeNewlines(text.text));
    text.text = 'b';
    trace("  HTML after set:   " + escapeNewlines(text.htmlText));
    trace("  Text after set:   " + escapeNewlines(text.text));
    text.styleSheet = null;
    trace("  HTML after reset: " + escapeNewlines(text.htmlText));
    trace("  Text after reset: " + escapeNewlines(text.text));
    text.text = 'c';
    trace("  HTML after set:   " + escapeNewlines(text.htmlText));
    trace("  Text after set:   " + escapeNewlines(text.text));

    trace("Setting HTML in text:");
    text = newTextField();
    text.styleSheet = newStyle();
    text.text = '<font face="TestFont"><span class="classRed">a</span></font>';
    trace("  HTML after style: " + escapeNewlines(text.htmlText));
    trace("  Text after style: " + escapeNewlines(text.text));
    text.styleSheet = null;
    trace("  HTML after reset: " + escapeNewlines(text.htmlText));
    trace("  Text after reset: " + escapeNewlines(text.text));

    trace("Modifying CSS after parsing HTML:");
    text = newTextField();
    var style = new TextField.StyleSheet();
    text.styleSheet = style;
    text.htmlText = '<font face="TestFont"><span class="classRed">a</span></font>';
    trace("  Style (original): " + text.styleSheet.styleNames);
    printText(text);
    var classRed:Object = new Object();
    classRed.color = "#FF0000";
    style.setStyle(".classRed", classRed);
    trace("  Style (after modifying CSS): " + text.styleSheet.styleNames);
    printText(text);
    text.htmlText = '<font face="TestFont"><span class="classRed">a</span></font>';
    trace("  Style (after updating HTML to the same value): " + text.styleSheet.styleNames);
    printText(text);
    text.htmlText = '<font face="TestFont"><span class="classRed">b</span></font>';
    trace("  Style (after updating HTML): " + text.styleSheet.styleNames);
    printText(text);
    text.styleSheet = style;
    trace("  Style (after updating CSS): " + text.styleSheet.styleNames);
    printText(text);
    text.htmlText = '<font face="TestFont"><span class="classRed">c</span></font>';
    trace("  Style (after updating HTML): " + text.styleSheet.styleNames);
    printText(text);
    var classRed:Object = new Object();
    classRed.color = "#00FF00";
    style.setStyle(".classRed", classRed);
    text.styleSheet = style;
    trace("  Style (after updating CSS without HTML): " + text.styleSheet.styleNames);
    printText(text);

    trace("Modifying text after removing CSS:");
    text = newTextField();
    var s = new TextField.StyleSheet();
    text.text = 'ab<font     face="TestFont">a</font>';
    text.styleSheet = s;
    text.styleSheet = null;
    text.text = 'ab<font     face="TestFont">b</font>';
    trace("  Text after: " + text.text);
    trace("  HTML after: " + text.htmlText);

    trace("Modifying text after removing CSS with HTML:");
    text = newTextField();
    var s = new TextField.StyleSheet();
    text.htmlText = 'ab<font     face="TestFont">a</font>';
    text.styleSheet = s;
    text.styleSheet = null;
    text.text = 'ab<font     face="TestFont">b</font>';
    trace("  Text after: " + text.text);
    trace("  HTML after: " + text.htmlText);

    trace("Updating CSS and resetting it:");
    text = newTextField();
    text.styleSheet = new TextField.StyleSheet();
    text.htmlText = '<font face="TestFont"><span class="classred">a</span></font>';
    printText(text);
    text.styleSheet = newStyle();
    text.styleSheet = null;
    printText(text);
}

function runHtmlTest(html, style) {
    html = '<font face="TestFont" size="20">' + html + '</font>';
    var text = newTextField();
    if (style) {
        text.styleSheet = style;
    } else {
        text.styleSheet = newStyle();
    }

    trace("======================================");
    trace("  Setting HTML: " + escapeNewlines(html));
    text.htmlText = html;
    trace("  HTML get: " + escapeNewlines(text.htmlText));
    trace("  Text get: " + escapeNewlines(text.text));
    printText(text);
}

function runHtmlTests() {
    nextY = 0;
    nextX += 100;

    // Unknown class
    runHtmlTest('<span class="unknownClass">a</span>');

    // Basic spans
    runHtmlTest('<span class="classSmol">a</span>');
    runHtmlTest('<span class="classRed">a</span>');
    runHtmlTest('<span class="classGreen">a</span>');
    runHtmlTest('<span class="classRedSmol">a</span>');

    // Nesting & overlapping classes
    runHtmlTest('<span class="classRed">a<span class="classGreen">b</span></span>');
    runHtmlTest('<span class="classRed">a<span class="classRed">b</span></span>');
    runHtmlTest('<span class="classRed">a<span class="classSmol">b</span></span><span class="classRedSmol">c</span>');

    // Multiple classes?
    runHtmlTest('<span class="classRed classSmol">a</span><span class="classRedSmol">b</span>');

    // Spaces in class names
    runHtmlTest('<span class="  classRed">a</span>');
    runHtmlTest('<span class="classRed  ">a</span>');
    runHtmlTest('<span class="   classRed  ">a</span>');

    // Dashes in names
    runHtmlTest('<span class="class-magenta">a</span><span class="classmagenta">b</span><span class="class magenta">c</span>');

    // Case sensitivity
    runHtmlTest('<span class="classred">a</span>');
    runHtmlTest('<span class="CLASSRED">a</span>');
    runHtmlTest('<span class="ClassRed">a</span>');

    // Importance of styles
    runHtmlTest('<font color="#00FF00">a<span class="classRed">b</span></font>');
    runHtmlTest('<span class="classRed">a<font color="#00FF00">b</font></span>');

    // Class on paragraph
    runHtmlTest('<p class="classRed">a</p><p class="classSmol">b</p>');
    runHtmlTest('<p>a</p><p class="classSmol">b</p>');
    runHtmlTest('<p class="classRed">a</p><p>b</p>');
    runHtmlTest('<p>a</p><p>b</p>');

    // Styling elements
    var style = newStyle();
    var classBlue:Object = new Object();
    classBlue.color = "#0000FF";
    style.setStyle("a", classBlue);
    style.setStyle("textformat", classBlue);
    style.setStyle("font", classBlue);
    style.setStyle("p", classBlue);
    style.setStyle("style", classBlue);
    style.setStyle("li", classBlue);
    runHtmlTest('<a>a</a>', style);
    runHtmlTest('<textformat>a</textformat>', style);
    runHtmlTest('<font>a</font>', style);
    runHtmlTest('<p>a</p>', style);
    runHtmlTest('<span>a</span>', style);
    runHtmlTest('<li>a</li>', style);
    runHtmlTest('<a class="classRed">a</a>', style);

    // Style vs attribute preference (font)
    var style = newStyle();
    var classBlue:Object = new Object();
    classBlue.color = "#0000FF";
    style.setStyle("font", classBlue);
    runHtmlTest('<font>a</font>', style);
    runHtmlTest('<font color="#00FF00">a</font>', style);

    // Style vs attribute preference (p)
    var style = newStyle();
    var classPa:Object = new Object();
    classPa.textAlign = "right";
    var classPb:Object = new Object();
    classPb.textAlign = "center";
    style.setStyle("classp", classPa);
    style.setStyle("p", classPb);
    runHtmlTest('<p>a</p>', style);
    runHtmlTest('<p class="classp">a</p>', style);
    runHtmlTest('<p align="justify">a</p>', style);
    runHtmlTest('<p class="classp" align="justify">a</p>', style);
    runHtmlTest('<p align="justify" class="classp">a</p>', style);

    // Style vs attribute preference (span)
    var style = newStyle();
    var classSpan:Object = new Object();
    classSpan.color = "#0000FF";
    style.setStyle("span", classSpan);
    runHtmlTest('<span>a</span>', style);
    runHtmlTest('<span class="colorRed">a</span>', style);

    // Star?
    var style = newStyle();
    var classAll:Object = new Object();
    classAll.color = "#0000FF";
    style.setStyle("*", classAll);
    runHtmlTest('<span>a</span>', style);
    runHtmlTest('a<a>b</a>', style);

    // Specific tags with styles
    var style = newStyle();
    var classAll:Object = new Object();
    classAll.color = "#0000FF";
    style.setStyle("p.classspec", classAll);
    runHtmlTest('<p class="classspec">a</p><span class="classspec">b</span>', style);
}

Stage.scaleMode = "noScale";
runInitialTests();
runHtmlTests();
