/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
    
    
//     var SECTION = 'As described in Netscape doc "Whats new in JavaScript 1.2"';
//     var VERSION = 'no version';
//     var TITLE = 'String:concat';


    var testcases = new getTestCases();
    
function getTestCases() {

    var array = new Array();
    var item = 0;
    
    var aString = new String("test string");
    var bString = new String(" another ");

    array[item++] = Assert.expectEq(  "String.prototype.concat.length", 0,     String.prototype.concat.length);

    array[item++] = Assert.expectEq(  "aString.concat(' more')", "test string more",     aString.concat(' more').toString());
    array[item++] = Assert.expectEq(  "aString.concat(bString)", "test string another ", aString.concat(bString).toString());
    array[item++] = Assert.expectEq(  "aString                ", "test string",          aString.toString());
    array[item++] = Assert.expectEq(  "bString                ", " another ",            bString.toString());
    array[item++] = Assert.expectEq(  "aString.concat(345)    ", "test string345",       aString.concat(345).toString());
    array[item++] = Assert.expectEq(  "aString.concat(true)   ", "test stringtrue",      aString.concat(true).toString());
    array[item++] = Assert.expectEq(  "aString.concat(null)   ", "test stringnull",      aString.concat(null).toString());
    array[item++] = Assert.expectEq(  "aString.concat([])     ", "test string",          aString.concat([]).toString());
    array[item++] = Assert.expectEq(  "aString.concat([1,2,3])", "test string1,2,3",     aString.concat([1,2,3]).toString());

    array[item++] = Assert.expectEq(  "'abcde'.concat(' more')", "abcde more",     'abcde'.concat(' more').toString());
    array[item++] = Assert.expectEq(  "'abcde'.concat(bString)", "abcde another ", 'abcde'.concat(bString).toString());
    array[item++] = Assert.expectEq(  "'abcde'                ", "abcde",          'abcde');
    array[item++] = Assert.expectEq(  "'abcde'.concat(345)    ", "abcde345",       'abcde'.concat(345).toString());
    array[item++] = Assert.expectEq(  "'abcde'.concat(true)   ", "abcdetrue",      'abcde'.concat(true).toString());
    array[item++] = Assert.expectEq(  "'abcde'.concat(null)   ", "abcdenull",      'abcde'.concat(null).toString());
    array[item++] = Assert.expectEq(  "'abcde'.concat([])     ", "abcde",          'abcde'.concat([]).toString());
    array[item++] = Assert.expectEq(  "'abcde'.concat([1,2,3])", "abcde1,2,3",     'abcde'.concat([1,2,3]).toString());
    array[item++] = Assert.expectEq(  "'abcde'.concat([1,2,3])", "abcde1,2,33,4,5string12345nulltrueundefined",     'abcde'.concat([1,2,3],[3,4,5],'string',12345,null,true,undefined).toString());

    //what should this do:
    array[item++] = Assert.expectEq(  "'abcde'.concat()       ", "abcde",          'abcde'.concat().toString());

    //concat method transferred to other objects for use as method
   
    var myobj = new Object();
       
    myobj.concat = String.prototype.concat;
       
       
    array[item++] = Assert.expectEq(  "myobj.concat([1,2,3])", "[object Object]1,2,33,4,5string12345nulltrueundefined",     myobj.concat([1,2,3],[3,4,5],'string',12345,null,true,undefined).toString());
    
    return array;
}
