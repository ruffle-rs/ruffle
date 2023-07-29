/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.1.3.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "encodeURIComponent";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    str1 = new String("h");

    array[item++] = Assert.expectEq(   "encodeURIComponent('')", "",  encodeURIComponent("") );

    array[item++] = Assert.expectEq(   "encodeURIComponent(str1)", "h",  encodeURIComponent(str1) );

    array[item++] = Assert.expectEq(   "encodeURIComponent('Hello{[World]}')", "Hello%7B%5BWorld%5D%7D",  encodeURIComponent("Hello{[World]}") );
    array[item++] = Assert.expectEq(   "encodeURIComponent('Macromedia - Flash')", "Macromedia%20-%20Flash",  encodeURIComponent("Macromedia - Flash") );
    array[item++] = Assert.expectEq(   "encodeURIComponent('2 * 4 + [8 + 5 ] - 3')", "2%20*%204%20%2B%20%5B8%20%2B%205%20%5D%20-%203",  encodeURIComponent("2 * 4 + [8 + 5 ] - 3") );
    array[item++] = Assert.expectEq(   "encodeURIComponent('Flash(Macromedia)')", "Flash(Macromedia)",  encodeURIComponent("Flash(Macromedia)") );
    array[item++] = Assert.expectEq(   "encodeURIComponent('BugID #17485')", "BugID%20%2317485",  encodeURIComponent("BugID #17485") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flasha player')", "http%3A%2F%2Fwww.macromedia.com%2Fflasha%20player",  encodeURIComponent("http://www.macromedia.com/flasha player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flashA player')", "http%3A%2F%2Fwww.macromedia.com%2FflashA%20player",  encodeURIComponent("http://www.macromedia.com/flashA player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash_ player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash_%20player",  encodeURIComponent("http://www.macromedia.com/flash_ player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash- player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash-%20player",  encodeURIComponent("http://www.macromedia.com/flash- player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash. player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash.%20player",  encodeURIComponent("http://www.macromedia.com/flash. player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash! player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash!%20player",  encodeURIComponent("http://www.macromedia.com/flash! player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash~ player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash~%20player",  encodeURIComponent("http://www.macromedia.com/flash~ player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash* player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash*%20player",  encodeURIComponent("http://www.macromedia.com/flash* player") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/'flash player'')", "http%3A%2F%2Fwww.macromedia.com%2F'flash%20player'",  encodeURIComponent("http://www.macromedia.com/'flash player'") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/(flash player)')", "http%3A%2F%2Fwww.macromedia.com%2F(flash%20player)",  encodeURIComponent("http://www.macromedia.com/(flash player)") );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http%3A%2F%2Fwww.macromedia.com/flash; player')", "http%3A%2F%2Fwww.macromedia.com%2Fflash%3B%20player",  encodeURIComponent("http://www.macromedia.com/flash; player")+"" );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash player?')", "http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%3F",  encodeURIComponent("http://www.macromedia.com/flash player?")+"" );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash player@')", "http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%40",  encodeURIComponent("http://www.macromedia.com/flash player@")+"" );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash player&')", "http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%26",  encodeURIComponent("http://www.macromedia.com/flash player&")+"" );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash player=')", "http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%3D",  encodeURIComponent("http://www.macromedia.com/flash player=")+"" );

    array[item++] = Assert.expectEq(   "encodeURIComponent('http://www.macromedia.com/flash player$')", "http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%24",  encodeURIComponent("http://www.macromedia.com/flash player$")+"" );

    return ( array );
}
