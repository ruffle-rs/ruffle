/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.1.3.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "decodeURIComponent";


    var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:Number= 0;

    var str1:String = new String("h");

    array[item++] = Assert.expectEq(   "decodeURIComponent('Hello%7B%5BWorld%5D%7D')", "Hello{[World]}",  decodeURIComponent("Hello%7B%5BWorld%5D%7D") );
    array[item++] = Assert.expectEq(   "decodeURIComponent('Macromedia%20-%20Flash')", "Macromedia - Flash",  decodeURIComponent("Macromedia%20-%20Flash") );
    array[item++] = Assert.expectEq(   "decodeURIComponent('2%20*%204%20%2B%20%5B8%20%2B%205%20%5D%20-%203')", "2 * 4 + [8 + 5 ] - 3",  decodeURIComponent("2%20*%204%20%2B%20%5B8%20%2B%205%20%5D%20-%203") );
    array[item++] = Assert.expectEq(   "decodeURIComponent('Flash(Macromedia)')", "Flash(Macromedia)",  decodeURIComponent("Flash(Macromedia)") );
    array[item++] = Assert.expectEq(   "decodeURIComponent('BugID%20%2317485')", "BugID #17485",  decodeURIComponent("BugID%20%2317485") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflasha%20player')", "http://www.macromedia.com/flasha player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflasha%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2FflashA%20player')", "http://www.macromedia.com/flashA player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2FflashA%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash_%20player')", "http://www.macromedia.com/flash_ player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash_%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash-%20player')", "http://www.macromedia.com/flash- player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash-%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash.%20player')", "http://www.macromedia.com/flash. player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash.%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash!%20player')", "http://www.macromedia.com/flash! player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash!%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash~%20player')", "http://www.macromedia.com/flash~ player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash~%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash*%20player')", "http://www.macromedia.com/flash* player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash*%20player") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2F'flash%20player'')", "http://www.macromedia.com/'flash player'",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2F'flash%20player'") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2F(flash%20player)')", "http://www.macromedia.com/(flash player)",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2F(flash%20player)") );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash%3B%20player')", "http://www.macromedia.com/flash; player",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash%3B%20player")+"" );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%3F')", "http://www.macromedia.com/flash player?",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%3F")+"" );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%40')", "http://www.macromedia.com/flash player@",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%40")+"" );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%26')", "http://www.macromedia.com/flash player&",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%26")+"" );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%3D')", "http://www.macromedia.com/flash player=",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%3D")+"" );

    array[item++] = Assert.expectEq(   "decodeURIComponent('http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%24')", "http://www.macromedia.com/flash player$",  decodeURIComponent("http%3A%2F%2Fwww.macromedia.com%2Fflash%20player%24")+"" );

    var thisError:String = 'no exception';
    try{
        decodeURIComponent('http://www.macromedia.com/flash%GKplayer')
    }catch(e:Error){
        thisError=(e.toString()).substring(0,8);
    }finally{
        array[item++] = Assert.expectEq(   "Characters following % should be hexa decimal digits", "URIError",  thisError);
    }


   thisError = 'no exception';
   try{
       decodeURIComponent('http://www.macromedia.com/flash%20player%')
   }catch(e1:Error){
       thisError=(e1.toString()).substring(0,8);
   }finally{
       array[item++] = Assert.expectEq(   "If the last character of string is % throw URIError", "URIError",  thisError);
   }

   thisError = 'no exception';
   try{
       decodeURIComponent('http://www.macromedia.com/flash5%2player')
   }catch(e2:Error){
       thisError=(e2.toString()).substring(0,8);
   }finally{
       array[item++] = Assert.expectEq(   "If the character at position k  of string before hexadecimal digits is not % throw URIError", "URIError",  thisError);
   }


    return ( array );
}
