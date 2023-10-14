/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

var str_utf16_codes = [0x31,
                0x32,
                0x33,
                0xd842,
                0xdf9f,
                0x54a4,
                0x41,
                0x42,
                0x43,
                0xd842,
                0xdfb7,
                0x91ce,
                0x5c4b,
                0x61,
                0x62,
                0x63,
                0x5357,
                0xd87e,
                0xdc84,
                0x99c5];

var str_utf16:String = "";
for each (var c in str_utf16_codes)
    str_utf16 += String.fromCharCode(c);

// note, it's critical that these be embedded as literal utf8 strings to trigger the proper code path
// (constructing via String.fromCharCode won't do the trick)
var str_utf8:String = "123𠮟咤ABC𠮷野屋abc南巽駅";
var str_utf8_a:String = "123𠮟咤ABC";
var str_utf8_b:String = "𠮷野屋abc南巽駅";

var str_utf8_ab:String = str_utf8_a + str_utf8_b;


Assert.expectEq("str_utf8 == str_utf16", true, str_utf8 == str_utf16);
Assert.expectEq("str_utf8.length == str_utf16.length", true, str_utf8.length == str_utf16.length);
Assert.expectEq("str_utf8_ab == str_utf8", true, str_utf8_ab == str_utf8);


