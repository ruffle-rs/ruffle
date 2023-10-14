/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.1.3.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "decodeURI";

    var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;

    var str1 = new String("h");

    array[item++] = Assert.expectEq(   "decodeURI('')", "",  decodeURI("") );

    array[item++] = Assert.expectEq(   "decodeURI(str1)", "h",  decodeURI(str1) );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flashA1player')", "http://www.macromedia.com/flash\u0041\u0031player",  decodeURI("http://www.macromedia.com/flashA1player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flasha%20player')", "http://www.macromedia.com/flasha player",  decodeURI("http://www.macromedia.com/flasha%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flashA%20player')", "http://www.macromedia.com/flashA player",  decodeURI("http://www.macromedia.com/flashA%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash_%20player')", "http://www.macromedia.com/flash_ player",  decodeURI("http://www.macromedia.com/flash_%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash-%20player')", "http://www.macromedia.com/flash- player",  decodeURI("http://www.macromedia.com/flash-%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash.%20player')", "http://www.macromedia.com/flash. player",  decodeURI("http://www.macromedia.com/flash.%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash!%20player')", "http://www.macromedia.com/flash!  player",  decodeURI("http://www.macromedia.com/flash!%20 player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash~%20player')", "http://www.macromedia.com/flash~ player",  decodeURI("http://www.macromedia.com/flash~%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash*%20player')", "http://www.macromedia.com/flash* player",  decodeURI("http://www.macromedia.com/flash*%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/'flash%20player'')", "http://www.macromedia.com/'flash player'",  decodeURI("http://www.macromedia.com/'flash%20player'") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/(flash%20player)')", "http://www.macromedia.com/(flash player)",  decodeURI("http://www.macromedia.com/(flash%20player)") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/%09flash%20player", "http://www.macromedia.com/\tflash player",  decodeURI("http://www.macromedia.com/%09flash%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/0987654321flash%20player')", "http://www.macromedia.com/0987654321flash player",  decodeURI("http://www.macromedia.com/0987654321flash%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash;%20player')", "http://www.macromedia.com/flash; player",  decodeURI("http://www.macromedia.com/flash;%20player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player?')", "http://www.macromedia.com/flash player?",  decodeURI("http://www.macromedia.com/flash%20player?") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player@')", "http://www.macromedia.com/flash player@",  decodeURI("http://www.macromedia.com/flash%20player@") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player&')", "http://www.macromedia.com/flash player&",  decodeURI("http://www.macromedia.com/flash%20player&") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player=')", "http://www.macromedia.com/flash player=",  decodeURI("http://www.macromedia.com/flash%20player=") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player$')", "http://www.macromedia.com/flash player$",  decodeURI("http://www.macromedia.com/flash%20player$") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player,')", "http://www.macromedia.com/flash player,",  decodeURI("http://www.macromedia.com/flash%20player,") );

    array[item++] = Assert.expectEq(   "decodeURI('aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ')", "aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ",  decodeURI("aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ") );

    array[item++] = Assert.expectEq(   "decodeURI('aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ')", "aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ",  decodeURI("aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%0Aplayer')", "http://www.macromedia.com/flash\nplayer",  decodeURI("http://www.macromedia.com/flash%0Aplayer") );

   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%0Bplayer')", "http://www.macromedia.com/flash\vplayer",  decodeURI("http://www.macromedia.com/flash%0Bplayer") );


   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%0Cplayer')", "http://www.macromedia.com/flash\fplayer",  decodeURI("http://www.macromedia.com/flash%0Cplayer") );

   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%0Dplayer')", "http://www.macromedia.com/flash\rplayer",  decodeURI("http://www.macromedia.com/flash%0Dplayer") );

   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%22player')", "http://www.macromedia.com/flash\"player",  decodeURI("http://www.macromedia.com/flash%22player") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%27player')", "http://www.macromedia.com/flash\'player",  decodeURI("http://www.macromedia.com/flash%27player") );

   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%5Cplayer')", "http://www.macromedia.com/flash\\player",  decodeURI("http://www.macromedia.com/flash%5Cplayer") );

   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash#player')", "http://www.macromedia.com/flash#player",  decodeURI("http://www.macromedia.com/flash#player") );

   array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%00Aplayer')", "http://www.macromedia.com/flash\u0000\u0041player",  decodeURI("http://www.macromedia.com/flash%00Aplayer") );


    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com')", "http://www.macromedia.com",  decodeURI("http://www.macromedia.com") );

    array[item++] = Assert.expectEq(   "decodeURI('http://www.macromedia.com/flash%20player')", "http://www.macromedia.com/flash player",  decodeURI("http://www.macromedia.com/flash%20player") );

    var thisError:String = 'no exception';
   try{
       decodeURI('http://www.macromedia.com/flash%GKplayer')
   }catch(e:Error){
       thisError=(e.toString()).substring(0,8);
   }finally{
       array[item++] = Assert.expectEq(   "Characters following % should be hexa decimal digits", "URIError",  thisError);
   }


   thisError = 'no exception';
   try{
       decodeURI('http://www.macromedia.com/flash%20player%')
   }catch(e1:Error){
       thisError=(e1.toString()).substring(0,8);
   }finally{
       array[item++] = Assert.expectEq(   "If the last character of string is % throw URIError", "URIError",  thisError);
   }

   thisError = 'no exception';
   try{
       decodeURI('http://www.macromedia.com/flash5%2player')
   }catch(e2:Error){
       thisError=(e2.toString()).substring(0,8);
   }finally{
       array[item++] = Assert.expectEq(   "If the character at position k  of string is not % throw URIError", "URIError",  thisError);
   }

    return ( array );
}
