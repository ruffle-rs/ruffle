/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.1.3.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "encodeURI";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    str1 = new String("h");

    array[item++] = Assert.expectEq(   "encodeURI('Empty String')", "",  encodeURI("") );
    array[item++] = Assert.expectEq(   "encodeURI('Single Character')", "h",  encodeURI(str1) );

    str2 = new String("http://www.macromedia.com/flash player");

    array[item++] = Assert.expectEq(   "encodeURI(str2)", "http://www.macromedia.com/flash%20player",  encodeURI(str2) );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com')", "http://www.macromedia.com",  encodeURI("http://www.macromedia.com") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com')", "http://www.macromedia.com/flashA1player",  encodeURI("http://www.macromedia.com/flash\u0041\u0031player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player')", "http://www.macromedia.com/flash%20player",  encodeURI("http://www.macromedia.com/flash player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flasha player')", "http://www.macromedia.com/flasha%20player",  encodeURI("http://www.macromedia.com/flasha player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flashA player')", "http://www.macromedia.com/flashA%20player",  encodeURI("http://www.macromedia.com/flashA player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash_ player')", "http://www.macromedia.com/flash_%20player",  encodeURI("http://www.macromedia.com/flash_ player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash- player')", "http://www.macromedia.com/flash-%20player",  encodeURI("http://www.macromedia.com/flash- player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash. player')", "http://www.macromedia.com/flash.%20player",  encodeURI("http://www.macromedia.com/flash. player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash! player')", "http://www.macromedia.com/flash!%20player",  encodeURI("http://www.macromedia.com/flash! player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash~ player')", "http://www.macromedia.com/flash~%20player",  encodeURI("http://www.macromedia.com/flash~ player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash* player')", "http://www.macromedia.com/flash*%20player",  encodeURI("http://www.macromedia.com/flash* player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/'flash player'')", "http://www.macromedia.com/'flash%20player'",  encodeURI("http://www.macromedia.com/'flash player'") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/(flash player)')", "http://www.macromedia.com/(flash%20player)",  encodeURI("http://www.macromedia.com/(flash player)") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com//tflash player')", "http://www.macromedia.com//tflash%20player",  encodeURI("http://www.macromedia.com//tflash player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/\tflash player')", "http://www.macromedia.com/%09flash%20player",  encodeURI("http://www.macromedia.com/\tflash player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/0987654321flash player')", "http://www.macromedia.com/0987654321flash%20player",  encodeURI("http://www.macromedia.com/0987654321flash player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash; player')", "http://www.macromedia.com/flash;%20player",  encodeURI("http://www.macromedia.com/flash; player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player?')", "http://www.macromedia.com/flash%20player?",  encodeURI("http://www.macromedia.com/flash player?") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player@')", "http://www.macromedia.com/flash%20player@",  encodeURI("http://www.macromedia.com/flash player@") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player&')", "http://www.macromedia.com/flash%20player&",  encodeURI("http://www.macromedia.com/flash player&") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player=')", "http://www.macromedia.com/flash%20player=",  encodeURI("http://www.macromedia.com/flash player=") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player$')", "http://www.macromedia.com/flash%20player$",  encodeURI("http://www.macromedia.com/flash player$") );


    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash player,')", "http://www.macromedia.com/flash%20player,",  encodeURI("http://www.macromedia.com/flash player,") );


    array[item++] = Assert.expectEq(   "encodeURI('aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ')", "aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ",  encodeURI("aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ") );

    array[item++] = Assert.expectEq(   "encodeURI('aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ')", "aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ",  encodeURI("aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\n player,')", "http://www.macromedia.com/flash%0Aplayer",  encodeURI("http://www.macromedia.com/flash\nplayer") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\v player,')", "http://www.macromedia.com/flash%0Bplayer",  encodeURI("http://www.macromedia.com/flash\vplayer") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\f player,')", "http://www.macromedia.com/flash%0Cplayer",  encodeURI("http://www.macromedia.com/flash\fplayer") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\r player,')", "http://www.macromedia.com/flash%0Dplayer",  encodeURI("http://www.macromedia.com/flash\rplayer") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\" player,')", "http://www.macromedia.com/flash%22player",  encodeURI("http://www.macromedia.com/flash\"player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\' player,')", "http://www.macromedia.com/flash'player",  encodeURI("http://www.macromedia.com/flash\'player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\\ player,')", "http://www.macromedia.com/flash%5Cplayer",  encodeURI("http://www.macromedia.com/flash\\player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash# player,')", "http://www.macromedia.com/flash#player",  encodeURI("http://www.macromedia.com/flash#player") );

    array[item++] = Assert.expectEq(   "encodeURI('http://www.macromedia.com/flash\u0000\u0041player,')", "http://www.macromedia.com/flash%00Aplayer",  encodeURI("http://www.macromedia.com/flash\u0000\u0041player") );

    return ( array );
}
