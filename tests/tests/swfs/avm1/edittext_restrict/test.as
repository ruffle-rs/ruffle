// https://open-flash.github.io/mirrors/as2-language-reference/TextField.html#restrict
// There is one TextField named "text", which receives the text input.
// We are changing its "restrict" property with various values (variable "restricts").
// When we receive "KeyDown" with the given code, we print
// the actual text and switch the restrict to the next one.
// For each restrict we are testing the following values:
//   a,b,c — lowercase ASCII letters
//   A,B,C — uppercase ASCII letters
//   0,1,2 — digits
//   ^,\,- — special characters with a meaning
//   *, ,& — other special characters
//   ą,δ,ł — lowercase non-ASCII letters
//   Ą,Δ,Ł — uppercase non-ASCII letters
//   ß     — a German letter with a controversial uppercase form
// When checking the test manually, one can copy the text, and keep pressing
// Ctrl-V, ESC, Ctrl-V, ESC, ... until all restricts are tested.
// Copy-pastable text: "abcABC012^\-* &ąδłĄΔŁß"

var restricts = [
    // different empty values
    undefined,
    null,
    "",
    false,
    // non-empty non-string values
    1,
    true,
    0.1,
    NaN,
    new Object(),
    // only selected chars
    "aB*Δ",
    "aa",
    // ASCII ranges
    "a-z",
    "A-Z",
    "a-bA",
    // non-standard ranges
    "a-",
    "-b",
    "b-a",
    "A-z",
    "-",
    "--",
    "---",
    "----",
    "-----",
    "-----b",
    "b-----",
    "a-b-c",
    "a-b-A",
    "a-a-b",
    "\\\\-\\^",
    "\\^-\\\\",
    // various behaviors with caret ^
    "^",
    "^^",
    "\\^a",
    "^\\^",
    "^aą",
    "a^b^c",
    "a^b^c^A^B",
    "a-zA-Z^bC",
    "a-zA-Z^",
    // escapes
    "\\-",
    "a\\-z",
    "\\\\",
    "\\^",
    "\\ab",
    "a\\",
    "\u0020-\u007E",
    // unicode range
    "α-ω"
];

var currentRestrict = -1;

function nextRestrict() {
    trace("Text: '" + text.text + "'");
    trace("====================");
    text.text = "";
    currentRestrict += 1;
    if (restricts.length <= currentRestrict) {
        trace("No more restricts");
        return;
    }
    text.restrict = restricts[currentRestrict];
    trace("Restrict set: '" + restricts[currentRestrict] + "'");
    trace("Restrict get: '" + text.restrict + "'");
}

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        nextRestrict();
    }
};
Key.addListener(listener);

Selection.setFocus(text);
