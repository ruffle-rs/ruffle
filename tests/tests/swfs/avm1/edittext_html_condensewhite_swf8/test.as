function escapeNewlines(text) {
    return text
        .split("\r").join("\\r")
        .split("\n").join("\\n");
}

function runTest(text, html) {
    trace("  HTML set:    " + escapeNewlines(html));

    text.multiline = false;
    text.htmlText = html;
    var lastHtml = text.htmlText;
    trace("  HTML get:    " + escapeNewlines(lastHtml));
    trace("  Text get:    " + escapeNewlines(text.text));

    text.multiline = true;
    text.htmlText = html;
    if (lastHtml === text.htmlText) {
        trace("  HTML get ml: <!-- the same -->");
    } else {
        trace("  HTML get ml: " + escapeNewlines(text.htmlText));
    }
    trace("  Text get:    " + escapeNewlines(text.text));
    trace("===============");
}

function runTests(text) {
    trace("condenseWhite: " + text.condenseWhite);
    text.condenseWhite = 'test';
    trace("condenseWhite = 'test': " + text.condenseWhite);
    text.condenseWhite = 5;
    trace("condenseWhite = 5: " + text.condenseWhite);
    text.condenseWhite = 0;
    trace("condenseWhite = 0: " + text.condenseWhite);
    text.condenseWhite = true;
    trace("condenseWhite = true: " + text.condenseWhite);

    runTest(text, '\n');
    runTest(text, '\n\n');
    runTest(text, ' ');
    runTest(text, '  ');
    runTest(text, ' \n');
    runTest(text, '\n ');
    runTest(text, '\n \tasdf \t  \n');
    runTest(text, ' test ');
    runTest(text, ' test test ');
    runTest(text, '\ntest  \n');
    runTest(text, 'test \n');
    runTest(text, 'test \ntest\n\ntest\n\n\ntest');
    runTest(text, 'test\n \ntest \n');
    runTest(text, '<b>test</b> \n');
    runTest(text, '\n <p>t\ne s\tt </p> \n');
    runTest(text, '<li>test</li>\n');
    runTest(text, '<b> \n </b>');
    runTest(text, '<b></b>\n');
    runTest(text, '<b> </b>');
    runTest(text, ' <b> </b> ');
    runTest(text, ' <p> </p> ');
    runTest(text, '<b> test </b>');
    runTest(text, '<b>\ntest\n</b>');
    runTest(text, '\n<p>test</p>\n');
    runTest(text, ' <p>test</p>  <p>test</p> ');
    runTest(text, '<p></p>\n');
    runTest(text, '<p>\n</p>');
    runTest(text, '<p>  \n  </p>\n  ');
    runTest(text, '<p> </p> ');
    runTest(text, '<p> test </p>');
    runTest(text, '<p>\ntest\n</p>');
    runTest(text, '<li></li>\n');
    runTest(text, '<li>\n</li>');
    runTest(text, '<li>test\n</li>');
    runTest(text, '<li> </li>');
    runTest(text, '<li> </li> ');
    runTest(text, '<li> test </li>');
    runTest(text, '<li>\ntest\n</li>');
    runTest(text, 'a b \xa0c \x01 \x02 \x03 d');
    runTest(text, 'a \x04 \x05 \x06 \x07 b');
    runTest(text, 'a \x08 \x09 \x0a \x0b b');
    runTest(text, 'a \x0c \x0d \x0e \x0f b');
    runTest(text, 'a \x10 \x11 \x12 \x13 b');
    runTest(text, 'a \x14 \x15 \x16 \x17 b');
    runTest(text, 'a \x18 \x19 b');
    runTest(text, '  <p>  test  </p>  ');
    runTest(text, '  <p>\n test  </p>\n ');
    runTest(text, '  <p>test</p>  ');
    runTest(text, '<p>  test  </p>');
    runTest(text, ' <b> test1 </b> <b> test2 </b> ');
    runTest(text, ' <b> test1 </b> <i> test2 </i> ');
}

runTests(text);
