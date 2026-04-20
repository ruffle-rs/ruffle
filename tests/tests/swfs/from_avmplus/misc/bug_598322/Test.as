/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;


//     var SECTION = "String.charAt/codeCodeAt optimizations";

    var testcases = getTestCases();


    function strCharCodeAtFF(s:String, d:Number):Number
    {
        return s.charCodeAt(d);
    }

    function strCharCodeAtFI(s:String, i:int):Number
    {
        return s.charCodeAt(i);
    }

    function strCharCodeAtFU(s:String, u:uint):Number
    {
        return s.charCodeAt(u);
    }

    function strCharCodeAtIF(s:String, d:Number):int
    {
        return s.charCodeAt(d);
    }

    function strCharCodeAtII(s:String, i:int):int
    {
        return s.charCodeAt(i);
    }

    function strCharCodeAtIU(s:String, u:uint):int
    {
        return s.charCodeAt(u);
    }

    function strCharAtF(s:String, d:Number):String
    {
        return s.charAt(d);
    }

    function strCharAtI(s:String, i:int):String
    {
        return s.charAt(i);
    }

    function strCharAtU(s:String, u:uint):String
    {
        return s.charAt(u);
    }

    function getTestCases() {
        var array = new Array();
        var item = 0;

        var a = "abcdefg";

        // test 6 different possible charCodeAt calls
        array[item++] = Assert.expectEq( "charCodeAt(1)", 98, strCharCodeAtFF(a, 1));
        array[item++] = Assert.expectEq( "charCodeAt(1)", 98, strCharCodeAtFI(a, 1));
        array[item++] = Assert.expectEq( "charCodeAt(1)", 98, strCharCodeAtFU(a, 1));
        array[item++] = Assert.expectEq( "charCodeAt(1)", 98, strCharCodeAtIF(a, 1));
        array[item++] = Assert.expectEq( "charCodeAt(1)", 98, strCharCodeAtII(a, 1));
        array[item++] = Assert.expectEq( "charCodeAt(1)", 98, strCharCodeAtIU(a, 1));
        array[item++] = Assert.expectEq( "charCodeAt(-1)", NaN, strCharCodeAtFF(a, -1));
        array[item++] = Assert.expectEq( "charCodeAt(-1)", NaN, strCharCodeAtFI(a, -1));
        array[item++] = Assert.expectEq( "charCodeAt(-1)", NaN, strCharCodeAtFU(a, -1));
        array[item++] = Assert.expectEq( "charCodeAt(-1)", 0, strCharCodeAtIF(a, -1));
        array[item++] = Assert.expectEq( "charCodeAt(-1)", 0, strCharCodeAtII(a, -1));
        array[item++] = Assert.expectEq( "charCodeAt(-1)", 0, strCharCodeAtIU(a, -1));
        array[item++] = Assert.expectEq( "charCodeAt(11)", NaN, strCharCodeAtFF(a, 11));
        array[item++] = Assert.expectEq( "charCodeAt(11)", NaN, strCharCodeAtFI(a, 11));
        array[item++] = Assert.expectEq( "charCodeAt(11)", NaN, strCharCodeAtFU(a, 11));
        array[item++] = Assert.expectEq( "charCodeAt(11)", 0, strCharCodeAtIF(a, 11));
        array[item++] = Assert.expectEq( "charCodeAt(11)", 0, strCharCodeAtII(a, 11));
        array[item++] = Assert.expectEq( "charCodeAt(11)", 0, strCharCodeAtIU(a, 11));

        // test 3 different charAt calls
        array[item++] = Assert.expectEq( "charAt(1)", "b", strCharAtF(a, 1));
        array[item++] = Assert.expectEq( "charAt(1)", "b", strCharAtI(a, 1));
        array[item++] = Assert.expectEq( "charAt(1)", "b", strCharAtU(a, 1));
        array[item++] = Assert.expectEq( "charAt(-1)", "", strCharAtF(a, -1));
        array[item++] = Assert.expectEq( "charAt(-1)", "", strCharAtI(a, -1));
        array[item++] = Assert.expectEq( "charAt(-1)", "", strCharAtU(a, -1));
        array[item++] = Assert.expectEq( "charAt(11)", "", strCharAtF(a, 11));
        array[item++] = Assert.expectEq( "charAt(11)", "", strCharAtI(a, 11));
        array[item++] = Assert.expectEq( "charAt(11)", "", strCharAtU(a, 11));

        var s:String = "foobar";
        var i_index:int;
        var b:Boolean

        // Test the charCodeAt logic in optimizeIntCmpWithNumberCall.
        // String.CharCodeAt == int - any constant integer but zero
        // String.CharCodeAt < int  - zero or any negative integer constant
        // String.CharCodeAt <= int - any negative integer constant
        // int == String.CharCodeAt - any constant integer but zero
        // int < String.CharCodeAt  - zero or any positive integer constant
        // int <= String.CharCodeAt - any positive integer constant

        i_index = 100;
        b = s.charCodeAt(i_index) == 10;
        array[item++] = Assert.expectEq( "charCodeAt(100) == 10", false, b);
        b = 9 == s.charCodeAt(i_index);
        array[item++] = Assert.expectEq( "9 == charCodeAt(100)", false, b);
        b = s.charCodeAt(i_index) < 8;
        array[item++] = Assert.expectEq( "charCodeAt(100 < 8)", false, b);
        b = s.charCodeAt(i_index) <= 7;
        array[item++] = Assert.expectEq( "charCodeAt(100) <= 7", false, b);
        b = s.charCodeAt(i_index) > 6;
        array[item++] = Assert.expectEq( "charCodeAt(100) > 6", false, b);
        b = s.charCodeAt(i_index) >= 5;
        array[item++] = Assert.expectEq( "charCodeAt(100) >= 5", false, b);

        b = (s.charCodeAt(i_index) == -10);
        array[item++] = Assert.expectEq( "charCodeAt(100) == -10", false, b);
        b = (-9 == s.charCodeAt(i_index));
        array[item++] = Assert.expectEq( "-9 = charCodeAt(100)", false, b);
        b = (s.charCodeAt(i_index) < -8);
        array[item++] = Assert.expectEq( "charCodeAt(100) < -8", false, b);
        b = (s.charCodeAt(i_index) <= -7);
        array[item++] = Assert.expectEq( "charCodeAt(100) <= -7", false, b);
        b = (s.charCodeAt(i_index) > -6);
        array[item++] = Assert.expectEq( "charCodeAt(100) > -6", false, b);
        b = (s.charCodeAt(i_index) >= -5);
        array[item++] = Assert.expectEq( "charCodeAt(100) >= -5", false, b);

        b = (s.charCodeAt(i_index) == 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) == 0", false, b);
        b = (0 == s.charCodeAt(i_index));
        array[item++] = Assert.expectEq( "0 == charCodeAt(100)", false, b);
        b = (s.charCodeAt(i_index) < 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) < 0", false, b);
        b = (s.charCodeAt(i_index) <= 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) <= 0", false, b);
        b = (s.charCodeAt(i_index) > 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) > 0", false, b);
        b = (s.charCodeAt(i_index) >= 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) >= 0", false, b);

        i_index = 1;
        b = (s.charCodeAt(i_index) == 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) == 114", false, b);
        b = (114 == s.charCodeAt(i_index));
        array[item++] = Assert.expectEq( "114 = charCodeAt(1)", false, b);
        b = (s.charCodeAt(i_index) < 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) < 114", true, b);
        b = (s.charCodeAt(i_index) <= 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) <= 114", true, b);
        b = (s.charCodeAt(i_index) > 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) > 114", false, b);
        b = (s.charCodeAt(i_index) >= 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) >= 114", false, b);

        b = (s.charCodeAt(i_index) == i_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) == i_index", false, b);
        b = (i_index == s.charCodeAt(i_index));
        array[item++] = Assert.expectEq( "i_index == charCodeAt(1)", false, b);
        b = (s.charCodeAt(i_index) < i_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) < i_index", false, b);
        b = (s.charCodeAt(i_index) <= i_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) <= i_index", false, b);
        b = (s.charCodeAt(i_index) > i_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) > i_index", true, b);
        b = (s.charCodeAt(i_index) >= i_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) >= i_index", true, b);

        var u_index:uint = 100;

        b = s.charCodeAt(u_index) == 10;
        array[item++] = Assert.expectEq( "charCodeAt(100) == 10", false, b);
        b = 9 == s.charCodeAt(u_index);
        array[item++] = Assert.expectEq( "9 == charCodeAt(100)", false, b);
        b = s.charCodeAt(u_index) < 8;
        array[item++] = Assert.expectEq( "charCodeAt(100 < 8)", false, b);
        b = s.charCodeAt(u_index) <= 7;
        array[item++] = Assert.expectEq( "charCodeAt(100) <= 7", false, b);
        b = s.charCodeAt(u_index) > 6;
        array[item++] = Assert.expectEq( "charCodeAt(100) > 6", false, b);
        b = s.charCodeAt(u_index) >= 5;
        array[item++] = Assert.expectEq( "charCodeAt(100) >= 5", false, b);

        b = (s.charCodeAt(u_index) == -10);
        array[item++] = Assert.expectEq( "charCodeAt(100) == -10", false, b);
        b = (-9 == s.charCodeAt(u_index));
        array[item++] = Assert.expectEq( "-9 = charCodeAt(100)", false, b);
        b = (s.charCodeAt(u_index) < -8);
        array[item++] = Assert.expectEq( "charCodeAt(100) < -8", false, b);
        b = (s.charCodeAt(u_index) <= -7);
        array[item++] = Assert.expectEq( "charCodeAt(100) <= -7", false, b);
        b = (s.charCodeAt(u_index) > -6);
        array[item++] = Assert.expectEq( "charCodeAt(100) > -6", false, b);
        b = (s.charCodeAt(u_index) >= -5);
        array[item++] = Assert.expectEq( "charCodeAt(100) >= -5", false, b);

        b = (s.charCodeAt(u_index) == 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) == 0", false, b);
        b = (0 == s.charCodeAt(u_index));
        array[item++] = Assert.expectEq( "0 == charCodeAt(100)", false, b);
        b = (s.charCodeAt(u_index) < 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) < 0", false, b);
        b = (s.charCodeAt(u_index) <= 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) <= 0", false, b);
        b = (s.charCodeAt(u_index) > 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) > 0", false, b);
        b = (s.charCodeAt(u_index) >= 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) >= 0", false, b);

        u_index = 1;
        b = (s.charCodeAt(u_index) == 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) == 114", false, b);
        b = (114 == s.charCodeAt(u_index));
        array[item++] = Assert.expectEq( "114 = charCodeAt(1)", false, b);
        b = (s.charCodeAt(u_index) < 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) < 114", true, b);
        b = (s.charCodeAt(u_index) <= 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) <= 114", true, b);
        b = (s.charCodeAt(u_index) > 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) > 114", false, b);
        b = (s.charCodeAt(u_index) >= 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) >= 114", false, b);

        b = (s.charCodeAt(u_index) == u_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) == u_index", false, b);
        b = (u_index == s.charCodeAt(u_index));
        array[item++] = Assert.expectEq( "u_index == charCodeAt(1)", false, b);
        b = (s.charCodeAt(u_index) < u_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) < u_index", false, b);
        b = (s.charCodeAt(u_index) <= u_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) <= u_index", false, b);
        b = (s.charCodeAt(u_index) > u_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) > u_index", true, b);
        b = (s.charCodeAt(u_index) >= u_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) >= u_index", true, b);

        var f_index:Number = 100;
        b = s.charCodeAt(f_index) == 10;
        array[item++] = Assert.expectEq( "charCodeAt(100) == 10", false, b);
        b = 9 == s.charCodeAt(f_index);
        array[item++] = Assert.expectEq( "9 == charCodeAt(100)", false, b);
        b = s.charCodeAt(f_index) < 8;
        array[item++] = Assert.expectEq( "charCodeAt(100 < 8)", false, b);
        b = s.charCodeAt(f_index) <= 7;
        array[item++] = Assert.expectEq( "charCodeAt(100) <= 7", false, b);
        b = s.charCodeAt(f_index) > 6;
        array[item++] = Assert.expectEq( "charCodeAt(100) > 6", false, b);
        b = s.charCodeAt(f_index) >= 5;
        array[item++] = Assert.expectEq( "charCodeAt(100) >= 5", false, b);

        b = (s.charCodeAt(f_index) == -10);
        array[item++] = Assert.expectEq( "charCodeAt(100) == -10", false, b);
        b = (-9 == s.charCodeAt(f_index));
        array[item++] = Assert.expectEq( "-9 = charCodeAt(100)", false, b);
        b = (s.charCodeAt(f_index) < -8);
        array[item++] = Assert.expectEq( "charCodeAt(100) < -8", false, b);
        b = (s.charCodeAt(f_index) <= -7);
        array[item++] = Assert.expectEq( "charCodeAt(100) <= -7", false, b);
        b = (s.charCodeAt(f_index) > -6);
        array[item++] = Assert.expectEq( "charCodeAt(100) > -6", false, b);
        b = (s.charCodeAt(f_index) >= -5);
        array[item++] = Assert.expectEq( "charCodeAt(100) >= -5", false, b);

        b = (s.charCodeAt(f_index) == 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) == 0", false, b);
        b = (0 == s.charCodeAt(f_index));
        array[item++] = Assert.expectEq( "0 == charCodeAt(100)", false, b);
        b = (s.charCodeAt(f_index) < 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) < 0", false, b);
        b = (s.charCodeAt(f_index) <= 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) <= 0", false, b);
        b = (s.charCodeAt(f_index) > 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) > 0", false, b);
        b = (s.charCodeAt(f_index) >= 0);
        array[item++] = Assert.expectEq( "charCodeAt(100) >= 0", false, b);

        f_index = 1;
        b = (s.charCodeAt(f_index) == 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) == 114", false, b);
        b = (114 == s.charCodeAt(f_index));
        array[item++] = Assert.expectEq( "114 = charCodeAt(1)", false, b);
        b = (s.charCodeAt(f_index) < 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) < 114", true, b);
        b = (s.charCodeAt(f_index) <= 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) <= 114", true, b);
        b = (s.charCodeAt(f_index) > 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) > 114", false, b);
        b = (s.charCodeAt(f_index) >= 114);
        array[item++] = Assert.expectEq( "charCodeAt(1) >= 114", false, b);

        b = (s.charCodeAt(f_index) == f_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) == f_index", false, b);
        b = (f_index == s.charCodeAt(f_index));
        array[item++] = Assert.expectEq( "f_index == charCodeAt(1)", false, b);
        b = (s.charCodeAt(f_index) < f_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) < f_index", false, b);
        b = (s.charCodeAt(f_index) <= f_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) <= f_index", false, b);
        b = (s.charCodeAt(f_index) > f_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) > f_index", true, b);
        b = (s.charCodeAt(f_index) >= f_index);
        array[item++] = Assert.expectEq( "charCodeAt(1) >= f_index", true, b);

        var val:int = 0;
        i_index = 0;
        switch (s.charAt(i_index))
        {
            case "a":
                val = 1;
                break;
            case "f":
                val = 6;
                break;
            default:
        }

        array[item++] = Assert.expectEq( "s.charAt(int) in switch", 6, val);

        u_index = 0;
        switch (s.charAt(u_index))
        {
            case "a":
                val = 1;
                break;
            case "f":
                val = 6;
                break;
            default:
        }

        array[item++] = Assert.expectEq( "s.charAt(uint) in switch", 6, val);

        f_index = 0;
        switch (s.charAt(f_index))
        {
            case "a":
                val = 1;
                break;
            case "f":
                val = 6;
                break;
            default:
        }

        array[item++] = Assert.expectEq( "s.charAt(Number) in switch", 6, val);

        return array;
    }


