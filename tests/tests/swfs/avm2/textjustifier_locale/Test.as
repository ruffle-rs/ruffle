package
{
    import flash.display.MovieClip;
    import flash.text.engine.SpaceJustifier;

    public class Test extends MovieClip
    {
        public function Test()
        {
            // Normal / locale-looking values
            testLocale("en", "en");
            testLocale("ja", "ja");
            testLocale("fr-FR", "fr-FR");
            testLocale("zh-Hant", "zh-Hant");
            testLocale("en-US", "en-US");
            testLocale("EN", "EN");

            // Length boundaries
            testLocale("empty", "");
            testLocale("1 ascii", "a");
            testLocale("2 ascii", "ab");
            testLocale("3 ascii", "abc");

            // Invalid-looking, but length >= 2
            testLocale("2 digits", "12");
            testLocale("2 at signs", "@@");
            testLocale("2 hyphens", "--");
            testLocale("2 underscores", "__");
            testLocale("punctuation", "!?");
            testLocale("slashes", "//");

            // Whitespace
            testLocale("1 space", " ");
            testLocale("2 spaces", "  ");
            testLocale("3 spaces", "   ");
            testLocale("tab", "\u0009");
            testLocale("2 tabs", "\u0009\u0009");
            testLocale("newline", "\n");
            testLocale("2 newlines", "\n\n");
            testLocale("CRLF", "\r\n");

            // Unicode whitespace / invisible chars
            testLocale("NBSP", "\u00A0");
            testLocale("2 NBSP", "\u00A0\u00A0");
            testLocale("ZWSP", "\u200B");
            testLocale("2 ZWSP", "\u200B\u200B");
            testLocale("BOM", "\uFEFF");
            testLocale("2 BOM", "\uFEFF\uFEFF");

            // Non-ASCII, one UTF-16 code unit
            testLocale("e acute", "\u00E9");
            testLocale("Japanese one char", "\u65E5");
            testLocale("Arabic one char", "\u0639");

            // Two non-ASCII chars
            testLocale("Japanese two chars", "\u65E5\u672C");
            testLocale("Arabic two chars", "\u0639\u0631");

            // Combining sequence: 2 UTF-16 code units, visually ~1 character
            testLocale("e + combining acute", "e\u0301");

            // Surrogate pair: one Unicode code point, but String.length == 2 in UTF-16
            testLocale(
                    "surrogate pair",
                    "\uD83D\uDE00"
                );

            // Individual surrogate code units
            testLocale("high surrogate", "\uD83D");
            testLocale("low surrogate", "\uDE00");
            testLocale(
                    "two high surrogates",
                    "\uD83D\uD83D"
                );

            // Embedded NUL/control characters
            testLocale("NUL", "\u0000");
            testLocale("2 NUL", "\u0000\u0000");
            testLocale("a NUL", "a\u0000");
            testLocale("NUL a", "\u0000a");
            testLocale("unit separator", "\u001F");
            testLocale("2 unit separators", "\u001F\u001F");

            // Longer values
            testLocale("length 10", repeatChar("a", 10));
            testLocale("length 15", repeatChar("a", 15));

            // Long locale-like garbage
            testLocale(
                    "long hyphenated",
                    "this-is-a-test-locale"
                );

            // null separately
            testNullLocale();
        }

        private function testLocale(label:String, value:String):void
        {
            try
            {
                var sj:SpaceJustifier = new SpaceJustifier(value);

                trace(
                        label +
                        " | input=" + escapeString(value) +
                        " | length=" + value.length +
                        " | locale=" + escapeString(sj.locale) +
                        " | localeLength=" + sj.locale.length
                    );
            }
            catch (e:*)
            {
                trace(
                        label +
                        " | input=" + escapeString(value) +
                        " | length=" + value.length +
                        " | ERROR=" + e.getStackTrace()
                    );
            }
        }

        private function testNullLocale():void
        {
            try
            {
                var sj:SpaceJustifier = new SpaceJustifier(null);
                trace(
                        "null | locale=" + String(sj.locale)
                    );
            }
            catch (e:*)
            {
                trace(
                        "null | ERROR=" + e.getStackTrace()
                    );
            }
        }

        private function repeatChar(c:String, count:int):String
        {
            var result:String = "";
            for (var i:int = 0; i < count; i++)
            {
                result += c;
            }
            return result;
        }

        private function escapeString(value:String):String
        {
            if (value == null)
            {
                return "null";
            }

            var result:String = "\"";

            for (var i:int = 0; i < value.length; i++)
            {
                var code:int = value.charCodeAt(i);

                if (code >= 0x20 && code <= 0x7E)
                {
                    result += value.charAt(i);
                }
                else
                {
                    var hex:String = code.toString(16).toUpperCase();

                    while (hex.length < 4)
                    {
                        hex = "0" + hex;
                    }

                    result += "\\u" + hex;
                }
            }

            result += "\"";
            return result;
        }
    }
}
